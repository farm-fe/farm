use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::Result,
  module::ModuleType,
  plugin::{
    Plugin, PluginLoadHookParam, PluginLoadHookResult, PluginResolveHookParam,
    PluginResolveHookResult, PluginTransformHookResult,
  },
};
use farmfe_macro_plugin::farm_plugin;
/// SassPlugin is used to support compiling scss files to css modules
#[farm_plugin]
pub struct FarmPluginSass {}

impl FarmPluginSass {
  fn new(config: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginSass {
  fn name(&self) -> &str {
    "FarmPluginSass"
  }

  fn resolve(
    &self,
    _param: &PluginResolveHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginResolveHookResult>> {
    Ok(Some(PluginResolveHookResult {
      id: String::from("resolve from FarmSassPlugin"),
      ..Default::default()
    }))
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginLoadHookResult>> {
    println!("load param {:?}", param);

    Ok(Some(PluginLoadHookResult {
      content: String::from("hello"),
      module_type: farmfe_core::module::ModuleType::Custom(String::from("scss")),
    }))
  }

  fn transform(
    &self,
    _param: &farmfe_core::plugin::PluginTransformHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    Ok(Some(PluginTransformHookResult {
      content: String::from("transformed data"),
      ..Default::default()
    }))
  }
}
