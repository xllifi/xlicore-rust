mod download;
mod utils;
use std::{error::Error, str::Bytes};
use log::{debug, info};
use utils::downloader::{DownloaderFile, DownloaderFileProgress, DownloaderFileTypes, DownloaderOpts};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  colog::init();
  let file: DownloaderFile = DownloaderFile {
    url: "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json".into(),
    dir: "store".into(),
    name: None,
    size: None,
    verify: None,
    file_type: DownloaderFileTypes::Meta,
    retries: 0,
  };
  let opts: DownloaderOpts = DownloaderOpts {
    on_download_progress: dl_progress,
    overwrite: Some(false),
    get_content: Some(false),
    total_size: None,
  };

  let temp_suffix: String = ".temp".into();

  let dl = utils::downloader::Downloader::new(temp_suffix, None);
  dl.single_download(file, Some(&opts)).await?;

  debug!("{:?}", opts);
  Ok(())
}

fn dl_progress(current_progress: DownloaderFileProgress, chunk: Bytes) {
  debug!(
    "Downloaded {0}{1} bytes ({2} more than last)",
    current_progress.downloaded_bytes,
    current_progress.total_bytes,
    current_progress.previous_diff_bytes
  )
}