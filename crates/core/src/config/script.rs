use serde::{Deserialize, Serialize};
use swc_ecma_ast::EsVersion;

use super::ScriptParserConfig;

use crate::{config::ConfigRegex, module::ModuleType};

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
#[serde(rename_all = "camelCase")]
pub enum DecoratorVersion {
  #[default]
  #[serde(rename = "2021-12")]
  V202112,

  #[serde(rename = "2022-03")]
  V202203,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptDecoratorsConfig {
  pub legacy_decorator: bool,
  pub decorator_metadata: bool,
  pub decorator_version: Option<DecoratorVersion>,
  pub includes: Vec<ConfigRegex>,
  pub excludes: Vec<ConfigRegex>,
}

impl Default for ScriptDecoratorsConfig {
  fn default() -> Self {
    Self {
      legacy_decorator: true,
      decorator_metadata: false,
      decorator_version: None,
      includes: vec![],
      excludes: vec![ConfigRegex::new("node_modules/")],
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptConfig {
  pub target: EsVersion,
  pub parser: ScriptParserConfig,
  pub plugins: Vec<ScriptConfigPlugin>,
  pub decorators: ScriptDecoratorsConfig,
  pub native_top_level_await: bool,
  pub import_not_used_as_values: String,
}

impl ScriptConfig {
  pub fn is_target_legacy(&self) -> bool {
    self.target == EsVersion::Es5 || self.target == EsVersion::Es3
  }
}

impl Default for ScriptConfig {
  fn default() -> Self {
    Self {
      target: Default::default(),
      parser: Default::default(),
      plugins: Default::default(),
      decorators: Default::default(),
      native_top_level_await: Default::default(),
      import_not_used_as_values: "preserve".to_string(),
    }
  }
}
