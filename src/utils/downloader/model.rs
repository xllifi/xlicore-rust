use bytes::Bytes;
use reqwest::Client;
use std::{error::Error, fmt::{self, Debug}, sync::Arc, time::Duration};
use sha1::Sha1;
use sha2::Sha256;

pub struct Downloader {
  /// Suffix that all partially downloaded files will have
  pub temp_suffix: String,
  pub default_opts: Option<DownloaderOpts>,
  /// Internal field, don't change
  pub reqwest_client: Client,
}

#[derive(Debug, Clone)]
pub struct DownloaderFile {
  pub url: String,
  pub dir: String,
  pub name: Option<String>,
  pub size: Option<u64>,
  pub verify: Option<DownloaderVerify>,
  #[allow(dead_code)] // provided for frontends
  pub file_type: DownloaderFileTypes,
  pub retries: u8,
}

#[derive(Debug, Clone)]
pub struct DownloaderVerify {
  pub hash: String,
  pub algorithm: DownloaderAlgorithm,
}

#[derive(Clone, Default)]
pub struct DownloaderOpts {
  pub on_download_progress: Option<DownloaderOptsProgressCallback>,
  pub on_download_finish: Option<DownloaderOptsFinishCallback>,
  pub overwrite: Option<bool>,
  pub total_size: Option<u64>,
}

pub type DownloaderOptsProgressCallback = Arc<dyn Fn(DownloaderFileProgress, Bytes, &DownloaderFile, &DownloaderFileLastProgress) + Send + Sync>;
pub type DownloaderOptsFinishCallback = Arc<dyn Fn(&DownloaderFile, &DownloaderFileLastProgress) + Send + Sync>;

impl DownloaderOpts {
  #[rustfmt::skip]
  /// Some() `self` fields overwrite Some() `other` fields
  pub fn merge(&self, other: &DownloaderOpts) -> Self {
    Self {
      on_download_progress: self.on_download_progress.clone().or(other.on_download_progress.clone()),
        on_download_finish: self.on_download_finish.clone().or(other.on_download_finish.clone()),
                 overwrite: self.overwrite.or(other.overwrite),
                total_size: self.total_size.or(other.total_size)
    }
  }
}

// Custom debug impl for ignoring callbacks
impl Debug for DownloaderOpts {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DownloaderOpts")
      .field("overwrite", &self.overwrite)
      .field("total_size", &self.total_size)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub enum DownloaderFileTypes {
  Asset,
  Library,
  Java,
  Modpack,
  Meta,
  Loader,
  Game,
}

pub struct DownloaderFileProgress {
  pub file_size: u64,
  pub downloaded_bytes: u64,
  #[allow(dead_code)] // provided for frontends
  /// If progress is working and not broken
  pub ok: bool,
}
#[derive(Default)]
pub struct DownloaderFileLastProgress {
  pub downloaded_bytes: u64,
  pub timestamp: Duration
}

#[derive(Debug)]
pub struct DownloaderError {
  pub cause: DownloaderErrorCauses,
  pub details: String,
}

impl Error for DownloaderError {}

impl fmt::Display for DownloaderError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} (Enum {})", self.details, self.cause)
  }
}

#[derive(Debug)]
pub enum DownloaderErrorCauses {
  /// Verification hashes mismatch
  VerifyFailed,
  /// Downloader stream broke
  BrokenStream,
  /// HTTP request failed
  HttpFailed,
}

impl fmt::Display for DownloaderErrorCauses {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

pub enum Hasher {
  Sha1(Sha1),
  Sha256(Sha256),
}

#[derive(Debug, Clone, Copy)]
pub enum DownloaderAlgorithm {
  Sha1,
  Sha256,
}