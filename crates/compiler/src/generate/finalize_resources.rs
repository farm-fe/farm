use std::sync::Arc;

use farmfe_core::context::CompilationContext;

pub fn finalize_resources(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  {
    let mut resources_map = context.resources_map.lock();

    context
      .plugin_driver
      .finalize_resources(&mut resources_map, context)?;
  }

  Ok(())
}
