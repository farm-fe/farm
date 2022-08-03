use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "camelCase", default)]
pub struct Config {
  pub input: HashMap<String, String>,
  pub root: String,
  pub mode: Mode,
  pub resolve: ResolveConfig,
  pub external: Vec<String>,
  pub runtime: RuntimeConfig,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      input: HashMap::new(),
      root: std::env::current_dir()
        .unwrap()
        .to_string_lossy()
        .to_string(),
      mode: Mode::Development,
      resolve: ResolveConfig::default(),
      external: vec![],
      runtime: Default::default(),
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub enum Mode {
  Development,
  Production,
}

impl Default for Mode {
  fn default() -> Self {
    Self::Development
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "camelCase", default)]
pub struct ResolveConfig {
  pub alias: HashMap<String, String>,
  pub main_fields: Vec<String>,
  pub extensions: Vec<String>,
  pub conditions: Vec<String>,
  pub symlinks: bool,
}

impl Default for ResolveConfig {
  fn default() -> Self {
    Self {
      alias: HashMap::new(),
      main_fields: vec![
        String::from("browser"),
        String::from("module"),
        String::from("main"),
      ],
      extensions: vec![
        String::from("tsx"),
        String::from("ts"),
        String::from("jsx"),
        String::from("mjs"),
        String::from("js"),
        String::from("json"),
      ],
      conditions: vec![
        String::from("import"),
        String::from("require"),
        String::from("browser"),
        String::from("development"),
        String::from("production"),
        String::from("default"),
      ],
      symlinks: true,
    }
  }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename = "camelCase", default)]
pub struct RuntimeConfig {
  /// the module system file path
  pub module_system_file_path: String,
  /// the module system's plugin's file path
  pub module_system_plugins: Vec<String>,
}
