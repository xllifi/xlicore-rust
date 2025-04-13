mod model;
pub use model::*;

use futures_util::StreamExt;
use log::debug;
use reqwest::Client;
use std::{
  cmp::min,
  time::{SystemTime, UNIX_EPOCH},
};
use std::{
  error::Error,
  fs::{File, create_dir_all, rename},
  io::Write,
  path::Path,
};

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
    file: DownloaderFile,
    opts: Option<&DownloaderOpts>,
  ) -> Result<(), Box<dyn Error>> {
    debug!(
      "Requested to download file {:?} with opts {:?}",
      file.url, opts
    );

    // Verify options
    let file_name: String = match file.name {
      Some(val) => val,
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

    // Execute download
    let resp = self
      .reqwest_client
      .get(&file.url)
      .send()
      .await
      .or(Err(format!("Failed to GET {}", &file.url)))?;
    let file_size = resp
      .content_length()
      .ok_or(format!("Failed to get content-length for {}", &file.url))?;
    debug!("Resolved file_size: {}", file_size);

    let final_path = Path::new(&file.dir).join(&file_name);
    let temp_path = Path::new(&file.dir).join(format!("{0}{1}", &file_name, &self.temp_suffix));

    // Write
    if let Some(path) = final_path.parent() {
      create_dir_all(path)?;
    }
    let mut temp_file = File::create_new(&temp_path)?;
    let mut downloaded: u64 = 0;
    let mut timestamp: u128 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
      let prev_downloaded = downloaded;
      let prev_time = timestamp;
      let chunk = item.or(Err(format!(
        "Failed to download file '{}'",
        final_path.to_str().unwrap()
      )))?;
      temp_file
        .write_all(&chunk)
        .or(Err("Error while writing to file".to_string()))?;
      downloaded = min(downloaded + (chunk.len() as u64), file_size);
      timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

      if let Some(progress_callback) = resolved_opts.on_download_progress {
        let progress: DownloaderFileProgress = DownloaderFileProgress {
          file_size,
          downloaded_bytes: downloaded,
          diff_bytes: downloaded - prev_downloaded,
          diff_time: timestamp - prev_time,
        };
        progress_callback(progress, chunk)
      }
    }

    rename(&temp_path, &final_path)?;

    Ok(())
  }
}
