use downloader::{hasher::Algorithm, module::*, Downloader};
use log::info;
use std::{sync::mpsc, thread};

#[tokio::main]
async fn main() {
  colog::basic_builder()
    .filter_level(log::LevelFilter::Debug)
    .init();
  let (tx, rx) = mpsc::channel();

  let mut files: Vec<File> = vec![
    File {
      url: "https://piston-data.mojang.com/v1/objects/eb1e1eb47cb740012fc82eacc394859463684132/server.txt".into(),
      dir: "./store/".into(),
      name: None,
      size: 8_186_232,
      verify: Some(Verify {
        hash: "eb1e1eb47cb740012fc82eacc394859463684132".into(),
        algorithm: Algorithm::Sha1,
      }),
      check_etag: true,
    },
    File {
      url: "https://piston-data.mojang.com/v1/objects/8d83af626cae1865deaf55fbf96934be4886fd45/client.txt".into(),
      dir: "./store/".into(),
      name: None,
      size: 10_988_969,
      verify: Some(Verify {
        hash: "8d83af626cae1865deaf55fbf96934be4886fd45".into(),
        algorithm: Algorithm::Sha1,
      }),
      check_etag: true,
    },
    File {
      url: "https://piston-data.mojang.com/v1/objects/05e4b48fbc01f0385adb74bcff9751d34552486c/server.jar".into(),
      dir: "./store/".into(),
      name: None,
      size: 57_556_704,
      verify: Some(Verify {
        hash: "05e4b48fbc01f0385adb74bcff9751d34552486c".into(),
        algorithm: Algorithm::Sha1,
      }),
      check_etag: true,
    },
  ];

  thread::spawn(move || {
    loop {
      match rx.recv() {
        Ok(msg) => match msg {
          ChannelMessage::Start {
            data,
            progress_enabled,
          } => {
            info!(
              "[{}] {} started! Progress {} be reported.",
              data.id,
              match data.action {
                Action::Download => "Download",
                Action::Verify => "Verify process",
              },
              if progress_enabled { "will" } else { "won't" }
            );
          }
          ChannelMessage::Progress {
            data,
            file_size_bytes,
            downloaded_bytes,
          } => {
            info!(
              "[{}] {} progress: {:.2}% ({downloaded_bytes}/{file_size_bytes})",
              data.id,
              match data.action {
                Action::Download => "Download",
                Action::Verify => "Verify process",
              },
              (downloaded_bytes as f64 / file_size_bytes as f64) * 100.0
            );
          }
          ChannelMessage::Verify {
            data,
            total_files,
            verified_files,
          } => {
            info!(
              "[{}] Verifying files ({verified_files}/{total_files})",
              data.id
            );
          }
          ChannelMessage::Finish { data } => {
            info!(
              "[{}] {} finished!",
              data.id,
              match data.action {
                Action::Download => "Download",
                Action::Verify => "Verify process",
              },
            );
          }
        },
        Err(_) => {
          info!("Channel closed abruptly!");
          break;
        }
      };
    }
  });

  let dl = Downloader::new(".temp".into(), tx, false);
  match dl.verify(&mut files).await {
    Ok(_) => {}
    Err(e) => {
      println!("{:?}", e)
    }
  };
}
