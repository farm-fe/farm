use serde::{Deserialize, Serialize};
use swc_ecma_ast::EsVersion;

use super::ScriptParserConfig;

use crate::{
  config::ConfigRegex,
  module::{ModuleId, ModuleType},
};

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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptConfig {
  pub target: EsVersion,
  pub parser: ScriptParserConfig,
  pub plugins: Vec<ScriptConfigPlugin>,
  pub decorators: ScriptDecoratorsConfig,
  pub native_top_level_await: bool,
  pub import_not_used_as_values: ImportNotUsedAsValues,
}

impl ScriptConfig {
  pub fn is_target_legacy(&self) -> bool {
    self.target == EsVersion::Es5 || self.target == EsVersion::Es3
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub enum ImportNotUsedAsValues {
  #[serde(rename = "remove")]
  #[default]
  Remove,
  #[serde(rename = "preserve")]
  Preserve,
  #[serde(untagged)]
  Rule(ImportNotUsedAsValuesRule),
}

impl ImportNotUsedAsValues {
  pub fn is_preserved(&self, module_id: &ModuleId) -> bool {
    match self {
      ImportNotUsedAsValues::Remove => false,
      ImportNotUsedAsValues::Preserve => true,
      ImportNotUsedAsValues::Rule(rule) => rule
        .preserve
        .iter()
        .any(|regex| regex.is_match(&module_id.to_string())),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ImportNotUsedAsValuesRule {
  pub preserve: Vec<ConfigRegex>,
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_script_import_not_used_as_values() {
    let script_config: &str = r#"
    {
      "importNotUsedAsValues": {
        "preserve": [
          "node_modules/test$"
        ]
      }
    }
    "#;

    let script_config: ScriptConfig = serde_json::from_str(script_config).unwrap();
    assert!(script_config
      .import_not_used_as_values
      .is_preserved(&ModuleId::from("node_modules/test")));
    assert!(!script_config
      .import_not_used_as_values
      .is_preserved(&ModuleId::from("node_modules/test1")));
  }
}
