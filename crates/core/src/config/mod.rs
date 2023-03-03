use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{EsConfig, TsConfig};

pub const FARM_GLOBAL_THIS: &str = "__farm_global_this__";
pub const FARM_MODULE_SYSTEM: &str = "__farm_module_system__";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
  pub input: HashMap<String, String>,
  pub output: OutputConfig,
  pub root: String,
  pub mode: Mode,
  pub resolve: ResolveConfig,
  pub external: Vec<String>,
  pub runtime: RuntimeConfig,
  pub script: ScriptConfig,
  pub partial_bundling: PartialBundlingConfig,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      input: HashMap::from([("index".to_string(), "./index.html".to_string())]),
      root: std::env::current_dir()
        .unwrap()
        .to_string_lossy()
        .to_string(),
      output: OutputConfig::default(),
      mode: Mode::Development,
      resolve: ResolveConfig::default(),
      external: vec![],
      runtime: Default::default(),
      script: Default::default(),
      partial_bundling: PartialBundlingConfig::default(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OutputConfig {
  pub path: String,
  pub public_path: String,
  pub filename: String,
}

impl Default for OutputConfig {
  fn default() -> Self {
    Self {
      filename: "[resourceName].[contentHash].[ext]".to_string(),
      public_path: "/".to_string(),
      path: "dist".to_string(),
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Mode {
  #[serde(rename = "development")]
  Development,
  #[serde(rename = "production")]
  Production,
}

impl Default for Mode {
  fn default() -> Self {
    Self::Development
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ScriptConfig {
  pub target: EsVersion,
  pub parser: ScriptParserConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ScriptParserConfig {
  pub es_config: EsConfig,
  pub ts_config: TsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct ResolveConfig {
  pub alias: HashMap<String, String>,
  pub main_fields: Vec<String>,
  pub main_files: Vec<String>,
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
      main_files: vec![String::from("index")],
      extensions: vec![
        String::from("tsx"),
        String::from("ts"),
        String::from("jsx"),
        String::from("mjs"),
        String::from("js"),
        String::from("json"),
        String::from("html"),
        String::from("css"),
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
#[serde(rename_all = "camelCase", default)]
pub struct RuntimeConfig {
  /// the compiled runtime file path, a runtime is required for script module loading, executing and hot module updating.
  pub path: String,
  /// the runtime plugins
  pub plugins: Vec<String>,
  /// swc helpers path
  pub swc_helpers_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PartialBundlingModuleBucketsConfig {
  pub name: String,
  /// Regex vec to match the modules in the module bucket
  pub test: Vec<String>,
  /// If true, this bucket will always contains all modules matches by the test field, this may bring duplicate modules when you have multiple entries and use dynamic import.
  /// be careful to use it
  pub isolate: bool,
}

impl Default for PartialBundlingModuleBucketsConfig {
  fn default() -> Self {
    Self {
      name: "".to_string(),
      test: vec![],
      isolate: false,
    }
  }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PartialBundlingConfig {
  /// custom module buckets
  pub module_buckets: Vec<PartialBundlingModuleBucketsConfig>,
}
