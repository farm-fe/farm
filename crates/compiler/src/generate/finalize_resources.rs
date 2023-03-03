use std::sync::Arc;

use farmfe_core::context::CompilationContext;
use farmfe_toolkit::tracing;

#[tracing::instrument(skip_all)]
pub fn finalize_resources(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  tracing::trace!("Staring finalize_resources...");
  let mut resources_map = context.resources_map.lock();

  context
    .plugin_driver
    .finalize_resources(&mut *resources_map, context)?;

  tracing::trace!("Finished finalize_resources.");
  Ok(())
}
