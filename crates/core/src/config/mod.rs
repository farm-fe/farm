use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use swc_css_prefixer::options::Targets;
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{EsConfig, TsConfig};

use crate::module::ModuleType;

use self::{
  config_regex::ConfigRegex, html::HtmlConfig, partial_bundling::PartialBundlingConfig,
  preset_env::PresetEnvConfig,
};

pub const FARM_GLOBAL_THIS: &str = "(globalThis || window || global || self)[__farm_namespace__]";
pub const FARM_MODULE_SYSTEM: &str = "__farm_module_system__";
pub const FARM_NAMESPACE: &str = "__farm_namespace__";

pub mod config_regex;
pub mod html;
pub mod partial_bundling;
pub mod preset_env;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
  pub input: HashMap<String, String>,
  pub output: OutputConfig,
  pub root: String,
  pub mode: Mode,
  pub resolve: ResolveConfig,
  pub external: Vec<ConfigRegex>,
  pub define: HashMap<String, String>,
  pub runtime: RuntimeConfig,
  pub script: ScriptConfig,
  pub assets: AssetsConfig,
  pub css: CssConfig,
  pub html: Box<HtmlConfig>,
  pub sourcemap: SourcemapConfig,
  pub partial_bundling: PartialBundlingConfig,
  pub lazy_compilation: bool,
  pub core_lib_path: Option<String>,
  pub tree_shaking: bool,
  // TODO: support minify options
  pub minify: bool,
  // TODO: support preset env options
  pub preset_env: Box<PresetEnvConfig>,
  pub record: bool,
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
      define: HashMap::new(),
      external: vec![],
      runtime: Default::default(),
      script: Default::default(),
      css: Default::default(),
      html: Box::default(),
      assets: Default::default(),
      sourcemap: Default::default(),
      partial_bundling: PartialBundlingConfig::default(),
      lazy_compilation: true,
      core_lib_path: None,
      tree_shaking: true,
      minify: true,
      preset_env: Box::<PresetEnvConfig>::default(),
      record: false
    }
  }
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
  pub format: ModuleFormat,
}

impl Default for OutputConfig {
  fn default() -> Self {
    Self {
      entry_filename: "[entryName].[ext]".to_string(),
      /// [resourceName].[contentHash].[ext]
      filename: "[resourceName].[ext]".to_string(),
      /// [resourceName].[contentHash].[ext]
      assets_filename: "[resourceName].[ext]".to_string(),
      public_path: "/".to_string(),
      path: "dist".to_string(),
      target_env: TargetEnv::default(),
      format: ModuleFormat::default(),
    }
  }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Default)]
pub enum TargetEnv {
  #[serde(rename = "browser")]
  #[default]
  Browser,
  #[serde(rename = "node")]
  Node,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Default)]
pub enum ModuleFormat {
  #[serde(rename = "esm")]
  #[default]
  EsModule,
  #[serde(rename = "cjs")]
  CommonJs,
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

impl ToString for Mode {
  fn to_string(&self) -> String {
    match self {
      Mode::Development => "development".to_string(),
      Mode::Production => "production".to_string(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptConfigPluginFilters {
  pub resolved_paths: Vec<ConfigRegex>,
  pub module_types: Vec<ModuleType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptConfigPlugin {
  pub name: String,
  pub options: serde_json::Value,
  pub filters: ScriptConfigPluginFilters,
}

impl Default for ScriptConfigPlugin {
  fn default() -> Self {
    Self {
      name: String::new(),
      options: serde_json::Value::Object(serde_json::Map::new()),
      filters: ScriptConfigPluginFilters::default(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptConfig {
  pub target: EsVersion,
  pub parser: ScriptParserConfig,
  pub plugins: Vec<ScriptConfigPlugin>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct CssModulesConfig {
  /// The paths regex to match css modules
  pub paths: Vec<String>,
  pub indent_name: String,
}

impl Default for CssModulesConfig {
  fn default() -> Self {
    Self {
      paths: vec![String::from("\\.module\\.(css|less|sass|scss)$")],
      indent_name: String::from("[name]-[hash]"),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CssPrefixerConfig {
  #[serde(skip_serializing)]
  pub targets: Option<Targets>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct CssConfig {
  pub modules: Option<CssModulesConfig>,
  pub prefixer: Option<CssPrefixerConfig>,
}

impl Default for CssConfig {
  fn default() -> Self {
    Self {
      modules: Some(Default::default()),
      prefixer: Some(Default::default()),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
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
  pub strict_exports: bool,
}

impl Default for ResolveConfig {
  fn default() -> Self {
    Self {
      alias: HashMap::new(),
      main_fields: vec![
        String::from("exports"),
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
      strict_exports: false,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RuntimeConfig {
  /// the absolute path of the runtime entry, a runtime is required for script module loading, executing and hot module updating.
  pub path: String,
  /// the runtime plugins
  pub plugins: Vec<String>,
  /// swc helpers path
  pub swc_helpers_path: String,
  /// namespace for the runtime
  pub namespace: String,
}

impl Default for RuntimeConfig {
  fn default() -> Self {
    Self {
      path: String::from(""),
      plugins: vec![],
      swc_helpers_path: String::from(""),
      namespace: String::from("__farm_default_namespace__"),
    }
  }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AssetsConfig {
  pub include: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SourcemapConfig {
  Bool(bool),
  /// Generate inline sourcemap instead of a separate file for mutable resources.
  #[serde(rename = "inline")]
  Inline,
  /// Generate sourcemap for all resources.
  /// By default, sourcemap is generated only for resources that are mutable.
  #[serde(rename = "all")]
  All,
  #[serde(rename = "all-inline")]
  AllInline,
}

impl Default for SourcemapConfig {
  fn default() -> Self {
    Self::Bool(true)
  }
}

impl SourcemapConfig {
  pub fn enabled(&self) -> bool {
    match self {
      Self::Bool(b) => *b,
      _ => true,
    }
  }
  pub fn is_inline(&self) -> bool {
    match self {
      Self::Bool(_) => false,
      Self::Inline => true,
      Self::All => false,
      Self::AllInline => true,
    }
  }

  pub fn is_all(&self) -> bool {
    match self {
      Self::Bool(_) => false,
      Self::Inline => false,
      Self::All => true,
      Self::AllInline => true,
    }
  }
}
