use std::path::PathBuf;

use archiver::unarchive;
use shared::error::Error;

fn main() -> Result<(), Error> {
  let destination = PathBuf::from("./store/unarchive/");

  let file_tar = PathBuf::from("./store/archive.tar.gz");
  unarchive(&file_tar, &destination, false)?;

  let file_zip = PathBuf::from("./store/archive.zip");
  unarchive(&file_zip, &destination, true)?;

  Ok(())
}
