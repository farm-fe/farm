use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{bool_or_obj::BoolOrObj, config_regex::ConfigRegex};

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum MinifyMode {
  #[serde(rename = "minify-resource-pot")]
  ResourcePot,
  #[default]
  #[serde(rename = "minify-module")]
  Module,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MinifyOptions {
  pub compress: BoolOrObj<Value>,
  pub mangle: BoolOrObj<Value>,
  pub include: Vec<ConfigRegex>,
  pub exclude: Vec<ConfigRegex>,
  pub mode: MinifyMode,
  pub module_decls: bool,
}

impl Default for MinifyOptions {
  fn default() -> Self {
    Self {
      compress: BoolOrObj::Bool(true),
      mangle: BoolOrObj::Bool(true),
      include: vec![],
      exclude: vec![ConfigRegex::new(".+\\.min\\.(js|css|html)$")],
      mode: MinifyMode::Module,
      module_decls: false,
    }
  }
}

impl From<Value> for MinifyOptions {
  fn from(val: Value) -> Self {
    serde_json::from_value(val)
      .expect("failed parser MinifyOptions, please ensure your options is correct")
  }
}

impl From<&BoolOrObj<Value>> for Option<MinifyOptions> {
  fn from(value: &BoolOrObj<Value>) -> Self {
    match value {
      BoolOrObj::Bool(v) => {
        if *v {
          Some(Default::default())
        } else {
          None
        }
      }

      BoolOrObj::Obj(v) => Some(MinifyOptions::from(v.clone())),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::config::minify::MinifyMode;

  #[test]
  fn deserialize_minify_mode() {
    use super::MinifyMode;
    use serde_json::json;

    let mode = json!("minify-resource-pot");
    let mode: MinifyMode = serde_json::from_value(mode).unwrap();
    assert!(matches!(mode, MinifyMode::ResourcePot));

    let mode = json!("minify-module");
    let mode: MinifyMode = serde_json::from_value(mode).unwrap();
    assert!(matches!(mode, MinifyMode::Module));
  }

  #[test]
  fn deserialize_minify_options() {
    use super::MinifyOptions;
    use crate::config::bool_or_obj::BoolOrObj;
    use serde_json::json;

    let options = json!({
      // "compress": true,
      // "mangle": true,
      // "include": ["@farmfe/runtime"],
      "exclude": ["node_modules/"],
      // "mode": "minify-resource-pot",
    });
    let minify =
      BoolOrObj::Obj(options).map(|val| serde_json::from_value::<MinifyOptions>(val).unwrap());

    assert!(matches!(minify, BoolOrObj::Obj(MinifyOptions { .. })));

    let options = json!({
      "compress": true,
      "mangle": true,
      "include": ["@farmfe/runtime"],
      "exclude": ["node_modules/"],
      "mode": "minify-resource-pot",
      "moduleDecls": false,
    });
    let minify =
      BoolOrObj::Obj(options).map(|val| serde_json::from_value::<MinifyOptions>(val).unwrap());

    assert!(matches!(
      minify,
      BoolOrObj::Obj(MinifyOptions {
        compress: BoolOrObj::Bool(true),
        mangle: BoolOrObj::Bool(true),
        mode: MinifyMode::ResourcePot,
        module_decls: false,
        ..
      })
    ));
  }
}
