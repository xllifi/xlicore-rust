mod model;
use std::error::Error;

pub use model::*;

use crate::utils::downloader::{Downloader, DownloaderAlgorithm, DownloaderFile, DownloaderFileTypes, DownloaderOpts, DownloaderVerify};

pub async fn get_version_manifest(
  dl: &Downloader,
) -> Result<MinecraftMetaVersionManifest, Box<dyn Error>> {
  let mut file: DownloaderFile = DownloaderFile {
    url: "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json".into(),
    dir: "store".into(),
    name: None,
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
  let ret: MinecraftMetaVersionManifest =
    dl.single_download_get_json(&mut file, Some(&opts)).await?;
  Ok(ret)
}

fn get_package_info(
  package_version: String,
  version_manifest: MinecraftMetaVersionManifest,
) -> Result<MinecraftMetaVersionManifestPackage, String> {
  let mut iterator = version_manifest.versions.into_iter();
  match package_version.as_str() {
    "lr" | "latest_release" | "release" | "latest" => {
      let id = version_manifest.latest.release;
      iterator.find(|x| x.id == id)
    }
    "ls" | "latest_snapshot" | "snapshot" => {
      let id = version_manifest.latest.snapshot;
      iterator.find(|x| x.id == id)
    }
    _ => iterator.find(|x| x.id == package_version),
  }
  .ok_or(format!(
    "Version {} not found!",
    package_version
  ))
}

pub async fn get_package_manifest(
  dl: &Downloader,
  package_version: String,
  version_manifest: &MinecraftMetaVersionManifest,
) -> Result<MinecraftPackageManifest, Box<dyn Error>> {
  let package = get_package_info(package_version, version_manifest.clone())?;

  let mut file: DownloaderFile = DownloaderFile {
    url: package.url,
    dir: format!("store/{}", package.id),
    name: Some(format!("{}.json", package.id)),
    size: None,
    verify: Some(DownloaderVerify {
      hash: package.sha1,
      algorithm: DownloaderAlgorithm::Sha1
    }),
    file_type: DownloaderFileTypes::Meta,
    retries: 2,
  };
  let opts: DownloaderOpts = DownloaderOpts {
    on_download_progress: None,
    on_download_finish: None,
    overwrite: Some(false),
    total_size: None,
  };

  let ret: MinecraftPackageManifest =
    dl.single_download_get_json(&mut file, Some(&opts)).await?;
  Ok(ret)
}