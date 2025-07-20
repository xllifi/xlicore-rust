use std::{fmt::Display, path::PathBuf};
use serde::Serialize;
use uuid::Uuid;

use crate::hasher::Algorithm;

/// A struct for internal use only.
#[derive(Clone, Copy, Serialize, Debug)]
pub struct RequestData {
  pub id: Uuid,
  pub action: Action,
}

impl RequestData {
  pub fn new(action: Action) -> Self {
    Self {
      id: Uuid::new_v4(),
      action,
    }
  }
}

#[derive(Clone, Serialize)]
pub struct File {
  pub url: String,
  pub dir: String,
  pub name: Option<String>,
  pub size: u64,
  pub verify: Option<Verify>,
  pub check_etag: bool,
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
  /// true: (don't overwrite) both last and current etags exist and are the same  
  /// 
  /// false: (do overwrite) either one of last or current etags don't exist or they differ
  pub etags_match: bool,
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