use bytes::Bytes;
use reqwest::Client;
use std::{error::Error, fmt};
use sha1::Sha1;
use sha2::Sha256;

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
  pub size: Option<u64>,
  pub verify: Option<DownloaderVerify>,
  #[allow(dead_code)] // provided for frontends
  pub file_type: DownloaderFileTypes,
  pub retries: u8,
}

#[derive(Debug)]
pub struct DownloaderVerify {
  pub hash: String,
  pub algorithm: DownloaderAlgorithm,
}

#[derive(Debug, Clone, Default)]
pub struct DownloaderOpts {
  pub on_download_progress: Option<fn(current_progress: DownloaderFileProgress, chunk: Bytes)>,
  pub on_download_finish: Option<fn(file: DownloaderFile)>,
  pub overwrite: Option<bool>,
  pub get_content: Option<bool>,
  pub total_size: Option<u64>,
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

pub struct DownloaderFileProgress {
  pub file_size: u64,
  pub downloaded_bytes: u64,
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
  VerifyFailed,
  BrokenStream,
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