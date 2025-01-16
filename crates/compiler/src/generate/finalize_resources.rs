use std::sync::Arc;

use farmfe_core::{
  config::Mode,
  context::CompilationContext,
  plugin::{PluginFinalizeResourcesHookParams, PluginHandleEntryResourceHookParams},
  resource::{Resource, ResourceType},
  HashMap,
};

pub fn finalize_resources(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  {
    let mut resources_map = context.resources_map.lock();

    handle_entry_resource(&mut resources_map, context)?;

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

fn handle_entry_resource(
  resources_map: &mut HashMap<String, Resource>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  let module_graph = context.module_graph.read();
  let module_group_graph = context.module_group_graph.read();
  let resource_pot_map = context.resource_pot_map.read();

  for (entry_module_id, _) in &module_graph.entries {
    let module = module_graph.module(entry_module_id).unwrap();

    for resource_pot in &module.resource_pots {
      let resource_pot = resource_pot_map.resource_pot(resource_pot).unwrap();

      let mut params = PluginHandleEntryResourceHookParams {
        resource: Resource::default(),
        resource_source_map: None,
        module_graph: &module_graph,
        module_group_graph: &module_group_graph,
        entry_module_id,
      };

      for resource_name in resource_pot.resources() {
        // get resource from resources_map
        let resource = resources_map.get_mut(resource_name).unwrap();
        let resource = std::mem::replace(resource, Resource::default());

        if let ResourceType::Js = &resource.resource_type {
          params.resource = resource;
        } else if let ResourceType::SourceMap(_) = &resource.resource_type {
          params.resource_source_map = Some(resource);
        }
      }

      context
        .plugin_driver
        .handle_entry_resource(&mut params, context)?;

      // write entry resource back to resources_map
      let resource = resources_map.get_mut(&params.resource.name).unwrap();
      *resource = params.resource;

      if let Some(resource) = params.resource_source_map {
        let resource = resources_map.get_mut(&resource.name).unwrap();
        *resource = resource.clone();
      }
    }
  }

  Ok(())
}
