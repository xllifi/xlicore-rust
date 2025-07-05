use futures::executor::block_on;

#[derive(Debug)]
pub struct Error {
  pub code: ErrorCode,
  pub message: String,
  pub verbose: Option<String>,
}

#[derive(Debug)]
pub enum ErrorCode {
  /// If supplying a string message and no verbose, use .into()
  Unknown,
  VerifyFailed,
  ReqwestError,
  IOError,
  HttpBadStatus,
}

impl Error {
  pub fn verify_err<S: Into<String>>(file_name: &String, verbose: S) -> Self {
    Error {
      code: ErrorCode::VerifyFailed,
      message: format!("Couldn't verify file {}", file_name),
      verbose: Some(verbose.into()),
    }
  }
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Error {
      code: ErrorCode::IOError,
      message: err.to_string(),
      verbose: None
    }
  }
}

impl From<String> for Error {
  fn from(string: String) -> Self {
    Error {
      code: ErrorCode::Unknown,
      message: string,
      verbose: None
    }
  }
}
impl From<&str> for Error {
  fn from(strslice: &str) -> Self {
    Error {
      code: ErrorCode::Unknown,
      message: strslice.into(),
      verbose: None
    }
  }
}

impl From<reqwest::Error> for Error {
  fn from(err: reqwest::Error) -> Self {
    Error {
      code: ErrorCode::ReqwestError,
      #[rustfmt::skip]
      message: format!(
        "An error occured while sending request to URL {}! HTTP status {}",
        if let Some(url)    = err.url()    { url.to_string()    } else { "[Unknown]".into() },
        if let Some(status) = err.status() { status.to_string() } else { "[Unknown]".into() }
      ),
      verbose: Some(err.to_string())
    }
  }
}

/// Assumes Response has a bad status (> 2XX)
impl From<reqwest::Response> for Error {
  fn from(response: reqwest::Response) -> Self {
    Error {
      code: ErrorCode::HttpBadStatus,
      message: format!(
        "HTTP request to {} failed with {}",
        response.url(),
        response.status(),
      ),
      verbose: Some(format!(
        "{}",
        if let Ok(json) = block_on(response.json::<serde_json::Value>()) {
          String::from("Response JSON:\n") + json.to_string().as_str() + "\n"
        } else {
          "".into()
        }
      ))
    }
  }
}