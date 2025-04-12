use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftMetaVersionManifest {
  pub latest: MinecraftMetaVersionManifestLatest,
  pub versions: Vec<MinecraftMetaVersionManifestPackage>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftMetaVersionManifestLatest {
  pub release: String,
  pub snapshot: String
}
#[derive(Serialize, Deserialize, Debug)]
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
  pub compliance_level: u32
}