use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  module::Module,
  plugin::{PluginAnalyzeDepsHookResultEntry, PluginFinalizeModuleHookParam},
};

pub fn finalize_module(
  module: &mut Module,
  deps: &Vec<PluginAnalyzeDepsHookResultEntry>,
  context: &Arc<CompilationContext>,
) -> Result<()> {
  let mut param = PluginFinalizeModuleHookParam { module, deps };
  context.plugin_driver.finalize_module(&mut param, context)
}
