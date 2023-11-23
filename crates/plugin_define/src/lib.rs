#![feature(path_file_prefix)]

use farmfe_core::{
  config::Config,

  // plugin::{constants::PLUGIN_BUILD_STAGE_META_RESOLVE_KIND, Plugin, ResolveKind},
  plugin::Plugin,
  serde_json,
};
use farmfe_toolkit::lazy_static::lazy_static;

// Default supported static assets: png, jpg, jpeg, gif, svg, webp, mp4, webm, wav, mp3, wma, m4a, aac, ico, ttf, woff, woff2
lazy_static! {
  static ref DEFAULT_STATIC_ASSETS: Vec<&'static str> = vec![
    "png", "jpg", "jpeg", "gif", "svg", "webp", "mp4", "webm", "wav", "mp3", "wma", "m4a", "aac",
    "ico", "ttf", "woff", "woff2", "txt"
  ];
}

const PLUGIN_NAME: &str = "FarmPluginDefine";

pub struct FarmPluginDefine {}

impl FarmPluginDefine {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginDefine {
  fn name(&self) -> &str {
    PLUGIN_NAME
  }
  /// Make sure this plugin is executed last
  fn priority(&self) -> i32 {
    -99
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    let define = &context.config.define;

    if !define.is_empty() {
      let mut content = String::new();

      for (key, value) in define {
        let value = match value {
          serde_json::Value::Null => "null".to_string(),
          serde_json::Value::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
          serde_json::Value::Number(num) => num.to_string(),
          serde_json::Value::String(str) => format!("{:?}", str),
          serde_json::Value::Array(arr) => serde_json::to_string(arr).unwrap(),
          serde_json::Value::Object(obj) => serde_json::to_string(obj).unwrap(),
        };
        // reduce a string allocation
        if content.is_empty() {
          content = param.content.replace(key, &value);
        } else {
          content = content.replace(key, &value);
        }
      }

      return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content,
        // TODO support source map
        ..Default::default()
      }));
    }

    Ok(None)
  }
}
