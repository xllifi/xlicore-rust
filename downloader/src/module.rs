use std::{path::PathBuf, sync::mpsc::Sender};

use reqwest::Client;
use serde::Serialize;

use crate::hasher::Algorithm;

pub struct Downloader {
  /// Suffix that all partially downloaded files will have
  pub temp_suffix: String,
  /// Internal field, don't change
  pub reqwest_client: Client,
}

#[derive(Clone)]
pub struct DownloaderRequest {
  pub request_type: RequestType,
  pub retries: u8,
  /// Should downloaded files overwrite existing
  /// 
  /// Note that files will be overwritten anyway if requested file's hash is different from existing.
  pub overwrite: bool,
  /// Channel's sender. See https://doc.rust-lang.org/rust-by-example/std_misc/channels.html
  pub channel_sender: Sender<DownloaderChannelMessage>,
  pub files: Vec<DownloaderFile>,
}
/// A struct for internal use only.
#[derive(Clone)]
pub struct PreppedRequest {
  pub request_type: RequestType,
  /// Should downloaded files overwrite existing
  /// 
  /// Note that files will be overwritten anyway if requested file's hash is different from existing.
  pub overwrite: bool,
}
#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum RequestType {
  Asset,
  Library,
  Java,
  Modpack,
  Meta,
  Loader,
  Game,
}

#[derive(Clone, Serialize)]
pub struct DownloaderFile {
  pub url: String,
  pub dir: String,
  pub name: Option<String>,
  pub size: u64,
  pub verify: Option<DownloaderVerify>,
}
/// A struct for internal use only.
#[derive(Clone, Debug)]
pub struct PreppedFile {
  pub url: String,
  pub final_path: PathBuf,
  pub temp_path: PathBuf,
  pub name: String,
  pub size: u64,
  pub verify: Option<DownloaderVerify>,
}

#[derive(Clone, Serialize, Debug)]
pub struct DownloaderVerify {
  pub hash: String,
  pub algorithm: Algorithm,
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "action", content = "data")]
pub enum DownloaderChannelMessage {
  Start {
    progress_enabled: bool,
    request_type: RequestType,
  },
  Progress {
    file_size_bytes: u64,
    downloaded_bytes: u64,
  },
  Verify {
    total_files: u32,
    verified_files: u32,
  },
  Finish {
    success: bool,
    failed_files: Option<Vec<String>>,
  }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressType {
  Download,
  Verify
}

#[derive(Clone, Debug)]
pub enum InternalMessage {
  RecalcSize,
  Finish,
}