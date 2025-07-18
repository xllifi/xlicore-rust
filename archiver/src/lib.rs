use std::{
  fs::{self, File},
  io,
  path::{Path, PathBuf},
};

use flate2::read::GzDecoder;
use shared::error::{Error, ErrorCode};
use tar::EntryType;

/// Strips root if possible (archive contains a single directory in root). `keep_root` prevents this behaviour
pub fn unarchive(file: &Path, destination: &Path, keep_root: bool) -> Result<(), Error> {
  if !file.is_file() {
    return Err("Input is not a file!".into());
  }

  let ext = file
    .extension()
    .and_then(|ext| ext.to_str())
    .ok_or("Couldn't determine archive type")?;
  match ext {
    "tar" | "tar.gz" | "gz" => extract_tar(file, destination, keep_root)?,
    "zip" => extract_zip(file, destination, keep_root)?,
    _ => return Err(format!("Unsupported archive type {ext}").into()),
  }

  Ok(())
}

fn extract_tar(file: &Path, destination: &Path, keep_root: bool) -> Result<(), Error> {
  let file = File::open(file)?;
  let tar = GzDecoder::new(file);
  let mut archive = tar::Archive::new(tar);

  let entries = archive
    .entries()?
    .map(|x| x.map_err(|e| Error::from(e)))
    .collect::<Result<Vec<tar::Entry<'_, GzDecoder<File>>>, Error>>()?;
  let paths = entries
    .iter()
    .map(|e| {
      e.path()
        .and_then(|x| Ok(x.into_owned()))
        .map_err(|err| Error {
          code: ErrorCode::Unknown,
          message: "Couldn't get path of entry".into(),
          verbose: Some(err.to_string()),
        })
    })
    .collect::<Result<Vec<PathBuf>, Error>>()?;

  let strip_root = match keep_root {
    true => false,
    false => count_roots(paths) <= 1,
  };

  for mut entry in entries {
    let outpath = match entry.path() {
      Ok(path) => match strip_root {
        true => {
          let mut components = path.components();
          components.next();
          components.as_path().to_owned()
        }
        false => path.into_owned(),
      },
      Err(err) => {
        return Err(Error {
          code: ErrorCode::Unknown,
          message: "Failed to get path of TAR entry".into(),
          verbose: Some(err.to_string()),
        });
      }
    };
    println!("OG path: {:?} | OUT path: {outpath:?}", entry.path());

    match entry.header().entry_type() {
      EntryType::Regular | EntryType::Link | EntryType::Symlink => {
        if let Some(path) = outpath.parent() {
          if !path.exists() {
            fs::create_dir_all(destination.join(path))?;
          }
        }
        let mut outfile = fs::File::create(destination.join(outpath))?;
        io::copy(&mut entry, &mut outfile)?;
      }
      EntryType::Directory => {
        fs::create_dir_all(destination.join(&outpath))?;
      }
      _ => return Err("Unsupported TAR entry type".into()),
    }
  }

  Ok(())
}

fn extract_zip(file: &Path, destination: &Path, keep_root: bool) -> Result<(), Error> {
  let file = File::open(file)?;
  let mut archive = zip::ZipArchive::new(file).map_err(|e| Error {
    code: ErrorCode::Unknown,
    message: "Couldn't open file as ZIP archive".into(),
    verbose: Some(e.to_string()),
  })?;
  let paths = (0..archive.len())
    .map(|i| {
      archive
        .by_index(i)
        .map_err(|e| Error {
          code: ErrorCode::Unknown,
          message: format!("Couldn't find file index {i} in archive"),
          verbose: Some(e.to_string()),
        })
        .and_then(|f| {
          f.enclosed_name()
            .ok_or(Error::from("Incorrect ZIP file path"))
        })
    })
    .collect::<Result<Vec<PathBuf>, Error>>()?;

  let strip_root = match keep_root {
    true => false,
    false => count_roots(paths) <= 1,
  };

  for i in 0..archive.len() {
    let mut file = archive.by_index(i).map_err(|e| Error {
      code: ErrorCode::Unknown,
      message: format!("Couldn't find file index {i} in archive"),
      verbose: Some(e.to_string()),
    })?;
    let outpath = match file.enclosed_name() {
      Some(path) => match strip_root {
        true => {
          let mut components = path.components();
          components.next();
          components.as_path().to_owned()
        }
        false => path,
      },
      None => continue,
    };

    if file.is_dir() {
      fs::create_dir_all(destination.join(&outpath))?;
    } else {
      if let Some(path) = outpath.parent() {
        if !path.exists() {
          fs::create_dir_all(destination.join(path))?;
        }
      }
      let mut outfile = fs::File::create(destination.join(outpath))?;
      io::copy(&mut file, &mut outfile)?;
    }
  }

  Ok(())
}

fn count_roots<P: AsRef<Path>>(paths: Vec<P>) -> u32 {
  let mut counter = 0;
  for path in paths {
    let path = path.as_ref();
    if path.parent().and_then(|x| x.to_str()).unwrap_or("") == "" {
      counter += 1;
    }
  }
  counter
}

