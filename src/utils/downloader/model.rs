use bytes::Bytes;
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

#[derive(Debug, Clone, Default)]
pub struct DownloaderOpts {
  pub on_download_progress: Option<fn(current_progress: DownloaderFileProgress, chunk: Bytes)>,
  pub on_download_finish: Option<fn(file: DownloaderFile)>,
  pub overwrite: Option<bool>,
  pub get_content: Option<bool>,
  pub total_size: Option<u32>,
}

impl DownloaderOpts {
  #[rustfmt::skip]
  pub fn merge(&self, other: &DownloaderOpts) -> Self {
    Self {
      on_download_progress: self.on_download_progress.or(other.on_download_progress),
        on_download_finish: self.on_download_finish.or(other.on_download_finish),
                 overwrite: self.overwrite.or(other.overwrite),
               get_content: self.get_content.or(other.get_content),
                total_size: self.total_size.or(other.total_size)
    }
  }
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
  pub file_size: u64,
  pub downloaded_bytes: u64,
  pub diff_bytes: u64,
  pub diff_time: u128,
}
