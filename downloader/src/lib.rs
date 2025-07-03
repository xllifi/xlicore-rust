pub mod error;
pub mod hasher;
pub mod helpers;
pub mod module;
use std::{
  fs::{File, create_dir_all, remove_file, rename},
  io::{BufReader, Read, Write},
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
    mpsc::{self, Sender},
  },
  thread,
  time::Duration,
};

use error::*;
use futures_util::{StreamExt, future::try_join_all};
use log::debug;
use module::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reqwest::Client;

use crate::hasher::Hasher;

impl Downloader {
  pub fn new(temp_suffix: String) -> Self {
    Downloader {
      temp_suffix,
      reqwest_client: Client::new(),
    }
  }

  pub async fn download(&self, request: DownloaderRequest) -> Result<Vec<PreppedFile>, Error> {
    debug!("Requested download for {} file(s)", request.files.len());

    // Verify variables
    let DownloaderRequest {
      request_type,
      mut retries,
      overwrite,
      channel_sender,
      files,
    } = request;
    let request = PreppedRequest {
      request_type,
      overwrite,
    };
    let mut total_size: u64;

    // Prepare files array
    let mut files = self.prep_files(&request, files).await?;
    let ret_files = files.clone();
    // Compensate for first run
    retries += 1;

    while files.len() > 0 {
      total_size = Self::calc_total_size(&files)?;

      if retries <= 0 {
        let filenames: Vec<String> = files.into_iter().map(|f| f.name).collect();
        channel_sender
          .send(DownloaderChannelMessage::Finish {
            success: false,
            failed_files: Some(filenames.clone()),
          })
          .unwrap();
        return Err(Error {
          code: ErrorCode::VerifyFailed,
          message: format!("Failed to download files: [{}]", &filenames.join(", ")),
        });
      }

      // Download files
      self
        .download_files(channel_sender.clone(), total_size, &mut files, request_type)
        .await?;
      // Verify files
      Self::verify_files(&mut files, channel_sender.clone());

      debug!(
        "Files left after verify: [{}]",
        files
          .iter()
          .map(|f| f.name.clone())
          .collect::<Vec<String>>()
          .join(", ")
      );

      retries -= 1;
    }

    // Download fully done
    channel_sender
      .send(DownloaderChannelMessage::Finish {
        success: true,
        failed_files: None,
      })
      .unwrap();

    Ok(ret_files)
  }

  fn calc_total_size(files: &Vec<PreppedFile>) -> Result<u64, Error> {
    Ok(files.iter().map(|x| x.size).sum::<u64>())
  }

  fn setup_progress(
    request_type: RequestType,
    channel_sender: Sender<DownloaderChannelMessage>,
    total_size: u64,
  ) -> Result<(Option<Arc<AtomicU64>>, Option<Sender<InternalMessage>>), Error> {
    let enable_progress = total_size != 0;
    let progress = match enable_progress {
      true => Some(Arc::new(AtomicU64::new(0))),
      false => None,
    };
    let done_tx = match enable_progress {
      true => {
        channel_sender
          .send(DownloaderChannelMessage::Start {
            request_type,
            progress_enabled: true,
          })
          .unwrap();
        let (done_tx, done_rx) = mpsc::channel::<InternalMessage>();

        let progress = progress.clone().unwrap();
        let channel_sender = channel_sender.clone();
        thread::spawn(move || -> Result<(), Error> {
          loop {
            if done_rx.try_recv().is_ok() {
              break;
            }
            channel_sender
              .send(DownloaderChannelMessage::Progress {
                file_size_bytes: total_size,
                downloaded_bytes: progress.load(Ordering::Relaxed),
              })
              .unwrap();
            thread::sleep(Duration::from_millis(250));
          }
          Ok(())
        });

        Some(done_tx)
      }
      false => {
        channel_sender
          .send(DownloaderChannelMessage::Start {
            request_type,
            progress_enabled: false,
          })
          .unwrap();

        None
      }
    };
    Ok((progress, done_tx))
  }

  async fn prep_files(
    &self,
    request: &PreppedRequest,
    files: Vec<DownloaderFile>,
  ) -> Result<Vec<PreppedFile>, Error> {
    let files: Vec<_> = files
      .into_iter()
      .map(|file| self.prep_file(request, file))
      .collect();
    try_join_all(files).await
  }

