use minify::MinifyOptions;
use regex::Regex;
use serde::{Deserialize, Serialize};
use swc_css_prefixer::options::Targets;

use swc_ecma_parser::{EsSyntax as EsConfig, TsSyntax as TsConfig};
use tree_shaking::TreeShakingConfig;

use self::{
  bool_or_obj::BoolOrObj, comments::CommentsConfig, config_regex::ConfigRegex, html::HtmlConfig,
  partial_bundling::PartialBundlingConfig, preset_env::PresetEnvConfig, script::ScriptConfig,
};

use crate::HashMap;

pub const FARM_MODULE_SYSTEM: &str = "m";
// transformed from dynamic import, e.g `import('./xxx')`
pub const FARM_DYNAMIC_REQUIRE: &str = "farmDynamicRequire";
// transformed from static import, e.g `import xxx from './xxx'`
pub const FARM_REQUIRE: &str = "farmRequire";
pub const FARM_MODULE: &str = "module";
pub const FARM_MODULE_EXPORT: &str = "exports";

pub mod asset;
pub mod bool_or_obj;
pub mod comments;
pub mod config_regex;
pub mod css;
pub mod custom;
pub mod external;
pub mod html;
pub mod minify;
mod output;
pub mod partial_bundling;
pub mod persistent_cache;
pub mod preset_env;
pub mod script;
pub mod tree_shaking;

use asset::AssetsConfig;

