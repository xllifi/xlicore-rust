mod download;
mod utils;
use bytes::Bytes;
use colored::Colorize;
use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, info};
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

  let mut file: DownloaderFile = DownloaderFile {
    url: "https://mirror.haku.host/100MB.test".into(),
    dir: "store".into(),
    name: None,
    size: None,
    verify: None,
    file_type: DownloaderFileTypes::Meta,
    retries: 2,
  };
  let opts: DownloaderOpts = DownloaderOpts {
    on_download_progress: Some(|current_progress: DownloaderFileProgress, _chunk: Bytes| {
      info!(
        "Downloaded {0}/{1} bytes",
        current_progress.downloaded_bytes,
        current_progress.file_size,
      )
    }),
    on_download_finish: Some(|file: DownloaderFile| info!("Finished downloading {:?}", file.name)),
    overwrite: Some(true),
    get_content: Some(false),
    total_size: None,
  };

  let temp_suffix: String = ".temp".into();

  let dl = utils::downloader::Downloader::new(temp_suffix, None);
  dl.single_download(&mut file, Some(&opts)).await?;

  debug!("{:?}", opts);
  debug!("{:?}", file);
  Ok(())
}
