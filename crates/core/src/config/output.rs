use serde::{Deserialize, Serialize};

use crate::HashMap;

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
  #[serde(rename = "iife")]
  IIFE,
  #[serde(rename = "umd")]
  UMD,
  #[serde(rename = "system")]
  System,
  #[serde(rename = "amd")]
  AMD,
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

  pub fn as_multiple(&self) -> Vec<ModuleFormat> {
    match self {
      ModuleFormatConfig::Single(_) => vec![self.as_single()],
      ModuleFormatConfig::Multiple(module_formats) => module_formats.to_vec(),
    }
  }

  pub fn contains(&self, module_format: &ModuleFormat) -> bool {
    match self {
      ModuleFormatConfig::Single(format) => *format == *module_format,
      ModuleFormatConfig::Multiple(formats) => formats.contains(module_format),
    }
  }

  pub fn contains_esm(&self) -> bool {
    self.contains(&ModuleFormat::EsModule)
  }

  pub fn contains_cjs(&self) -> bool {
    self.contains(&ModuleFormat::CommonJs)
  }

  pub fn contains_umd(&self) -> bool {
    self.contains(&ModuleFormat::UMD)
  }

  pub fn contains_iife(&self) -> bool {
    self.contains(&ModuleFormat::IIFE)
  }

  pub fn contains_system(&self) -> bool {
    self.contains(&ModuleFormat::System)
  }

  pub fn contains_amd(&self) -> bool {
    self.contains(&ModuleFormat::AMD)
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
  /// external globals name, for example, if you set `external_globals: {"react": "React"}`,
  /// if you use `import * as React from 'react'`, you can access `React` from `window.React`
  /// NOTE: only works when `target_env` is `browser`, or `library` with format `iife` or `umd`
  pub external_globals: HashMap<String, String>,
  /// necessary for umd/iife format, if not set, default name '__farm_global__' will be used
  pub name: String,
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
      external_globals: HashMap::default(),
      name: "__farm_global__".to_string(),
    }
  }
}
