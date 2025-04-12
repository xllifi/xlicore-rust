mod model;
pub use model::*;

use std::cmp::min;
use log::{debug, info};
use reqwest::Client;
use std::{error::Error, fs::{self, create_dir_all, rename, File}, io::Write, path::Path};
use url::Url;
use futures_util::StreamExt;

impl Downloader {
  pub fn new(temp_suffix: String, default_opts: Option<DownloaderOpts>) -> Self {
    Downloader {
      temp_suffix,
      default_opts,
      reqwest_client: Client::new(),
    }
  }

  pub async fn single_download(&self, file: DownloaderFile, opts: Option<&DownloaderOpts>) -> Result<(), Box<dyn Error>> {
    debug!("Requested to download file {:?} with opts {:?}", file.url, opts);

    // Verify options
    let url = Url::parse(&file.url)?;
    let file_name: String;
    if file.name.is_none() {
      file_name = url.path_segments().unwrap().last().unwrap().into();
    } else {
      file_name = file.name.unwrap();
    }
    debug!("Resolved file_name: {:?}", file_name);
    
    // Execute download

    // Prepare request
    let resp = self.reqwest_client
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
    create_dir_all(final_path.parent().unwrap())?;
    let mut temp_file = File::create_new(&temp_path)?;
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();
    while let Some(item) = stream.next().await {
      // TODO: progress callbacks
      let chunk = item.or(Err(format!("Failed to download file '{}'", final_path.to_str().unwrap())))?;
      temp_file.write_all(&chunk).or(Err(format!("Error while writing to file")))?;
      let new = min(downloaded + (chunk.len() as u64), file_size);
      downloaded = new;
      info!("Download progress: {0}/{1} bytes", downloaded, file_size);
    }

    rename(&temp_path, &final_path)?;

    Ok(())
  }
}
