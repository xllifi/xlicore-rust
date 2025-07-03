use crate::error::{Error, ErrorCode};

/// Get filename from the URL.
///
/// # Errors
/// - No "/" in the input string.
pub fn filename_from_url(url: &str) -> Result<String, Error> {
  let filename: String = url
    .split('/')
    .next_back()
    .ok_or(Error {
      code: ErrorCode::Unknown,
      message: format!("Filename could not be derived from URL {}!", url),
    })?
    .split('?') // Make sure no query parameters are in filename
    .next()
    .unwrap() // This should never panic. Unless cosmic ray flipping bits which is out of scope for this project.
    .into();
  Ok(filename)
}