  async fn prep_file(
    &self,
    request: &PreppedRequest,
    file: DownloaderFile,
  ) -> Result<PreppedFile, Error> {
    // Make sure file name is some
    let file_name = match file.name {
      Some(name) => name,
      None => helpers::filename_from_url(&file.url)?,
    };
    debug!("Resolved file_name: {}", &file_name);

    // Make sure file.size is not 0
    let file_size = if file.size == 0 {
      debug!(
        "No file.size for {}, fetching via HEAD request!",
        &file_name
      );
      let resp = self
        .reqwest_client
        .head(&file.url)
        .send()
        .await
        .map_err(|_| Error::unknown("Failed to fetch file size"))?;
      if let Some(size) = resp.content_length() {
        size
      } else {
        0
      }
    } else {
      file.size
    };

    // Make sure dirs exist
    create_dir_all(&file.dir)?;
    let final_path = Path::new(&file.dir).join(&file_name);
    let temp_path = Path::new(&file.dir).join(file_name.clone() + self.temp_suffix.as_str());

    if final_path.exists() && request.overwrite {
      remove_file(&final_path)
        .map_err(|e| Error::unknown(format!("Failed to remove file {}: {e}", &file_name)))?;
    }

    Ok(PreppedFile {
      url: file.url,
      final_path,
      temp_path,
      name: file_name,
      size: file_size,
      verify: file.verify,
    })
  }

  async fn download_files(
    &self,
    channel_sender: Sender<DownloaderChannelMessage>,
    total_size: u64,
    files: &mut Vec<PreppedFile>,
    request_type: RequestType,
  ) -> Result<(), Error> {
    // Progress tracking
    let (progress, done_tx) =
      Self::setup_progress(request_type, channel_sender.clone(), total_size)?;

    // Start downloads
    let files: Vec<_> = files
      .into_iter()
      .map(|file| {
        debug!("Downloading file URL {}", file.url);
        self.download_file(file, &progress)
      })
      .collect();
    try_join_all(files).await?;

    // Stop progress tracking thread if it was spawned
    if let Some(done_tx) = done_tx {
      done_tx.send(InternalMessage::Finish).unwrap();
    };
    Ok(())
  }

  async fn download_file(
    &self,
    file: &mut PreppedFile,
    progress: &Option<Arc<AtomicU64>>,
  ) -> Result<(), Error> {
    if file.temp_path.exists() {
      remove_file(&file.temp_path)
        .map_err(|e| Error::unknown(format!("Failed to remove file {:?}: {e}", file.temp_path)))?;
    }
    let mut temp_file = File::create_new(&file.temp_path)
      .map_err(|e| Error::unknown(format!("Failed to create file {:?}: {e}", file.temp_path)))?;

    let resp = match self.reqwest_client.get(&file.url).send().await {
      Ok(res) => res,
      Err(err) => return Err(Error::unknown(err.to_string())),
    };
    if !resp.status().is_success() {
      return Err(resp.into());
    }

    // if let Some(content_length) = resp.content_length() {
    //   if content_length != file.size {
    //     file.size = content_length
    //   }
    // }

    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
      let chunk = item.or(Err(Error {
        code: ErrorCode::Unknown,
        message: format!(
          "Failed to download file '{}'",
          file.final_path.to_str().unwrap()
        ),
      }))?;
      temp_file
        .write_all(&chunk)
        .map_err(|e| Error::unknown(format!("Failed to write to file: {e}")))?;
      if let Some(progress) = progress {
        progress.fetch_add(chunk.len() as u64, Ordering::Relaxed);
      }
    }

    rename(&file.temp_path, &file.final_path).map_err(|e| {
      Error::unknown(format!(
        "Failed to rename {:?} to  {:?}: {e}",
        file.temp_path, file.final_path
      ))
    })?;

    debug!("Successfuly downloaded {}", file.name);

    Ok(())
  }

  /// Removes all files that verified successfully from passed array
  fn verify_files(
    files: &mut Vec<PreppedFile>,
    channel_sender: Sender<DownloaderChannelMessage>,
  ) -> () {
    let (tx, rx) = mpsc::channel();
    let total_files = files.len() as u32;
    thread::spawn(move || {
      let mut verified_files: u32 = 0;
      while rx.recv().is_ok() && verified_files < total_files {
        verified_files += 1;
        channel_sender
          .send(DownloaderChannelMessage::Verify {
            total_files,
            verified_files,
          })
          .unwrap();
      }
    });

    let keep = files
      .par_iter()
      .map(|file| {
        let result = Self::verify_file(file).is_err(); // Only keep bad files
        tx.send(()).unwrap();
        result
      })
      .collect::<Vec<bool>>();
    let mut iter = keep.iter();
    files.retain(|_| *iter.next().unwrap());
  }

  fn verify_file(file: &PreppedFile) -> Result<(), Error> {
    if !file.final_path.exists() {
      return Err(Error::unknown("File doesn't exist!"));
    };
    let verify = match file.verify.clone() {
      Some(val) => val,
      None => unreachable!(),
    };

    let mut hasher = Hasher::new(verify.algorithm);

    let fsfile = File::open(&file.final_path)?;
    let mut reader = BufReader::new(fsfile);
    let mut buf = [0u8; 512];
    while let Ok(n) = reader.read(&mut buf) {
      if n == 0 {
        break;
      }
      hasher.update(&buf[..n]);
    }

    let hash = hasher.finalize();
    if hex::encode(hash) != verify.hash {
      return Err(Error {
        code: ErrorCode::Unknown,
        message: format!("Couldn't verify file {}! (hashes mismatch)", file.name),
      });
    }

    Ok(())
  }
}
