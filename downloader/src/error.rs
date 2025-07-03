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
  IOError,
  ChannelDisconnected,
  HttpBadStatus,
  VerifyFailed,
  MissingFileSize,
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

impl<T> From<SendError<T>> for Error {
  fn from(_: SendError<T>) -> Self {
    Error {
      code: ErrorCode::ChannelDisconnected,
      message: "Failed to send channel message".into(),
    }
  }
}

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
