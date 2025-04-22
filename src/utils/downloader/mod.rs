mod model;
pub use model::*;

use futures_util::StreamExt;
use log::{debug, error};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::from_reader;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{
  cmp::min,
  fs::{exists, remove_file},
  io::{BufReader, Read},
  path::PathBuf,
};
use std::{
  error::Error,
  fs::{File, create_dir_all, rename},
  io::Write,
  path::Path,
};

use crate::utils::helpers::hex_to_bytes;

impl Downloader {
  pub fn new(temp_suffix: String, default_opts: Option<DownloaderOpts>) -> Self {
    Downloader {
      temp_suffix,
      default_opts,
      reqwest_client: Client::new(),
    }
  }

  pub async fn single_download(
    &self,
    file: &mut DownloaderFile,
    opts: Option<&DownloaderOpts>,
  ) -> Result<PathBuf, String> {
    let og_retries = file.retries.clone();
    loop {
      match self.exec_download(file, opts).await {
        Ok(res) => return Ok(res),
        Err(err) => {
          if file.retries <= 0 {
            return Err(format!(
              "Failed to download {} after {} retries",
              file.url, og_retries
            ));
          }
          error!(
            "Failed to download file {}, with error '{}', retrying {} more time(s)",
            file.url, err, file.retries
          );
          file.retries -= 1;
        }
      }
    }
  }

  pub async fn single_download_get_json<T: DeserializeOwned>(
    &self,
    file: &mut DownloaderFile,
    opts: Option<&DownloaderOpts>,
  ) -> Result<T, String> {
    let path = self.single_download(file, opts).await?;

    let file = match File::open(path) {
      Ok(res) => res,
      Err(err) => return Err(format!("{}", err.to_string())),
    };
    let reader = BufReader::new(file);

    Ok(match from_reader(reader) {
      Ok(res) => res,
      Err(err) => return Err(format!("{}", err.to_string())),
    })
  }

  async fn exec_download(
    &self,
    file: &mut DownloaderFile,
    opts: Option<&DownloaderOpts>,
  ) -> Result<PathBuf, Box<dyn Error>> {
    debug!(
      "Requested to download file {} with opts {:?}",
      file.url, opts
    );

    // Verify options
    let file_name: String = match &file.name {
      Some(val) => val.clone(),
      None => file.url.split('/').next_back().unwrap().into(),
    };
    debug!("Resolved file_name: {:?}", file_name);

    let resolved_opts: DownloaderOpts = match &self.default_opts {
      Some(default_opts) => match opts {
        Some(opts) => default_opts.merge(opts),
        None => default_opts.clone(),
      },
      None => match opts {
        Some(opts) => opts.clone(),
        None => DownloaderOpts::default(),
      },
    };

    // Verify dir and create file
    let final_path = Path::new(&file.dir).join(&file_name);
    let temp_path = Path::new(&file.dir).join(format!("{0}{1}", &file_name, &self.temp_suffix));
    if let Some(path) = final_path.parent() {
      create_dir_all(path)?;
    }
    let final_path_exists: bool = exists(&final_path)?;
    if resolved_opts.overwrite.unwrap_or(false) {
      if final_path_exists {
        remove_file(&final_path)?;
      }
    } else if let Some(verify) = &file.verify {
      if final_path_exists {
        let file = File::open(&final_path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Hasher::new(verify.algorithm);

        let mut buffer = [0; 65536];

        loop {
          let count = reader.read(&mut buffer)?;
          if count == 0 {
            break;
          }
          hasher.update(&buffer[..count]);
        }

        if !Downloader::check_hashes(hasher, verify) {
          return Err(Box::new(DownloaderError {
            cause: DownloaderErrorCauses::VerifyFailed,
            details: "Failed to verify file hash".into(),
          }));
        } else {
          return Ok(final_path);
        }
      }
    }
    if exists(&temp_path)? {
      remove_file(&temp_path)?;
    }
    let mut temp_file = File::create_new(&temp_path)?;

    // Execute download
    let resp = match self.reqwest_client.get(&file.url).send().await {
      Ok(res) => res,
      Err(err) => {
        return Err(Box::new(DownloaderError {
          cause: DownloaderErrorCauses::HttpFailed,
          details: err.to_string(),
        }));
      }
    };

    let file_size = resp.content_length().or(file.size).ok_or(format!(
      "Failed to get file size from request or file meta for {}",
      &file.url
    ))?;
    // TODO^ make it NOT error but just disable progress
    debug!("Resolved file_size: {}", file_size);

    // Write from stream
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    let mut hasher = Hasher::new(file.verify.as_ref().unwrap().algorithm);

    while let Some(item) = stream.next().await {
      let chunk = item.or(Err(DownloaderError {
        cause: DownloaderErrorCauses::BrokenStream,
        details: format!("Failed to download file '{}'", final_path.to_str().unwrap()),
      }))?;
      temp_file.write_all(&chunk)?;
      hasher.update(&chunk);
      downloaded = min(downloaded + (chunk.len() as u64), file_size);

      if let Some(progress_callback) = resolved_opts.on_download_progress {
        let progress: DownloaderFileProgress = DownloaderFileProgress {
          file_size,
          downloaded_bytes: downloaded,
        };
        progress_callback(progress, chunk)
      }
    }

    match &file.verify {
      Some(verify) => {
        if !Downloader::check_hashes(hasher, &verify) {
          return Err(Box::new(DownloaderError {
            cause: DownloaderErrorCauses::VerifyFailed,
            details: "Failed to verify file hash".into(),
          }));
        }
      }
      None => (),
    }

    rename(&temp_path, &final_path)?;
    Ok(final_path)
  }

  fn check_hashes(hasher: Hasher, verify: &DownloaderVerify) -> bool {
    let hash: Vec<u8> = hasher.finalize();
    #[cfg(debug_assertions)]
    {
      let verify_hash_u8slice = hex_to_bytes(&verify.hash).unwrap();
      if hash == verify_hash_u8slice {
        debug!("HASHES MATCH:");
        debug!("{:x?} and {:x?}", hash, verify_hash_u8slice);
      } else {
        debug!("HASHES NON-MATCH:");
        debug!("{:x?} and {:x?}", hash, verify_hash_u8slice);
      }
    }
    hash == hex_to_bytes(&verify.hash).unwrap()
  }
}

impl Hasher {
  fn new(algorithm: DownloaderAlgorithm) -> Self {
    match algorithm {
      DownloaderAlgorithm::Sha1 => Hasher::Sha1(Sha1::new()),
      DownloaderAlgorithm::Sha256 => Hasher::Sha256(Sha256::new()),
    }
  }
  fn update(&mut self, data: &[u8]) {
    match self {
      Hasher::Sha1(hasher) => hasher.update(data),
      Hasher::Sha256(hasher) => hasher.update(data),
    }
  }

  fn finalize(self) -> Vec<u8> {
    match self {
      Hasher::Sha1(hasher) => hasher.finalize().to_vec(),
      Hasher::Sha256(hasher) => hasher.finalize().to_vec(),
    }
  }
}
