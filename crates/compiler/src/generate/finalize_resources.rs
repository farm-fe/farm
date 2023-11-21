use std::sync::Arc;

use farmfe_core::context::CompilationContext;

pub fn finalize_resources(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  {
    let mut resources_map = context.resources_map.lock();

    context
      .plugin_driver
      .finalize_resources(&mut resources_map, context)?;

    // if cache enabled, clear unused resources
    if context.config.persistent_cache.enabled() {
      let mut resources_to_remove = vec![];
      let module_graph = context.module_graph.read();

      for resource in resources_map.values() {
        match &resource.origin {
          farmfe_core::resource::ResourceOrigin::ResourcePot(_) => { /* do nothing for resource pot */
          }
          farmfe_core::resource::ResourceOrigin::Module(m) => {
            if !module_graph.has_module(m) {
              resources_to_remove.push(resource.name.clone());
            }
          }
        }
      }

      resources_to_remove.into_iter().for_each(|r| {
        resources_map.remove(&r);
      });
    }
  }

  Ok(())
}
