use std::{path::PathBuf, sync::Arc};

use farmfe_core::{context::CompilationContext, error::CompilationError};

pub fn write_resources(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  let mut resources_map = context.resources_map.lock();

  println!("writing resources");

  context
    .plugin_driver
    .write_resources(&mut *resources_map, context)?;

  // println!("emit resources to disk");

  // resources_map.values().try_for_each(|resource| {
  //   // TODO determine the emit location by config
  //   if !resource.emitted {
  //     let root = PathBuf::from(context.config.root.as_str());
  //     let output_path = root.join("dist").join(&resource.name);

  //     std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
  //     std::fs::write(output_path, &resource.bytes).unwrap();
  //   }

  //   Ok::<(), CompilationError>(())
  // })?;

  Ok(())
}
