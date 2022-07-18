use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::Result,
  plugin::{Plugin, PluginResolveHookParam, PluginResolveHookResult},
};

/// ScriptPlugin is used to support compiling js/ts/jsx/tsx files to js chunks
pub struct FarmPluginResolve {}

impl FarmPluginResolve {
  pub fn new(config: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginResolve {
  fn name(&self) -> &str {
    "FarmPluginResolve"
  }

  fn resolve(
    &self,
    _param: &PluginResolveHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginResolveHookResult>> {
    Ok(Some(PluginResolveHookResult {
      id: String::from("resolved from FarmPluginResolve"),
      ..Default::default()
    }))
  }
}