pub use output::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasItem {
  pub find: StringOrRegex,
  pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrRegex {
  String(String),
  #[serde(with = "serde_regex")]
  Regex(Regex),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
  pub input: HashMap<String, String>,
  pub output: Box<OutputConfig>,
  pub root: String,
  pub mode: Mode,
  pub resolve: Box<ResolveConfig>,
  pub external: Vec<ConfigRegex>,
  pub define: HashMap<String, serde_json::Value>,
  pub runtime: Box<RuntimeConfig>,
  pub script: Box<ScriptConfig>,
  pub assets: Box<AssetsConfig>,
  pub css: Box<CssConfig>,
  pub html: Box<HtmlConfig>,
  pub sourcemap: Box<SourcemapConfig>,
  pub partial_bundling: Box<PartialBundlingConfig>,
  pub lazy_compilation: bool,
  pub core_lib_path: Option<String>,
  pub tree_shaking: Box<BoolOrObj<TreeShakingConfig>>,
  pub minify: Box<BoolOrObj<MinifyOptions>>,
  pub preset_env: Box<PresetEnvConfig>,
  /// whether to record the compilation flow stats, default is false.
  pub record: bool,
  pub progress: bool,
  pub persistent_cache: Box<persistent_cache::PersistentCacheConfig>,
  /// concatenateModules is used to concatenate modules into a single file. And it will not be wrapped by the runtime.
  /// Note: all runtime options will be ignored if concatenateModules is true. Which means you can't use `runtime.plugins`, `path` and so on.
  pub concatenate_modules: bool,
  /// comments config for script, css and html
  pub comments: Box<CommentsConfig>,
  /// preserved for future compatibility usage when there are more config options
  pub custom: Box<HashMap<String, String>>,
}

impl Default for Config {
  fn default() -> Self {
    let root = std::env::current_dir()
      .unwrap()
      .to_string_lossy()
      .to_string();

    Self {
      input: HashMap::from_iter([("index".to_string(), "./index.html".to_string())]),
      root: root.clone(),
      output: Default::default(),
      mode: Mode::Development,
      resolve: Default::default(),
      define: HashMap::default(),
      external: Default::default(),
      runtime: Default::default(),
      script: Default::default(),
      css: Default::default(),
      html: Box::default(),
      assets: Default::default(),
      sourcemap: Default::default(),
      partial_bundling: Default::default(),
      lazy_compilation: true,
      core_lib_path: None,
      tree_shaking: Box::new(BoolOrObj::Bool(true)),
      minify: Box::new(BoolOrObj::Bool(true)),
      preset_env: Box::<PresetEnvConfig>::default(),
      record: false,
      progress: true,
      persistent_cache: Box::<persistent_cache::PersistentCacheConfig>::new(
        // the config file path will be set after the Config is initialized
        persistent_cache::PersistentCacheConfig::get_default_config(&root),
      ),
      concatenate_modules: false,
      comments: Box::default(),
      custom: Box::<HashMap<String, String>>::default(),
    }
  }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Mode {
  #[serde(rename = "development")]
  Development,
  #[serde(rename = "production")]
  Production,
}

impl Mode {
  pub fn is_dev(&self) -> bool {
    matches!(self, Mode::Development)
  }

  pub fn is_prod(&self) -> bool {
    matches!(self, Mode::Production)
  }
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
  pub alias: Vec<AliasItem>,
  pub main_fields: Vec<String>,
  pub main_files: Vec<String>,
  pub extensions: Vec<String>,
  pub conditions: Vec<String>,
  pub symlinks: bool,
  pub strict_exports: bool,
  pub auto_external_failed_resolve: bool,
}

impl Default for ResolveConfig {
  fn default() -> Self {
    Self {
      alias: vec![],
      main_fields: vec![
        String::from("browser"),
        String::from("module"),
        String::from("main"),
        String::from("jsnext:main"),
        String::from("jsnext"),
      ],
      main_files: vec![String::from("index")],
      extensions: vec![
        String::from("tsx"),
        String::from("ts"),
        String::from("mts"),
        String::from("cts"),
        String::from("jsx"),
        String::from("mjs"),
        String::from("js"),
        String::from("cjs"),
        String::from("json"),
        String::from("html"),
        String::from("css"),
      ],
      conditions: vec![
        String::from("development"),
        String::from("production"),
        String::from("module"),
      ],
      symlinks: true,
      strict_exports: false,
      auto_external_failed_resolve: false,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourcemapConfig {
  /// Generate inline sourcemap instead of a separate file for mutable resources.
  #[serde(rename = "inline")]
  Inline,
  /// Generate sourcemap for all resources.
  /// By default, sourcemap is generated only for resources that are mutable.
  #[serde(rename = "all")]
  All,
  #[serde(rename = "all-inline")]
  AllInline,
  #[serde(untagged)]
  Bool(bool),
}

impl Default for SourcemapConfig {
  fn default() -> Self {
    Self::Bool(true)
  }
}

impl SourcemapConfig {
  pub fn enabled(&self, immutable: bool) -> bool {
    match self {
      Self::Bool(b) => *b && !immutable,
      Self::Inline => !immutable,
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

  pub fn is_false(&self) -> bool {
    match self {
      Self::Bool(b) => !*b,
      _ => false,
    }
  }
}

mod tests {

  #[test]
  fn test_deserialize() {
    use super::SourcemapConfig;
    let config: SourcemapConfig = serde_json::from_str("true").expect("failed to parse");

    assert!(matches!(config, SourcemapConfig::Bool(true)));

    let config: SourcemapConfig = serde_json::from_str("false").expect("failed to parse");

    assert!(matches!(config, SourcemapConfig::Bool(false)));

    let config: SourcemapConfig = serde_json::from_str("\"all-inline\"").expect("failed to parse");

    assert!(matches!(config, SourcemapConfig::AllInline));

    let config: SourcemapConfig = serde_json::from_str("\"inline\"").expect("failed to parse");

    assert!(matches!(config, SourcemapConfig::Inline));

    let config: SourcemapConfig = serde_json::from_str("\"all\"").expect("failed to parse");

    assert!(matches!(config, SourcemapConfig::All));
  }

  #[test]
  fn target_env() {
    use super::TargetEnv;
    let env = TargetEnv::Browser;
    assert!(env.is_browser());
    assert!(!env.is_node());
    assert!(!env.is_library());

    let env = TargetEnv::Node;
    assert!(env.is_node());
    assert!(!env.is_browser());
    assert!(!env.is_library());

    let env = TargetEnv::Library;
    assert!(env.is_library());
    assert!(!env.is_node());
    assert!(!env.is_browser());
  }
}
