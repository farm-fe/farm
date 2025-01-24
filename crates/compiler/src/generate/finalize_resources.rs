use std::sync::Arc;

use farmfe_core::{
  config::Mode,
  context::CompilationContext,
  module::{
    module_group::{ModuleGroupId, ModuleGroupType},
    ModuleId,
  },
  plugin::{PluginFinalizeResourcesHookParam, PluginHandleEntryResourceHookParam},
  resource::{Resource, ResourceType},
  HashMap,
};
use farmfe_toolkit::resources::{
  get_dynamic_resources_code, get_dynamic_resources_map, get_initial_resources, InitialResources,
};

pub fn finalize_resources(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  {
    let mut resources_map = context.resources_map.lock();

    handle_entry_resource(&mut resources_map, context)?;

    let mut param = PluginFinalizeResourcesHookParam {
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

  let mut runtime_code = String::new();
  let mut runtime_resource_name = String::new();

  for resource in resources_map.values() {
    if matches!(resource.resource_type, ResourceType::Runtime) {
      runtime_code = String::from_utf8(resource.bytes.clone()).unwrap();
      runtime_resource_name = resource.name.clone();
    }
  }

  let mut params = PluginHandleEntryResourceHookParam {
    resource: Resource::default(),
    resource_sourcemap: None,
    module_graph: &module_graph,
    module_group_graph: &module_group_graph,
    resource_pot_map: &resource_pot_map,
    entry_module_id: &ModuleId::new("", "", ""),
    initial_resources: vec![],
    dynamic_resources: String::new(),
    dynamic_module_resources_map: String::new(),
    runtime_code: &runtime_code,
    runtime_resource_name: &runtime_resource_name,
    emit_runtime: false,
  };

  for (entry_module_id, _) in &module_graph.entries {
    params.entry_module_id = entry_module_id;

    let InitialResources {
      entry_resource_name,
      entry_resource_sourcemap_name,
      initial_resources,
    } = get_initial_resources(
      entry_module_id,
      &module_graph,
      &module_group_graph,
      &resource_pot_map,
      resources_map,
    );

    let entry_resource = resources_map.get_mut(&entry_resource_name).unwrap();
    params.resource = std::mem::take(entry_resource);

    if let Some(entry_resource_sourcemap_name) = entry_resource_sourcemap_name {
      let entry_resource_sourcemap = resources_map
        .get_mut(&entry_resource_sourcemap_name)
        .unwrap();
      params.resource_sourcemap = Some(std::mem::take(entry_resource_sourcemap));
    }

    params.initial_resources = initial_resources;

    let module_group_id = ModuleGroupId::new(entry_module_id, &ModuleGroupType::Entry);

    let dynamic_resources_map = get_dynamic_resources_map(
      &module_group_graph,
      &module_group_id,
      &resource_pot_map,
      resources_map,
      &module_graph,
    );
    let (dynamic_resources, dynamic_module_resources_map) =
      get_dynamic_resources_code(&dynamic_resources_map, context.config.mode.clone());

    params.dynamic_resources = dynamic_resources;
    params.dynamic_module_resources_map = dynamic_module_resources_map;

    context
      .plugin_driver
      .handle_entry_resource(&mut params, context)?;

    // write entry resource back to resources_map
    let resource = resources_map.get_mut(&params.resource.name).unwrap();
    *resource = params.resource;

    if let Some(resource) = std::mem::take(&mut params.resource_sourcemap) {
      let resource = resources_map.get_mut(&resource.name).unwrap();
      *resource = resource.clone();
    }
  }

  if params.emit_runtime {
    let runtime_resource = resources_map.get_mut(&runtime_resource_name).unwrap();
    runtime_resource.emitted = false;
  }

  Ok(())
}
