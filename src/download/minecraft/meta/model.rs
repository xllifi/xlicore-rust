use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Struct for https://piston-meta.mojang.com/mc/game/version_manifest_v2.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftMetaVersionManifest {
  pub latest: MinecraftMetaVersionManifestLatest,
  pub versions: Vec<MinecraftMetaVersionManifestPackage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftMetaVersionManifestLatest {
  pub release: String,
  pub snapshot: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftMetaVersionManifestPackage {
  pub id: String,
  #[serde(rename = "type")]
  pub release_type: String,
  pub url: String,
  pub time: String,
  #[serde(rename = "releaseTime")]
  pub release_time: String,
  pub sha1: String,
  #[serde(rename = "complianceLevel")]
  pub compliance_level: u32,
}

/// Structs for https://piston-meta.mojang.com/v1/packages/<hash>/<version>.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifest {
  pub arguments: MinecraftPackageManifestArguments,
  #[serde(rename = "assetIndex")]
  pub asset_index: MinecraftPackageManifestAssetIndex,
  pub assets: String,
  #[serde(rename = "complianceLevel")]
  pub compliance_level: u8,
  pub downloads: MinecraftPackageManifestDownloads,
  pub id: String,
  #[serde(rename = "javaVersion")]
  pub java_version: MinecraftPackageManifestJavaVersion,
  pub libraries: Vec<MinecraftPackageManifestLibrary>,
  pub logging: MinecraftPackageManifestLogging,
  #[serde(rename = "mainClass")]
  pub main_class: String,
  #[serde(rename = "minimumLauncherVersion")]
  pub minimum_launcher_version: u16,
  #[serde(rename = "releaseTime")]
  pub release_time: String,
  pub time: String,
  #[serde(rename = "type")]
  pub package_type: MinecraftPackageType
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestArguments {
  pub game: Vec<MinecraftPackageManifestArgumentTypes>,
  pub jvm: Vec<MinecraftPackageManifestArgumentTypes>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MinecraftPackageManifestArgumentTypes {
  Simple(String),
  Ruled(MinecraftPackageManifestArgumentRuled),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestArgumentRuled {
  pub rules: Vec<MinecraftPackageManifestRuleTypes>,
  pub value: MinecraftPackageManifestArgumentRuledValueTypes,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MinecraftPackageManifestArgumentRuledValueTypes {
  Single(String),
  Multiple(Vec<String>)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MinecraftPackageManifestRuleTypes {
  Os(MinecraftPackageManifestRuleOs),
  Features(MinecraftPackageManifestRuleFeatures),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestRuleOs {
  pub action: MinecraftPackageManifestRuleConditionAction,
  pub os: MinecraftPackageManifestRuleOsConditionValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestRuleFeatures {
  pub action: MinecraftPackageManifestRuleConditionAction,
  pub features: HashMap<String, bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MinecraftPackageManifestRuleConditionAction {
  #[serde(rename = "allow")]
  Allow,
  #[serde(rename = "disallow")]
  Disallow
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestRuleOsConditionValue {
  pub name: Option<MinecraftPackageManifestRuleConditionOsNames>,
  pub arch: Option<MinecraftPackageManifestRuleConditionArchNames>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MinecraftPackageManifestRuleConditionOsNames {
  #[serde(rename = "windows")]
  Windows,
  #[serde(rename = "linux")]
  Linux,
  #[serde(rename = "osx")]
  Macos
}

impl MinecraftPackageManifestRuleConditionOsNames {
  pub fn as_str(&self) -> &'static str {
    match self {
        MinecraftPackageManifestRuleConditionOsNames::Windows => "windows",
        MinecraftPackageManifestRuleConditionOsNames::Linux => "linux",
        MinecraftPackageManifestRuleConditionOsNames::Macos => "macos",
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// This might be wrong. I couldn't find any info regarding all possible values
pub enum MinecraftPackageManifestRuleConditionArchNames {
  #[serde(rename = "x86")]
  X86,
  #[serde(rename = "x64")]
  X64,
}

impl MinecraftPackageManifestRuleConditionArchNames {
  pub fn as_str(&self) -> &'static str {
    match self {
        MinecraftPackageManifestRuleConditionArchNames::X86 => "x86",
        MinecraftPackageManifestRuleConditionArchNames::X64 => "x64",
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestAssetIndex {
  pub id: String,
  pub sha1: String,
  pub size: u64,
  #[serde(rename = "totalSize")]
  pub total_size: u64,
  pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestDownloads {
  pub client: MinecraftPackageManifestDownloadsEntry,
  pub client_mappings: MinecraftPackageManifestDownloadsEntry,
  pub server: MinecraftPackageManifestDownloadsEntry,
  pub server_mappings: MinecraftPackageManifestDownloadsEntry,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestDownloadsEntry {
  pub sha1: String,
  pub size: u64,
  pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestJavaVersion {
  component: MinecraftPackageManifestJavaVersionComponents,
  #[serde(rename = "majorVersion")]
  major_version: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MinecraftPackageManifestJavaVersionComponents {
  #[serde(rename = "java-runtime-alpha")]
  JavaRuntimeAlpha,
  #[serde(rename = "java-runtime-beta")]
  JavaRuntimeBeta,
  #[serde(rename = "java-runtime-delta")]
  JavaRuntimeDelta,
  #[serde(rename = "java-runtime-gamma")]
  JavaRuntimeGamma,
  #[serde(rename = "java-runtime-gamma-snapshot")]
  JavaRuntimeGammaSnapshot,
  #[serde(rename = "jre-legacy")]
  JreLegacy,
  #[serde(rename = "minecraft-java-exe")]
  MinecraftJavaExe,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MinecraftPackageManifestLibrary {
  Simple(MinecraftPackageManifestLibrarySimple),
  Ruled(MinecraftPackageManifestLibraryRuled)
}
pub trait MinecraftPackageManifestLibraryBase {
  fn downloads(&self) -> &MinecraftPackageManifestLibraryDownloads;
  fn name(&self) -> &str;
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestLibrarySimple {
  pub downloads: MinecraftPackageManifestLibraryDownloads,
  pub name: String,
}
impl MinecraftPackageManifestLibraryBase for MinecraftPackageManifestLibrarySimple {
  fn downloads(&self) -> &MinecraftPackageManifestLibraryDownloads {&self.downloads}
  fn name(&self) -> &str {&self.name}
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestLibraryRuled {
  pub downloads: MinecraftPackageManifestLibraryDownloads,
  pub name: String,
  pub rules: Vec<MinecraftPackageManifestRuleOs>
}
impl MinecraftPackageManifestLibraryBase for MinecraftPackageManifestLibraryRuled {
  fn downloads(&self) -> &MinecraftPackageManifestLibraryDownloads {&self.downloads}
  fn name(&self) -> &str {&self.name}
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestLibraryDownloads {
  pub artifact: MinecraftPackageManifestLibraryDownloadsEntry,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestLibraryDownloadsEntry {
  pub path: String,
  pub sha1: String,
  pub size: u64,
  pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestLogging {
  pub client: MinecraftPackageManifestLoggingClient
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestLoggingClient {
  pub argument: String,
  pub file: MinecraftPackageManifestLoggingFile,
  #[serde(rename = "type")]
  pub logging_type: String
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftPackageManifestLoggingFile {
  pub id: String,
  pub sha1: String,
  pub size: u64,
  pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MinecraftPackageType {
  #[serde(rename = "release")]
  Release,
  #[serde(rename = "snapshot")]
  Snapshot
}