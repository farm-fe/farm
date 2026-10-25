use farmfe_core::{
  config::Config,
  parking_lot::RwLock,
  plugin::Plugin,
  regex::Regex,
  serde_json::{self, Value},
};
use farmfe_toolkit::lazy_static::lazy_static;
use farmfe_toolkit::swc_ecma_visit::VisitMutWith;

mod environment;

#[cfg(test)]
mod tests;

// Default supported static assets: png, jpg, jpeg, gif, svg, webp, mp4, webm, wav, mp3, wma, m4a, aac, ico, ttf, woff, woff2
lazy_static! {
  static ref DEFAULT_STATIC_ASSETS: Vec<&'static str> = vec![
    "png", "jpg", "jpeg", "gif", "svg", "webp", "mp4", "webm", "wav", "mp3", "wma", "m4a", "aac",
    "ico", "ttf", "woff", "woff2", "txt"
  ];
}

const PLUGIN_NAME: &str = "FarmPluginDefine";
const REGEX_PREFIX: &str = "$__farm_regex:";
const DEFAULT_DEFINE_PROCESS_ENV: &str = "FARM_PROCESS_ENV";

pub struct FarmPluginDefine {
  /// Sort define by key len desc
  sorted_define: RwLock<Vec<(String, Value)>>,
  environment: RwLock<environment::EnvironmentDefines>,
}

impl FarmPluginDefine {
  pub fn new(_: &Config) -> Self {
    Self {
      sorted_define: RwLock::new(vec![]),
      environment: RwLock::new(Default::default()),
    }
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

  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    let (environment, automatic_keys) = environment::prepare(config)?;
    *self.environment.write() = environment;
    let define = config.define.clone();
    let mut sorted_define = define
      .into_iter()
      .filter(|(key, _)| !automatic_keys.contains(key))
      .collect::<Vec<_>>();
    sorted_define.sort_by_key(|b| std::cmp::Reverse(b.0.len()));

    let mut self_sorted_define = self.sorted_define.write();
    self_sorted_define.clear();
    // make sure internal defines are processed at last
    let mut delayed_define = vec![];

    for item in sorted_define {
      if item.0 == *DEFAULT_DEFINE_PROCESS_ENV {
        delayed_define.push(item);
      } else {
        self_sorted_define.push(item);
      }
    }

    for d in delayed_define {
      self_sorted_define.push(d);
    }

    Ok(Some(()))
  }

  fn process_module(
    &self,
    param: &mut farmfe_core::plugin::PluginProcessModuleHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let environment = self.environment.read();
    if !param.module_type.is_script() || environment.is_empty() {
      return Ok(None);
    }
    let globals = context.meta.get_globals(param.module_id);
    farmfe_core::swc_common::GLOBALS.set(globals.value(), || {
      let script = param.meta.as_script_mut();
      let mut visitor = environment::EnvironmentVisitor::new(
        &environment,
        farmfe_core::swc_common::Mark::from_u32(script.unresolved_mark),
      );
      script.ast.visit_mut_with(&mut visitor);
    });
    Ok(Some(()))
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    let define = self.sorted_define.read();

    if !define.is_empty() {
      let mut content = String::new();

      for (key, value) in &*define {
        let value = match value {
          serde_json::Value::Null => "null".to_string(),
          serde_json::Value::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
          serde_json::Value::Number(num) => num.to_string(),
          serde_json::Value::String(str) => str.to_string(),
          serde_json::Value::Array(arr) => serde_json::to_string(arr).unwrap(),
          serde_json::Value::Object(obj) => serde_json::to_string(obj).unwrap(),
        };
        if let Some(reg) = key.strip_prefix(REGEX_PREFIX) {
          let regex = Regex::new(reg).unwrap();
          if content.is_empty() {
            content = regex.replace_all(&param.content, &value).to_string();
          } else {
            content = regex.replace_all(&content, &value).to_string();
          }
        } else {
          if content.is_empty() {
            content = param.content.replace(key, &value);
          } else {
            content = content.replace(key, &value);
          }
        };
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
