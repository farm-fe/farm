use std::sync::Arc;

use farmfe_core::{
  config::partial_bundling::PartialBundlingEnforceResourceConfig,
  context::CompilationContext,
  error::CompilationError,
  hashbrown::HashSet,
  module::{module_graph::ModuleGraph, module_group::ModuleGroupGraph, ModuleId},
  plugin::PluginHookContext,
  resource::{
    resource_pot::{ResourcePot, ResourcePotId, ResourcePotType},
    resource_pot_map::ResourcePotMap,
  },
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

  let resource_pot_map =
    generate_resource_pot_map(&mut context_module_group_graph, context, hook_context)?;
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
  module_group_graph: &mut ModuleGroupGraph,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<ResourcePotMap> {
  let mut modules = HashSet::new();
  let mut enforce_resource_pot_map = ResourcePotMap::new();
  let module_graph = context.module_graph.read();

  // generate enforce resource pots first
  for g in module_group_graph.module_groups_mut() {
    for module_id in g.modules() {
      if let Some(name) = get_enforce_resource_name_for_module(
        module_id,
        &context.config.partial_bundling.enforce_resources,
      ) {
        let (resource_pot_type, resource_pot_id) =
          get_resource_pot_id_for_enforce_resources(name.clone(), module_id, &module_graph);

        let resource_pot = enforce_resource_pot_map.resource_pot_mut(&resource_pot_id);

        if let Some(resource_pot) = resource_pot {
          resource_pot.add_module(module_id.clone());
        } else {
          let mut resource_pot = ResourcePot::new(resource_pot_id, resource_pot_type);
          resource_pot.add_module(module_id.clone());
          enforce_resource_pot_map.add_resource_pot(resource_pot);
        }
      } else {
        // if the module is not in any enforce resource pot, add it modules for partial bundling
        modules.insert(module_id.clone());
      }
    }
    modules.extend(g.modules().clone());
  }

  drop(module_graph);

  let mut resources_pots =
    call_partial_bundling_hook(&modules.into_iter().collect(), context, hook_context)?;
  // extends enforce resource pots
  resources_pots.extend(enforce_resource_pot_map.take_resource_pots());

  resources_pots =
    fill_necessary_fields_for_resource_pot(resources_pots, module_group_graph, context);

  let mut resource_pot_map = ResourcePotMap::new();

  for resource_pot in resources_pots {
    resource_pot_map.add_resource_pot(resource_pot);
  }

  Ok(resource_pot_map)
}

pub fn get_enforce_resource_name_for_module(
  module_id: &ModuleId,
  enforce_resources: &Vec<PartialBundlingEnforceResourceConfig>,
) -> Option<String> {
  for enforce_resource_config in enforce_resources {
    if enforce_resource_config
      .test
      .iter()
      .any(|test| test.is_match(&module_id.to_string()))
    {
      return Some(enforce_resource_config.name.clone());
    }
  }

  None
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

pub fn fill_necessary_fields_for_resource_pot(
  mut resources_pots: Vec<ResourcePot>,
  module_group_graph: &mut ModuleGroupGraph,
  context: &Arc<CompilationContext>,
) -> Vec<ResourcePot> {
  let mut module_graph = context.module_graph.write();

  for resource_pot in &mut resources_pots {
    let mut module_groups = HashSet::new();
    let mut entry_module = None;

    for module_id in resource_pot.modules() {
      let module = module_graph.module_mut(module_id).unwrap();
      module.resource_pot = Some(resource_pot.id.clone());
      module_groups.extend(module.module_groups.clone());

      if module_graph.entries.contains_key(module_id) {
        if entry_module.is_some() {
          panic!("a resource pot can only have one entry module");
        }
        entry_module = Some(module_id.clone());
      }
    }

    resource_pot.entry_module = entry_module;
    resource_pot.module_groups = module_groups.clone();

    for module_group_id in module_groups {
      let module_group = module_group_graph
        .module_group_mut(&module_group_id)
        .unwrap();
      module_group.add_resource_pot(resource_pot.id.clone());
    }
  }

  resources_pots
}

pub fn get_resource_pot_id_for_enforce_resources(
  name: String,
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
) -> (ResourcePotType, ResourcePotId) {
  let module = module_graph.module(module_id).unwrap();
  let resource_pot_type = ResourcePotType::from(module.module_type.clone());
  let resource_pot_id = ResourcePotId::new(format!("{}_{}", name, resource_pot_type.to_string()));

  (resource_pot_type, resource_pot_id)
}
// TODO test generate_resource_pot_map
