mod model;
use bytes::Bytes;
pub use model::*;

use futures_util::{StreamExt, future};
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
  sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
  },
  time::{SystemTime, UNIX_EPOCH},
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

  pub async fn multiple_download(
    &self,
    files: Vec<DownloaderFile>,
    opts: Option<&DownloaderOpts>,
  ) -> Result<(), String> {
    let opts = match opts {
      Some(opts) => opts.clone(),
      None => DownloaderOpts::default(),
    };
    let total_size = match opts.total_size {
      Some(total_size) => Ok(total_size),
      None => Err("Must provide opts with total_size!"),
    }?;

    let total_downloaded = Arc::new(AtomicU64::new(0));

    // let chunks: Vec<Vec<&DownloaderFile>> = files.chunks(16).map(|x| x.into()).collect();

    let on_progress_total_size = total_size.clone();
    let on_progress_total_downloaded = total_downloaded.clone();
    let on_progress_from_opts = opts.on_download_progress.clone();
    let on_progress: DownloaderOptsProgressCallback = Arc::new(
      move |current_progress: DownloaderFileProgress,
            chunk: Bytes,
            file: &DownloaderFile,
            last_progress: &DownloaderFileLastProgress| {
        on_progress_total_downloaded.fetch_add(
          current_progress.downloaded_bytes - last_progress.downloaded_bytes,
          Ordering::SeqCst,
        );

        debug!(
          "MULDL PROGR: {}/{}",
          on_progress_total_downloaded.load(Ordering::Relaxed),
          on_progress_total_size
        );

        if on_progress_from_opts.is_some() {
          let callable = on_progress_from_opts.clone().unwrap();
          callable(current_progress, chunk, file, last_progress)
        }
      },
    );

    let on_finish_total_size = total_size.clone();
    let on_finish_total_downloaded = total_downloaded.clone();
    let on_finish_from_opts = opts.on_download_finish.clone();
    let on_finish: DownloaderOptsFinishCallback = Arc::new(move |file: &DownloaderFile, last_progress: &DownloaderFileLastProgress| {
      if let Some(size) = file.size {
        on_finish_total_downloaded.fetch_add(size - last_progress.downloaded_bytes, Ordering::SeqCst);
      }

      debug!(
        "MULDL FNISH: {}/{}",
        on_finish_total_downloaded.load(Ordering::Relaxed),
        on_finish_total_size
      );

      if on_finish_from_opts.is_some() {
        let callable = on_finish_from_opts.clone().unwrap();
        callable(file, last_progress)
      }
    });

    let mul_opts: DownloaderOpts = DownloaderOpts {
      on_download_progress: Some(on_progress),
      on_download_finish: Some(on_finish),
      overwrite: None,
      total_size: None,
    };

    let mopts = opts.merge(&mul_opts);

    let fututes = files
      .iter()
      .map(async |file| self.single_download(file, Some(&mopts)).await);

    future::join_all(fututes).await;

    Ok(())
  }

  pub async fn single_download(
    &self,
    file: &DownloaderFile,
    opts: Option<&DownloaderOpts>,
  ) -> Result<PathBuf, String> {
    let default_opts = &DownloaderOpts::default();
    let mut opts = opts.unwrap_or(default_opts).clone();
    let og_retries = file.retries.clone();
    let mut retries_remain = file.retries.clone();
    loop {
      match self.exec_download(file, Some(&opts)).await {
        Ok(res) => return Ok(res),
        Err(err) => {
          if retries_remain <= 0 {
            return Err(format!(
              "Failed to download {} after {} retries",
              file.url, og_retries
            ));
          }
          error!(
            "Failed to download file {}, with error '{}', retrying {} more time(s)",
            file.url, err, retries_remain
          );
          retries_remain -= 1;
        }
      }
      opts.overwrite = Some(true);
    }
  }

  pub async fn single_download_get_json<T: DeserializeOwned>(
    &self,
    file: &DownloaderFile,
    opts: Option<&DownloaderOpts>,
  ) -> Result<T, String> {
    let path = self.single_download(file, opts).await?;

    let file = File::open(path).map_err(|x| x.to_string())?;
    let reader = BufReader::new(file);

    from_reader(reader).map_err(|x| x.to_string())
  }

  async fn exec_download(
    &self,
    file: &DownloaderFile,
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

    let opts: DownloaderOpts = match &self.default_opts {
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
    if final_path_exists {
      if opts.overwrite.unwrap_or(false) {
        remove_file(&final_path)?;
      } else {
        if let Some(verify) = &file.verify {
          let fs_file = File::open(&final_path)?;
          let mut reader = BufReader::new(fs_file);
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
            debug!("Skipping download because existing file's hash is okay");
            if let Some(callable) = opts.on_download_finish {
              callable(file, &DownloaderFileLastProgress::default())
            }
            return Ok(final_path);
          }
        }
      }
    }
    if exists(&temp_path)? {
      remove_file(&temp_path)?;
    }
    let mut temp_file = File::create_new(&temp_path)?;

    // Start executing download
    let resp = match self.reqwest_client.get(&file.url).send().await {
      Ok(res) => res,
      Err(err) => {
        return Err(Box::new(DownloaderError {
          cause: DownloaderErrorCauses::HttpFailed,
          details: err.to_string(),
        }));
      }
    };

    let file_size = resp.content_length().or(file.size).unwrap_or(0);
    let progress_ok: bool = file_size > 0;
    debug!("Resolved file_size: {}", file_size);

    // Write from stream
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    let mut hasher: Option<Hasher> = match &file.verify {
      Some(verify) => Some(Hasher::new(verify.algorithm)),
      None => None,
    };

    let mut last_progress: DownloaderFileLastProgress = DownloaderFileLastProgress {
      downloaded_bytes: 0,
      timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?,
    };
    while let Some(item) = stream.next().await {
      let chunk = item.or(Err(DownloaderError {
        cause: DownloaderErrorCauses::BrokenStream,
        details: format!("Failed to download file '{}'", final_path.to_str().unwrap()),
      }))?;
      temp_file.write_all(&chunk)?;
      match &mut hasher {
        Some(hasher) => hasher.update(&chunk),
        None => (),
      }
      downloaded = min(downloaded + (chunk.len() as u64), file_size);

      if let Some(ref progress_callback) = opts.on_download_progress {
        let progress: DownloaderFileProgress = DownloaderFileProgress {
          file_size,
          downloaded_bytes: downloaded,
          ok: progress_ok,
        };
        progress_callback(progress, chunk, file, &last_progress)
      }

      last_progress.downloaded_bytes = downloaded;
      last_progress.timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?;
    }

    match &file.verify {
      Some(verify) => match hasher {
        Some(hasher) => {
          if !Downloader::check_hashes(hasher, &verify) {
            return Err(Box::new(DownloaderError {
              cause: DownloaderErrorCauses::VerifyFailed,
              details: "Failed to verify file hash".into(),
            }));
          }
        }
        None => (),
      },
      None => (),
    }

    rename(&temp_path, &final_path)?;

    if let Some(callable) = opts.on_download_finish {
      callable(file, &last_progress)
    }
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
