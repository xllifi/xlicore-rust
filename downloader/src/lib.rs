pub mod error;
pub mod hasher;
pub mod helpers;
pub mod module;
use std::{
  fs::{self, create_dir_all, remove_file, rename},
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
use uuid::Uuid;

use crate::{hasher::Hasher, module::File};

impl Downloader {
  pub fn new(temp_suffix: String, channel_sender: Sender<ChannelMessage>, overwrite: bool) -> Self {
    Downloader {
      reqwest_client: Client::new(),
      temp_suffix,
      channel_sender,
      overwrite,
    }
  }

  // Shared functions

  fn setup_progress(
    &self,
    data: &RequestData,
    total: u64,
  ) -> Result<Option<(Arc<AtomicU64>, Sender<()>)>, Error> {
    if total <= 0 {
      debug!("total is none, progress not reported");
      return Ok(None);
    }

    let progress = Arc::new(AtomicU64::new(0));
    let done_tx = {
      let (done_tx, done_rx) = mpsc::channel::<()>();

      let progress = progress.clone();
      let channel_sender = self.channel_sender.clone();
      let data = data.clone();
      thread::spawn(move || -> Result<(), Error> {
        loop {
          if done_rx.try_recv().is_ok() {
            break;
          }
          channel_sender
            .send(ChannelMessage::Progress {
              data,
              file_size_bytes: total,
              downloaded_bytes: progress.load(Ordering::Relaxed),
            })
            .unwrap();
          thread::sleep(Duration::from_millis(250));
        }
        Ok(())
      });

      done_tx
    };
    Ok(Some((progress, done_tx)))
  }

  async fn prep_files(
    &self,
    files: Vec<File>,
    ignore_size: bool,
  ) -> Result<Vec<PreppedFile>, Error> {
    if files.len() <= 0 {
      return Err(Error {
        code: ErrorCode::NoFiles,
        message: "No files requested!".into()
      })
    };
    let files: Vec<_> = files
      .into_iter()
      .map(|file| self.prep_file(file, ignore_size))
      .collect();
    try_join_all(files).await
  }

  async fn prep_file(&self, file: File, ignore_size: bool) -> Result<PreppedFile, Error> {
    // Make sure file name is some
    let file_name = match file.name {
      Some(name) => name,
      None => helpers::filename_from_url(&file.url)?,
    };
    debug!("Resolved file_name: {}", &file_name);

    // Make sure file.size is not 0
    let file_size = if file.size == 0 && !ignore_size {
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
      resp.content_length().unwrap_or(0)
    } else {
      file.size
    };

    // Make sure dirs exist
    create_dir_all(&file.dir)?;
    let final_path = Path::new(&file.dir).join(&file_name);
    let temp_path = Path::new(&file.dir).join(file_name.clone() + self.temp_suffix.as_str());

    if final_path.exists() && self.overwrite {
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

  fn send_start(&self, data: &RequestData, progress_enabled: bool) {
    self.channel_sender
      .send(ChannelMessage::Start {
        data: *data,
        progress_enabled,
      })
      .unwrap();
  }
  fn send_finish(&self, data: &RequestData) {
    self.channel_sender
      .send(ChannelMessage::Finish { data: *data })
      .unwrap();
  }

  // Download

  fn calc_total_size(files: &Vec<PreppedFile>) -> Result<u64, Error> {
    Ok(files.iter().map(|x| x.size).sum::<u64>())
  }

  pub async fn download(&self, files: &Vec<File>) -> Result<(), Error> {
    debug!("Requested download for {} file(s)", files.len());
    let data = RequestData {
      id: Uuid::new_v4(),
      action: Action::Download,
    };

    // Prepare files array
    let files = self.prep_files(files.clone(), false).await?;
    let total_size = Self::calc_total_size(&files)?;
    debug!("{}", total_size);

    // Download files
    // Progress tracking
    let tracking = self.setup_progress(&data, total_size)?;
    let progress = tracking.clone().map(|(x, _)| x);
    let done_tx = tracking.map(|(_, x)| x);
    self.send_start(&data, progress.is_some());

    // Start downloads
    let futures: Vec<_> = files
      .iter()
      .map(|file| {
        debug!("Downloading file URL {}", file.url);
        self.download_file(file, &progress)
      })
      .collect();
    try_join_all(futures).await?;

    // Stop progress tracking thread if it was spawned
    if let Some(done_tx) = done_tx {
      done_tx.send(()).unwrap();
    };

    // Download fully done
    self.send_finish(&data);

    Ok(())
  }

  async fn download_file(
    &self,
    file: &PreppedFile,
    progress: &Option<Arc<AtomicU64>>,
  ) -> Result<(), Error> {
    if file.temp_path.exists() {
      remove_file(&file.temp_path)
        .map_err(|e| Error::unknown(format!("Failed to remove file {:?}: {e}", file.temp_path)))?;
    }
    let mut temp_file = fs::File::create_new(&file.temp_path)
      .map_err(|e| Error::unknown(format!("Failed to create file {:?}: {e}", file.temp_path)))?;

    let resp = match self.reqwest_client.get(&file.url).send().await {
      Ok(res) => res,
      Err(err) => return Err(Error::unknown(err.to_string())),
    };
    if !resp.status().is_success() {
      return Err(resp.into());
    }

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

  // Verify

  /// This function operates in place, removing all files that passed verification.
  /// Returns a vector with each file verification results.
  pub async fn verify(&self, files: &mut Vec<File>) -> Result<Vec<Result<(), Error>>, Error> {
    debug!("Requested verify for {} file(s)", files.len());
    let data = RequestData {
      id: Uuid::new_v4(),
      action: Action::Verify,
    };

    // Prepare files array
    let prep_files = self.prep_files(files.clone(), true).await?;
    let total_size: u64 = prep_files.len() as u64;

    // Progress tracking
    let (progress, done_tx) = self.setup_progress(&data, total_size)?.unwrap();
    self.send_start(&data, true);

    let results = prep_files
      .par_iter()
      .map(|file| {
        let result = Self::verify_file(file); // Only keep bad files
        progress.fetch_add(1, Ordering::Relaxed);
        result
      })
      .collect::<Vec<Result<(), Error>>>();
    let mut iter = results.iter().map(|result| result.is_err());

    // Stop progress tracking thread
    done_tx.send(()).unwrap();
    self.send_finish(&data);

    files.retain(|_| iter.next().unwrap());

    Ok(results)
  }

  fn verify_file(file: &PreppedFile) -> Result<(), Error> {
    if !file.final_path.exists() {
      return Err(Error::unknown("File doesn't exist!"));
    };
    let verify = match file.verify.clone() {
      Some(val) => val,
      None => return Err(format!("Couldn't verify file {}! (no verify data)", file.name).into()),
    };

    let mut hasher = Hasher::new(verify.algorithm);

    let fsfile = fs::File::open(&file.final_path)?;
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
