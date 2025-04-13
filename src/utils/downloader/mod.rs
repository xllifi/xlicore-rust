mod model;
pub use model::*;

use futures_util::StreamExt;
use log::{debug, error};
use reqwest::Client;
use std::{
  cmp::min,
  fs::{exists, remove_file},
  io::stdout,
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
    file: &mut DownloaderFile,
    opts: Option<&DownloaderOpts>,
  ) -> Result<(), String> {
    let og_retries = file.retries.clone();
    loop {
      debug!("Trying to download file {}", file.url);
      stdout().flush().expect("Failed to flush stdout");
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

  async fn exec_download(
    &self,
    file: &mut DownloaderFile,
    opts: Option<&DownloaderOpts>,
  ) -> Result<(), Box<dyn Error>> {
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
    let temp_path_exists: bool = exists(&temp_path)?;
    let final_path_exists: bool = exists(&final_path)?;
    if resolved_opts.overwrite.unwrap_or(false) {
      if temp_path_exists { remove_file(&temp_path)?; }
      if final_path_exists { remove_file(&final_path)?; }
    }/* else if temp_path_exists || final_path_exists {
      TODO: add verify & overwrite logic
    }*/
    let mut temp_file = File::create_new(&temp_path)?;

    // Execute download
    let resp = self
      .reqwest_client
      .get(&file.url)
      .send()
      .await
      .or(Err(format!("Failed to GET {}", &file.url)))?;

    let file_size = resp.content_length().or(file.size).ok_or(format!(
      "Failed to get file size from request or file meta for {}",
      &file.url
    ))?;
    // TODO^ make it NOT error but just disable progress
    debug!("Resolved file_size: {}", file_size);

    // Write from stream
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
      let chunk = item.or(Err(format!(
        "Failed to download file '{}'",
        final_path.to_str().unwrap()
      )))?;
      temp_file
        .write_all(&chunk)
        .or(Err("Error while writing to file".to_string()))?;
      downloaded = min(downloaded + (chunk.len() as u64), file_size);

      if let Some(progress_callback) = resolved_opts.on_download_progress {
        let progress: DownloaderFileProgress = DownloaderFileProgress {
          file_size,
          downloaded_bytes: downloaded,
        };
        progress_callback(progress, chunk)
      }
    }

    rename(&temp_path, &final_path)?;

    Ok(())
  }
}
