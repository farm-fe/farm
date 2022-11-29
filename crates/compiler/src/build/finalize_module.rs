use std::sync::Arc;

use farmfe_core::{context::CompilationContext, error::Result, module::ModuleId};

pub fn finalize_module(module_id: &ModuleId, context: &Arc<CompilationContext>) -> Result<()> {
  context.plugin_driver.finalize_module(module_id, context)
}
