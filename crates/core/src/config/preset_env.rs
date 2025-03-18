use serde::{Deserialize, Serialize};

use super::config_regex::ConfigRegex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PresetEnvConfig {
  Bool(bool),
  Obj(PresetEnvConfigObj),
}

impl PresetEnvConfig {
  pub fn enabled(&self) -> bool {
    match self {
      PresetEnvConfig::Bool(b) => *b,
      PresetEnvConfig::Obj(_) => true,
    }
  }
}

impl Default for PresetEnvConfig {
  fn default() -> Self {
    PresetEnvConfig::Bool(true)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PresetEnvConfigObj {
  /// The value will be deserialized in FarmPluginPolyfill
  pub options: Box<serde_json::Value>,
  pub include: Vec<ConfigRegex>,
  pub exclude: Vec<ConfigRegex>,
  pub assumptions: Box<serde_json::Value>,
}

impl Default for PresetEnvConfigObj {
  fn default() -> Self {
    Self {
      options: Box::new(serde_json::Value::Object(Default::default())),
      include: vec![ConfigRegex::new_farm_runtime()],
      exclude: vec![ConfigRegex::new_node_modules()],
      assumptions: Box::new(serde_json::Value::Object(Default::default())),
    }
  }
}
