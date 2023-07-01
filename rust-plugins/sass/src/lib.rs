#![deny(clippy::all)]

use farmfe_core::{config::Config, module::ModuleType, plugin::Plugin};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{fs, regex::Regex};
use serde::{Deserialize, Serialize};

#[farm_plugin]
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FarmPluginSass {
  sass_options: String,
}

impl FarmPluginSass {
  pub fn new(_config: &Config, options: String) -> Self {
    Self {
      sass_options: options,
    }
  }
  
  // TODO support options
  // pub fn get_sass_options(&self, resolve_path: String) -> grass::Options {
  //   let options = serde_json::from_str(&self.sass_options).unwrap_or_default();
  // }
}

impl Plugin for FarmPluginSass {
  fn name(&self) -> &str {
    "FarmPluginSass"
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    let reg = Regex::new(r#"\.(sass|scss)$"#).unwrap();
    if reg.is_match(param.resolved_path) {
      let content = fs::read_file_utf8(param.resolved_path).unwrap();
      return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
        content,
        module_type: ModuleType::Custom(String::from("sass")),
      }));
    }
    Ok(None)
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if param.module_type == ModuleType::Custom(String::from("sass")) {
      let css = grass::from_string(
        param.content.to_owned(),
        &grass::Options::default())
        .map_err(|e| farmfe_core::error::CompilationError::TransformError {
          resolved_path: param.resolved_path.to_string(),
          msg: e.to_string(),
        })?;
      return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content: css,
        source_map: None,
        module_type: Some(farmfe_core::module::ModuleType::Css),
      }));
    }
    Ok(None)
  }
}




