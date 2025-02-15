use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{bool_or_obj::BoolOrObj, config_regex::ConfigRegex};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MinifyOptions {
  pub compress: BoolOrObj<Value>,
  pub mangle: BoolOrObj<Value>,
  pub include: Vec<ConfigRegex>,
  pub exclude: Vec<ConfigRegex>,
  pub mangle_exports: bool,
}

impl Default for MinifyOptions {
  fn default() -> Self {
    Self {
      compress: BoolOrObj::Bool(true),
      mangle: BoolOrObj::Bool(true),
      include: vec![],
      exclude: vec![ConfigRegex::new(".+\\.min\\.(js|css|html)$")],
      mangle_exports: true,
    }
  }
}
