use std::sync::mpsc::SendError;

use reqwest::Response;

#[derive(Debug)]
pub struct Error {
  pub code: ErrorCode,
  pub message: String,
}

#[derive(Debug)]
pub enum ErrorCode {
  Unknown,
  NoFiles,
  IOError,
  HttpBadStatus,
  VerifyFailed,
  SerdeError,
}

impl Error {
  pub fn unknown<S: Into<String>>(reason: S) -> Self {
    Error {
      code: ErrorCode::Unknown,
      message: reason.into(),
    }
  }
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Error {
      code: ErrorCode::IOError,
      message: err.to_string(),
    }
  }
}

/// Assumes Response has a bad status (> 2XX)
impl From<Response> for Error {
  fn from(response: Response) -> Self {
    Error {
      code: ErrorCode::HttpBadStatus,
      message: format!(
        "HTTP request to {} failed with {}",
        response.url(),
        response.status()
      ),
    }
  }
}


impl From<String> for Error {
  fn from(string: String) -> Self {
    Error {
      code: ErrorCode::Unknown,
      message: string
    }
  }
}
impl From<&str> for Error {
  fn from(strslice: &str) -> Self {
    Error {
      code: ErrorCode::Unknown,
      message: strslice.into()
    }
  }
}