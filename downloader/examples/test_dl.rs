use downloader::{hasher::Algorithm, module::*};
use log::info;
use std::{sync::mpsc, thread};

#[tokio::main]
async fn main() {
  let (tx, rx) = mpsc::channel();
  
  let files: Vec<DownloaderFile> = vec![
    DownloaderFile {
      url: "https://piston-data.mojang.com/v1/objects/eb1e1eb47cb740012fc82eacc394859463684132/server.txt".into(),
      dir: "./store/".into(),
      name: None,
      size: 8_186_232,
      verify: Some(DownloaderVerify {
        hash: "eb1e1eb47cb740012fc82eacc394859463684132".into(),
        algorithm: Algorithm::Sha1,
      }),
    },
    DownloaderFile {
      url: "https://piston-data.mojang.com/v1/objects/8d83af626cae1865deaf55fbf96934be4886fd45/client.txt".into(),
      dir: "./store/".into(),
      name: None,
      size: 10_988_969,
      verify: Some(DownloaderVerify {
        hash: "8d83af626cae1865deaf55fbf96934be4886fd45".into(),
        algorithm: Algorithm::Sha1,
      }),
    },
    DownloaderFile {
      url: "https://piston-data.mojang.com/v1/objects/05e4b48fbc01f0385adb74bcff9751d34552486c/server.jar".into(),
      dir: "./store/".into(),
      name: None,
      size: 57_556_704,
      verify: Some(DownloaderVerify {
        hash: "05e4b48fbc01f0385adb74bcff9751d34552486c".into(),
        algorithm: Algorithm::Sha1,
      }),
    },
  ];
  let req: DownloaderRequest = DownloaderRequest {
    request_type: RequestType::Game,
    retries: 2,
    overwrite: true,
    channel_sender: tx.clone(),
    files,
  };

  thread::spawn(move || {
    loop {
      match rx.recv() {
        Ok(msg) => match msg {
          DownloaderChannelMessage::Start {
            progress_enabled,
            request_type,
          } => {
            info!(
              "Download of type {:?} started! Progress {} be reported.",
              request_type,
              if progress_enabled { "will" } else { "won't" }
            );
          }
          DownloaderChannelMessage::Progress {
            file_size_bytes,
            downloaded_bytes,
          } => {
            info!(
              "Download progress: {:.2}% ({downloaded_bytes}/{file_size_bytes})",
              (downloaded_bytes as f64 / file_size_bytes as f64) * 100.0
            );
          }
          DownloaderChannelMessage::Verify {
            total_files,
            verified_files,
          } => {
            info!("Verifying files ({verified_files}/{total_files})");
          }
          DownloaderChannelMessage::Finish {
            success,
            failed_files,
          } => {
            if success {
              info!("All done!");
            } else {
              if let Some(failed_files) = failed_files {
                log::error!(
                  "Failed to download {} file(s): [{}]",
                  failed_files.iter().len(),
                  failed_files.join(", ")
                )
              }
            }
            break;
          }
        },
        Err(_) => {
          info!("Channel closed abruptly!");
          break;
        },
      };
    }
  });

  let dl = Downloader::new(".temp".into());
  match dl.download(req).await {
    Ok(_) => {}
    Err(e) => {
      println!("{:?}", e)
    }
  };
}
