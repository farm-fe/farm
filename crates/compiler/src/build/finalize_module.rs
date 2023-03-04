use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  module::{Module, ModuleType},
  plugin::{PluginAnalyzeDepsHookResultEntry, PluginFinalizeModuleHookParam},
};
use farmfe_toolkit::tracing;

#[tracing::instrument(skip_all)]
pub fn finalize_module(
  module: &mut Module,
  deps: &Vec<PluginAnalyzeDepsHookResultEntry>,
  context: &Arc<CompilationContext>,
) -> Result<()> {
  tracing::trace!("finalize_module: {:?}", module.id);

  let mut param = PluginFinalizeModuleHookParam { module, deps };
  context.plugin_driver.finalize_module(&mut param, context)?;

  tracing::trace!("finalized_module: {:?}", module.id);

  Ok(())
}
