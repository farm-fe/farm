use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Default)]
pub enum TargetEnv {
  #[serde(rename = "browser")]
  #[default]
  Browser,
  #[serde(rename = "node")]
  Node,
  #[serde(rename = "library")]
  Library,
  #[serde(untagged)]
  Custom(String),
}

impl TargetEnv {
  pub fn is_browser(&self) -> bool {
    matches!(self, TargetEnv::Browser)
  }

  pub fn is_node(&self) -> bool {
    matches!(self, TargetEnv::Node)
  }

  pub fn is_library(&self) -> bool {
    matches!(self, TargetEnv::Library)
  }
}

impl ToString for TargetEnv {
  fn to_string(&self) -> String {
    match self {
      TargetEnv::Browser => "browser".to_string(),
      TargetEnv::Node => "node".to_string(),
      TargetEnv::Library => "library".to_string(),
      TargetEnv::Custom(s) => s.clone(),
    }
  }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Default)]
pub enum ModuleFormat {
  #[serde(rename = "esm")]
  #[default]
  EsModule,
  #[serde(rename = "cjs")]
  CommonJs,
}

// pub struct ModuleFormatConfig {
//   pub format: ModuleFormat,
//   pub output_dir: Option<String>,
//   pub output_
// }

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
  pub show_file_size: bool,
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
      show_file_size: true,
    }
  }
}
