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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum ModuleFormatConfig {
  Single(ModuleFormat),
  Multiple(Vec<ModuleFormat>),
}

impl Default for ModuleFormatConfig {
  fn default() -> Self {
    Self::Single(ModuleFormat::default())
  }
}

impl ModuleFormatConfig {
  pub fn as_single(&self) -> ModuleFormat {
    match self {
      ModuleFormatConfig::Single(module_format) => *module_format,
      ModuleFormatConfig::Multiple(_) => {
        unreachable!("Multiple output.format is only allowed when output.target_env is library")
      }
    }
  }

  pub fn contains_esm(&self) -> bool {
    match self {
      ModuleFormatConfig::Single(module_format) => *module_format == ModuleFormat::EsModule,
      ModuleFormatConfig::Multiple(module_formats) => {
        module_formats.contains(&ModuleFormat::EsModule)
      }
    }
  }

  pub fn contains_cjs(&self) -> bool {
    match self {
      ModuleFormatConfig::Single(module_format) => *module_format == ModuleFormat::CommonJs,
      ModuleFormatConfig::Multiple(module_formats) => {
        module_formats.contains(&ModuleFormat::CommonJs)
      }
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
pub enum LibraryBundleType {
  #[serde(rename = "single-bundle")]
  #[default]
  SingleBundle,
  /// TODO set default to MultipleBundle when multiple bundle is fully supported
  #[serde(rename = "multiple-bundle")]
  MultipleBundle,
  #[serde(rename = "bundle-less")]
  BundleLess,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct OutputConfig {
  pub path: String,
  pub public_path: String,
  pub entry_filename: String,
  pub filename: String,
  pub assets_filename: String,
  pub target_env: TargetEnv,
  pub format: ModuleFormatConfig,
  pub show_file_size: bool,
  pub library_bundle_type: LibraryBundleType,
  pub ascii_only: bool,
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
      format: ModuleFormatConfig::default(),
      show_file_size: true,
      library_bundle_type: Default::default(),
      ascii_only: false,
    }
  }
}
