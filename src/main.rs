mod download;
mod utils;
use bytes::Bytes;
use colored::Colorize;
use fern::colors::{Color, ColoredLevelConfig};
use log::info;
use std::{error::Error, io::stdout};
use utils::downloader::{
  DownloaderFile, DownloaderFileProgress, DownloaderFileTypes, DownloaderOpts,
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

  let file: DownloaderFile = DownloaderFile {
    url: "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json".into(),
    dir: "store".into(),
    name: None,
    size: None,
    verify: None,
    file_type: DownloaderFileTypes::Meta,
    retries: 0,
  };
  let opts: DownloaderOpts = DownloaderOpts {
    on_download_progress: Some(|current_progress: DownloaderFileProgress, _chunk: Bytes| {
      info!(
        "Downloaded {0}{1} bytes (speed: {2} bytes/second)",
        current_progress.downloaded_bytes,
        current_progress.file_size,
        if current_progress.diff_time > 0 {
          (current_progress.diff_bytes as u128 / current_progress.diff_time) * 1000
        } else {
          0
        }
      )
    }),
    on_download_finish: Some(|file: DownloaderFile| info!("Finished downloading {:?}", file.name)),
    overwrite: Some(false),
    get_content: Some(false),
    total_size: None,
  };

  let temp_suffix: String = ".temp".into();

  let dl = utils::downloader::Downloader::new(temp_suffix, None);
  dl.single_download(file, Some(&opts)).await?;

  info!("{:?}", opts);
  Ok(())
}
