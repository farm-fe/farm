use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  module::Module,
  plugin::{PluginAnalyzeDepsHookResultEntry, PluginFinalizeModuleHookParam},
};

pub fn finalize_module(
  module: &mut Module,
  deps: &mut Vec<PluginAnalyzeDepsHookResultEntry>,
  context: &Arc<CompilationContext>,
) -> Result<()> {
  #[cfg(feature = "profile")]
  farmfe_core::puffin::profile_function!();

  let mut param = PluginFinalizeModuleHookParam { module, deps };
  context.plugin_driver.finalize_module(&mut param, context)?;
  
  Ok(())
}
