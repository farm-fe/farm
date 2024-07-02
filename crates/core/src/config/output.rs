use serde::{Deserialize, Serialize};

use super::{ModuleFormat, TargetEnv};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct OutputConfig {
  pub path: String,
  pub public_path: String,
  pub entry_filename: String,
  pub filename: String,
  pub assets_filename: String,
  pub target_env: TargetEnv,
  pub format: ModuleFormat,
}

impl Default for OutputConfig {
  fn default() -> Self {
    Self {
      entry_filename: "[entryName].[ext]".to_string(),
      // [resourceName].[contentHash].[ext]
      filename: "[resourceName].[ext]".to_string(),
      // [resourceName].[contentHash].[ext]
      assets_filename: "[resourceName].[ext]".to_string(),
      public_path: "/".to_string(),
      path: "dist".to_string(),
      target_env: TargetEnv::default(),
      format: ModuleFormat::default(),
    }
  }
}
