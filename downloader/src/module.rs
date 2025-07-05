use std::{fmt::Display, path::PathBuf, sync::mpsc::Sender};

use reqwest::Client;
use serde::Serialize;
use uuid::Uuid;

use crate::hasher::Algorithm;

pub struct Downloader {
  /// Suffix that all partially downloaded files will have
  pub temp_suffix: String,
  /// Internal field, don't change
  pub reqwest_client: Client,
  /// For progress reporting. See https://doc.rust-lang.org/rust-by-example/std_misc/channels.html
  pub channel_sender: Sender<ChannelMessage>,
  /// Should downloaded files overwrite existing.  
  /// Note that files will be overwritten anyway if requested file's hash is different from existing.
  pub overwrite: bool,
}

/// A struct for internal use only.
#[derive(Clone, Copy, Serialize, Debug)]
pub struct RequestData {
  pub id: Uuid,
  pub action: Action,
}

#[derive(Clone, Serialize)]
pub struct File {
  pub url: String,
  pub dir: String,
  pub name: Option<String>,
  pub size: u64,
  pub verify: Option<Verify>,
}
/// A struct for internal use only.
#[derive(Clone, Debug)]
pub struct PreppedFile {
  pub url: String,
  pub final_path: PathBuf,
  pub temp_path: PathBuf,
  pub name: String,
  pub size: u64,
  pub verify: Option<Verify>,
}

#[derive(Clone, Serialize, Debug)]
pub struct Verify {
  pub hash: String,
  pub algorithm: Algorithm,
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "action", content = "data")]
pub enum ChannelMessage {
  Start {
    data: RequestData,
    progress_enabled: bool,
  },
  Progress {
    data: RequestData,
    file_size_bytes: u64,
    downloaded_bytes: u64,
  },
  Verify {
    data: RequestData,
    total_files: u32,
    verified_files: u32,
  },
  Finish {
    data: RequestData,
  }
}

#[derive(Clone, Copy, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Action {
  Download,
  Verify
}

impl Display for Action {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Action::Download => write!(f, "download"),
      Action::Verify => write!(f, "verify"),
    }
  }
}