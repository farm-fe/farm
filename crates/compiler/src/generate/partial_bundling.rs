use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  hashbrown::HashSet,
  module::{module_group::ModuleGroupGraph, ModuleId},
  plugin::PluginHookContext,
  resource::{resource_pot::ResourcePot, resource_pot_map::ResourcePotMap},
};

pub fn partial_bundling(
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<()> {
  #[cfg(feature = "profile")]
  farmfe_core::puffin::profile_function!();

  let module_group_graph = analyze_module_graph(context, hook_context)?;
  // insert the module group map into the context
  let mut context_module_group_graph = context.module_group_graph.write();
  context_module_group_graph.replace(module_group_graph);
  drop(context_module_group_graph);
  let resource_pot_map = generate_resource_pot_map(context, hook_context)?;
  // insert the resource pot graph into the context
  let mut g = context.resource_pot_map.write();
  g.replace(resource_pot_map);

  Ok(())
}

fn analyze_module_graph(
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<ModuleGroupGraph> {
  let mut module_graph = context.module_graph.write();

  let module_group_graph = context
    .plugin_driver
    .analyze_module_graph(&mut module_graph, context, hook_context)?
    .ok_or(CompilationError::PluginHookResultCheckError {
      hook_name: "analyze_module_graph".to_string(),
    })?;

  Ok(module_group_graph)
}

fn generate_resource_pot_map(
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<ResourcePotMap> {
  let mut module_group_graph = context.module_group_graph.write();
  let mut modules = HashSet::new();

  for g in module_group_graph.module_groups_mut() {
    modules.extend(g.modules().clone());
  }

  drop(module_group_graph);

  let resources_pots =
    call_partial_bundling_hook(&modules.into_iter().collect(), context, hook_context)?;

  let mut module_group_graph = context.module_group_graph.write();

  let mut resource_pot_map = ResourcePotMap::new();
  let module_graph = context.module_graph.read();

  // TODO: module_id maybe in multiple resource_pot, so we need to keep it correctly
  for mut resource_pot in resources_pots {
    let mut module_groups = HashSet::new();

    let resource_modules = resource_pot.modules();
    if resource_pot.entry_module.is_some() {
      module_groups.insert(resource_pot.entry_module.clone().unwrap());
    } else {
      for module_id in resource_modules {
        let module = module_graph.module(module_id).unwrap();
        module_groups.extend(module.module_groups.clone());
      }
    }

    resource_pot.module_groups = module_groups.clone();

    for module_group_id in module_groups {
      let module_group = module_group_graph
        .module_group_mut(&module_group_id)
        .unwrap();
      module_group.add_resource_pot(resource_pot.id.clone());
    }

    resource_pot_map.add_resource_pot(resource_pot);
  }

  Ok(resource_pot_map)
}

pub fn call_partial_bundling_hook(
  modules: &Vec<ModuleId>,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<Vec<ResourcePot>> {
  let res = context
    .plugin_driver
    .partial_bundling(modules, context, hook_context)?
    .ok_or(CompilationError::PluginHookResultCheckError {
      hook_name: "partial_bundling".to_string(),
    })?;

  Ok(res)
}
