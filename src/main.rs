mod download;
mod utils;
use bytes::Bytes;
use colored::Colorize;
use download::minecraft::meta::{get_package_version, get_version_manifest, MinecraftPackageManifestLibrary};
use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, info};
use std::{error::Error, io::stdout};
use utils::downloader::{
  Downloader, DownloaderAlgorithm, DownloaderFile, DownloaderFileProgress, DownloaderFileTypes, DownloaderOpts, DownloaderVerify
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let colors: ColoredLevelConfig = ColoredLevelConfig::new()
    .debug(Color::BrightBlack)
    .info(Color::Green)
    .warn(Color::Yellow)
    .error(Color::Red);

  fern::Dispatch::new()
    .format(move |out, message, record| {
      out.finish(format_args!(
        "{} | {} | {} > {}",
        Colorize::bright_black(chrono::Local::now().format("%H:%M:%S").to_string().as_str()),
        format!("{:^5}", colors.color(record.level())),
        Colorize::cyan(record.target()),
        message
      ));
    })
    .level(log::LevelFilter::Debug)
    .chain(stdout())
    .apply()?;

  let dl = Downloader::new(".temp".into(), None);
  let version_manifest = get_version_manifest(&dl).await?;
  let package_version = get_package_version(&dl, "latest".into(), version_manifest).await?;

  for library in package_version.libraries {
    match library {
      MinecraftPackageManifestLibrary::Ruled(val) => {
        println!("{:?}", val.downloads.artifact.url)
      },
      MinecraftPackageManifestLibrary::Simple(val) => {
        println!("{:?}", val.downloads.artifact.url)
      }
    }
      
  }

  Ok(())
}
