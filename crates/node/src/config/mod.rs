use std::collections::HashMap;

use farmfe_core::config::Config;
use napi::JsObject;

/// User defined config from js side, can be transformed from or to native Compiler config automatically
#[napi(object)]
pub struct JsUserConfig {
  pub input: HashMap<String, String>,
  pub js_plugins: Vec<JsObject>,
  pub wasm_plugins: Vec<String>,
}

impl Into<Config> for JsUserConfig {
  fn into(self) -> Config {
    Config { input: self.input }
  }
}

pub enum JsObjectOrString {
  JsObject(JsObject),
  String(String),
}