#[cfg(test)]
mod tests {
  use super::*;
  use flate2::{Compression, write::GzEncoder};
  use std::io::{Read, Write};
  use tempfile::{TempDir, tempdir, tempfile};
  use zip::{ZipWriter, write::SimpleFileOptions};

  #[test]
  fn test_count_roots_normal() {
    let paths = vec![
      "root_path1",
      "root_path1/subdir1",
      "root_path1/subdir2",
      "root_path2",
      "root_path3",
    ];
    assert_eq!(count_roots(paths), 3);
  }

  #[test]
  fn test_count_roots_empty_vec() {
    let paths: Vec<&Path> = vec![];
    assert_eq!(count_roots(paths), 0);
  }

  fn read_inner_paths<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>, Error> {
    let paths = dir
      .as_ref()
      .read_dir()?
      .map(|x| x.map_err(|e| Error::from(e)))
      .map(|x| x.and_then(|x| Ok(x.path())))
      .collect::<Result<Vec<PathBuf>, Error>>()?;
    let dirs = paths
      .clone()
      .into_iter()
      .map(|x| {
        if x.is_dir() {
          read_inner_paths(x)
        } else {
          Ok(vec![x])
        }
      })
      .collect::<Result<Vec<Vec<PathBuf>>, Error>>()?;
    Ok(dirs.into_iter().flatten().collect::<Vec<PathBuf>>())
  }

  fn make_tar_archive(tempdir: &TempDir) -> PathBuf {
    let tar_path = tempdir.path().join("archive.tar");
    let tar_file = File::create(&tar_path).unwrap();
    let mut tar_archive = tar::Builder::new(tar_file);

    let mut file1 = tempfile().unwrap();
    file1.write(b"Hello world").unwrap();
    let mut file2 = tempfile().unwrap();
    file2.write(b"Hello world!").unwrap();

    #[rustfmt::skip]
    {
      tar_archive.append_file("subdir/file1.txt", &mut file1).unwrap();
      tar_archive.append_file("subdir/file2.txt", &mut file2).unwrap();
    };
    tar_archive.finish().unwrap();

    let targz_path = tempdir.path().join("archive.tar.gz");
    let targz_file = File::create(&targz_path).unwrap();
    let mut encoder = GzEncoder::new(targz_file, Compression::default());

    let mut tar_read = File::open(tar_path).unwrap();
    let mut buf: Vec<u8> = vec![];
    tar_read.read_to_end(&mut buf).unwrap();
    encoder.write(&buf).unwrap();
    encoder.finish().unwrap();

    targz_path
  }

  #[test]
  fn test_extract_tar_keep_root() {
    let tempdir = tempdir().unwrap();

    let archive_path = make_tar_archive(&tempdir);
    let destination = tempdir.path().join("unpack");

    extract_tar(&archive_path, &destination, true).unwrap();
    let mut paths = read_inner_paths(&destination).unwrap();

    paths.sort();
    let mut correct = vec![
      destination.join("subdir/file1.txt"),
      destination.join("subdir/file2.txt"),
    ];
    correct.sort();
    assert_eq!(paths, correct);
  }

  #[test]
  fn test_extract_tar_strip_root() {
    let tempdir = tempdir().unwrap();

    let archive_path = make_tar_archive(&tempdir);
    let destination = tempdir.path().join("unpack");

    extract_tar(&archive_path, &destination, false).unwrap();
    let mut paths = read_inner_paths(&destination).unwrap();

    paths.sort();
    let mut correct = vec![destination.join("file1.txt"), destination.join("file2.txt")];
    correct.sort();
    assert_eq!(paths, correct);
  }

  fn make_zip_archive(tempdir: &TempDir) -> PathBuf {
    let archive_path = tempdir.path().join("archive.zip");
    let archive_file = File::create(&archive_path).unwrap();
    let mut zip_archive = ZipWriter::new(archive_file);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip_archive.start_file("subdir/file1.txt", options).unwrap();
    zip_archive.write(b"Hello!").unwrap();
    zip_archive.start_file("subdir/file2.txt", options).unwrap();
    zip_archive.write(b"Hello!").unwrap();
    zip_archive.finish().unwrap();

    archive_path
  }

  #[test]
  fn test_extract_zip_keep_root() {
    let tempdir = tempdir().unwrap();

    let archive_path = make_zip_archive(&tempdir);
    let destination = tempdir.path().join("unpack");

    extract_zip(&archive_path, &destination, true).unwrap();
    let mut paths = read_inner_paths(&destination).unwrap();

    paths.sort();
    let mut correct = vec![
      destination.join("subdir/file1.txt"),
      destination.join("subdir/file2.txt"),
    ];
    correct.sort();
    assert_eq!(paths, correct);
  }

  #[test]
  fn test_extract_zip_strip_root() {
    let tempdir = tempdir().unwrap();

    let archive_path = make_zip_archive(&tempdir);
    let destination = tempdir.path().join("unpack");

    extract_zip(&archive_path, &destination, false).unwrap();
    let mut paths = read_inner_paths(&destination).unwrap();

    paths.sort();
    let mut correct = vec![destination.join("file1.txt"), destination.join("file2.txt")];
    correct.sort();
    assert_eq!(paths, correct);
  }
}
