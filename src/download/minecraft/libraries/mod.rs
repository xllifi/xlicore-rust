mod model;
use std::{env::consts::{ARCH, OS}, error::Error, path::{Path, PathBuf}};


use crate::utils::downloader::{Downloader, DownloaderAlgorithm, DownloaderFile, DownloaderFileTypes, DownloaderOpts, DownloaderVerify};
use super::meta::{MinecraftPackageManifest, MinecraftPackageManifestLibrary, MinecraftPackageManifestLibraryBase};

pub async fn download_libraries(dl: &Downloader, package_manifest: &MinecraftPackageManifest) -> Result<Vec<PathBuf>, Box<dyn Error>> {
  let mut files: Vec<DownloaderFile> = vec![];
  let mut classpath: Vec<PathBuf> = vec![];

  'libLoop: for lib in &package_manifest.libraries {
    let (file, path) = match &lib {
        MinecraftPackageManifestLibrary::Simple(lib) => simple_lib_to_dlfile(lib)?,
        MinecraftPackageManifestLibrary::Ruled(lib) => {
          for rule in &lib.rules {
            let os_match = match &rule.os.name {
                Some(val) => val.as_str() == OS,
                None => true,
            };
            let arch_match = match &rule.os.arch {
                Some(val) => val.as_str() == ARCH,
                None => true,
            };
            if !os_match || !arch_match { continue 'libLoop }
          }
          simple_lib_to_dlfile(lib)?
        },
    };
    classpath.push(path);
    files.push(file);
  }

  let client_jar_file = DownloaderFile {
    url: package_manifest.downloads.client.url.clone(),
    dir: format!("store/{}", package_manifest.id), // TODO: rootdir
    name: Some(format!("{}.jar", package_manifest.id)),
    size: Some(package_manifest.downloads.client.size),
    verify: Some(DownloaderVerify {
      hash: package_manifest.downloads.client.sha1.clone(),
      algorithm: DownloaderAlgorithm::Sha1
    }),
    file_type: DownloaderFileTypes::Game,
    retries: 2
  };
  classpath.push(PathBuf::from(&client_jar_file.dir).join(&client_jar_file.name.clone().unwrap()));
  files.push(client_jar_file);

  let total_size = files.iter().map(|file| file.size
    .unwrap_or(0))
    .reduce(|acc, cv| acc + cv ).unwrap_or(0);

  let opts: DownloaderOpts = DownloaderOpts {
    total_size: Some(total_size),
    ..Default::default()
  };

  dl.multiple_download(files, Some(&opts)).await?;

  Ok(classpath)
}

fn simple_lib_to_dlfile(lib: &dyn MinecraftPackageManifestLibraryBase) -> Result<(DownloaderFile, PathBuf), String> {
  let path = PathBuf::from(&lib.downloads().artifact.path);
  let dir: String = Path::new("./store").join(&path).parent().ok_or(
    format!(
      "Library {} has path without parent ({})!",
      lib.name(),
      lib.downloads().artifact.path
    )
  )?.to_str().ok_or(
    format!(
      "Failed to convert dirname of library {} to string",
      lib.name()
    )
  )?.into();

  let name: String = path.file_name().ok_or(
    format!(
      "Library {} has path with incorrect filename ({})!",
      lib.name(),
      lib.downloads().artifact.path
    )
  )?.to_str().ok_or(
    format!(
      "Failed to convert filename of library {} to string",
      lib.name()
    )
  )?.into();

  let file = DownloaderFile {
    url: lib.downloads().artifact.url.clone(),
    dir,
    name: Some(name),
    size: Some(lib.downloads().artifact.size),
    verify: Some(DownloaderVerify {
      hash: lib.downloads().artifact.sha1.clone(),
      algorithm: DownloaderAlgorithm::Sha1
    }),
    file_type: DownloaderFileTypes::Library,
    retries: 2
  };

  return Ok((file, path))
}