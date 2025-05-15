use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricAllVersions {
  pub game: Vec<FabricGameVersion>,
  pub mappings: Vec<FabricYarnVersion>,
  pub intermediary: Vec<FabricIntermediaryVersion>,
  pub loader: Vec<FabricLoaderVersion>,
  pub installer: Vec<FabricInstallerVersion>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricGameVersion {
  /// Minecraft version
  pub version: String,
  /// Is version a release
  pub stable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricYarnVersion {
  /// Minecraft version
  #[serde(rename = "gameVersion")]
  pub game_version: String,
  /// Minecraft version and Yarn build number separator
  pub separator: String,
  /// Yarn build number
  pub build: u32,
  /// Package's maven name
  pub maven: String,
  /// Package's version
  pub version: String,
  /// Is latest release
  pub stable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricIntermediaryVersion {
  /// Package's maven name
  pub maven: String,
  /// Package's version
  pub version: String,
  /// https://github.com/FabricMC/fabric-meta/issues/5
  pub stable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricLoaderVersion {
  /// Major and minor versions separator
  pub separator: String,
  /// Minor version
  pub build: u32,
  /// Package's maven name
  pub maven: String,
  /// Package's version
  pub version: String,
  /// Recommended release
  pub stable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricInstallerVersion {
  /// Download URL
  url: String,
  /// Package's maven name
  maven: String,
  /// Package's version
  version: String,
  /// Recommended release
  stable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricVersion {
  loader: FabricLoaderVersion,
  intermediary: FabricIntermediaryVersion,
  #[serde(rename = "launcherMeta")]
  launcher_meta: FabricVersionLauncherMeta,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricVersionLauncherMeta {
  version: u32,
  min_java_version: u32,
  libraries: FabricVersionLibraries,
  #[serde(rename = "mainClass")]
  main_class: FabricVersionMainClass,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct FabricVersionLibraries {
  client: Vec<FabricVersionDownload>,
  common: Vec<FabricVersionDownload>,
  server: Vec<FabricVersionDownload>,
  development: Vec<FabricVersionDownload>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct FabricVersionMainClass {
  client: String,
  server: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricVersionDownload {
  name: String,
  url: String,
  md5: String,
  sha1: String,
  sha256: String,
  sha512: String,
  size: u64,
}
