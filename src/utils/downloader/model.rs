use std::str::Bytes;

use reqwest::Client;

pub struct Downloader {
  /// Suffix that all partially downloaded files will have
  pub temp_suffix: String,
  pub default_opts: Option<DownloaderOpts>,
  /// Internal field, don't change
  pub reqwest_client: Client,
}

#[derive(Debug)]
pub struct DownloaderFile {
  pub url: String,
  pub dir: String,
  pub name: Option<String>,
  pub size: Option<u32>,
  pub verify: Option<DownloaderVerify>,
  pub file_type: DownloaderFileTypes,
  pub retries: i8,
}

#[derive(Debug)]
pub struct DownloaderVerify {
  pub hash: String,
  pub algorithm: DownloaderAlgorithm,
  pub retry_download: bool,
}

#[derive(Debug)]
pub struct DownloaderOpts {
  pub on_download_progress: fn(current_progress: DownloaderFileProgress, chunk: Bytes),
  // onDownloadFinish?: DownloaderCallbackOnFinish
  pub overwrite: Option<bool>,
  pub get_content: Option<bool>,
  pub total_size: Option<u32>,
}

#[derive(Debug)]
pub enum DownloaderFileTypes {
  Asset,
  Library,
  Java,
  Modpack,
  Meta,
  Loader,
  Game,
}

#[derive(Debug)]
pub enum DownloaderAlgorithm {
  Sha1,
  Sha256,
}

pub struct DownloaderFileProgress {
  pub timestamp: u64,
  pub total_bytes: u64,
  pub downloaded_bytes: u64,
  pub previous_diff_bytes: u64,
}