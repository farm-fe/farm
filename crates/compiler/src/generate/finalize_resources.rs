use std::sync::Arc;

use farmfe_core::{
  config::Mode, context::CompilationContext, plugin::PluginFinalizeResourcesHookParams,
};

pub fn finalize_resources(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  {
    let mut resources_map = context.resources_map.lock();

    let mut param = PluginFinalizeResourcesHookParams {
      resources_map: &mut resources_map,
      config: &context.config,
    };

    context
      .plugin_driver
      .finalize_resources(&mut param, context)?;

    // if cache enabled, clear unused resources
    if context.config.persistent_cache.enabled()
      && matches!(context.config.mode, Mode::Production)
      && !context.config.lazy_compilation
    {
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

// TODO handleGenerateResourceWrite
// TODO writeResourceToDisk (napi)
// TODO prepareOutDir
