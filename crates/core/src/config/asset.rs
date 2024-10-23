use serde::{Deserialize, Serialize};

use super::TargetEnv;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetFormatMode {
  Node,
  Browser,
}

impl From<TargetEnv> for AssetFormatMode {
  fn from(value: TargetEnv) -> Self {
    if value.is_browser() {
      AssetFormatMode::Browser
    } else {
      AssetFormatMode::Node
    }
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AssetsConfig {
  pub include: Vec<String>,
  /// Used internally, this option will be not exposed to user.
  pub public_dir: Option<String>,
  // TODO: v2
  // for ssr mode, should specify asset path format, default from `output.targetEnv`
  // pub mode: Option<AssetFormatMode>,
}
