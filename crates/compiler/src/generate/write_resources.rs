use std::sync::Arc;

use farmfe_core::context::CompilationContext;

pub fn write_resources(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  let mut resources_map = context.resources_map.lock();

  println!("writing resources");

  context
    .plugin_driver
    .write_resources(&mut *resources_map, context)?;

  Ok(())
}
