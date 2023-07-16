use farmfe_core::{
  config::Config,
  error::CompilationError,
  module::ModuleType,
  plugin::{Plugin, PluginLoadHookResult},
  serde_json,
};
use farmfe_toolkit::fs;

pub fn add(left: usize, right: usize) -> usize {
  left + right
}

pub struct FarmPluginJson {}

impl FarmPluginJson {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

fn match_json_file(file_name: &str) -> bool {
  file_name.ends_with(".json")
}

impl Plugin for FarmPluginJson {
  fn name(&self) -> &str {
    "FarmPluginJson"
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if match_json_file(param.resolved_path) {
      return Ok(Some(PluginLoadHookResult {
        content: fs::read_file_utf8(param.resolved_path)?,
        module_type: ModuleType::Custom(String::from("json")),
      }));
    }

    Ok(None)
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if matches!(param.module_type, ModuleType::Custom(ref suffix) if suffix == "json") {
      let json = serde_json::from_str::<serde_json::Value>(&param.content).map_err(|e| {
        CompilationError::TransformError {
          resolved_path: param.resolved_path.to_string(),
          msg: format!("JSON parse error: {:?}", e),
        }
      })?;

      let js = format!("module.exports = {}", json);

      Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content: js,
        module_type: Some(ModuleType::Js),
        source_map: None,
      }))
    } else {
      Ok(None)
    }
  }
}
