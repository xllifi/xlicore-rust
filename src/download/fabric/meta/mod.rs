mod model;
use std::error::Error;

pub use model::*;
use reqwest::Client;

use crate::{download::minecraft::meta::MinecraftPackageManifest, utils::downloader::{Downloader, DownloaderFile, DownloaderFileTypes, DownloaderOpts}};

async fn test_fabric_meta_servers(servers: Vec<&str>) -> Result<&str, String> {
  let client = Client::new();
  for server in servers {
    let resp = match client.head(server).send().await {
      Ok(val) => val,
      Err(_) => continue,
    };
    if resp.status() == 200 {
      return Ok(server);
    }
  }

  Err("All servers failed!".into())
}

pub async fn get_fabric_launcher_meta(
  dl: &Downloader,
  package_manifest: &MinecraftPackageManifest,
  fabric_version: Option<String>
) -> Result<FabricVersion, Box<dyn Error>> {
  let server = test_fabric_meta_servers(vec!["https://meta.fabricmc.net", "https://meta2.fabricmc.net", "https://meta3.fabricmc.net"]).await?;

  let all_fabric_versions: Vec<FabricLoaderVersion> = reqwest::get(format!("{server}/v2/versions/loader")).await?.json().await?;

  let fabric_version = match fabric_version {
    Some(val) => val,
    None => all_fabric_versions[0].version.clone(),
  };
  drop(all_fabric_versions);
  
  let mut file: DownloaderFile = DownloaderFile {
    url: format!("{server}/v2/versions/loader/{}/{fabric_version}", package_manifest.id),
    dir: format!("store/{}", package_manifest.id),
    name: Some(format!("fabric-{fabric_version}.json")),
    size: None,
    verify: None,
    file_type: DownloaderFileTypes::Meta,
    retries: 2,
  };
  let opts: DownloaderOpts = DownloaderOpts {
    on_download_progress: None,
    on_download_finish: None,
    overwrite: Some(true),
    total_size: None,
  };

  let ret: FabricVersion =
    dl.single_download_get_json(&mut file, Some(&opts)).await?;
  Ok(ret)
}
