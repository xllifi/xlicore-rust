mod download;
mod utils;
use colored::Colorize;
use download::{fabric::meta::get_fabric_launcher_meta, minecraft::{libraries::download_libraries, meta::{get_package_manifest, get_version_manifest, MinecraftPackageManifestLibrary}}};
use fern::colors::{Color, ColoredLevelConfig};
use log::info;
use std::{error::Error, io::stdout};
use utils::downloader::
  Downloader
;

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
  let package_manifest = get_package_manifest(&dl, "latest".into(), &version_manifest).await?;

  for library in &package_manifest.libraries {
    match library {
      MinecraftPackageManifestLibrary::Ruled(val) => {
        info!("{:?}", val.downloads.artifact.url)
      },
      MinecraftPackageManifestLibrary::Simple(val) => {
        info!("{:?}", val.downloads.artifact.url)
      }
    }
  }

  let fabric_meta = get_fabric_launcher_meta(&dl, &package_manifest, None).await?;

  info!("{:?}", fabric_meta);

  download_libraries(&dl, &package_manifest).await?;

  Ok(())
}
