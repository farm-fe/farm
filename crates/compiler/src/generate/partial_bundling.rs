use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  hashbrown::HashMap,
  module::module_group::{ModuleGroup, ModuleGroupMap},
  plugin::PluginHookContext,
  resource::{
    resource_pot::{ResourcePot, ResourcePotId},
    resource_pot_graph::ResourcePotGraph,
  },
};

pub fn partial_bundling(
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<()> {
  let module_group_map = analyze_module_graph(context, hook_context)?;
  // insert the module group map into the context
  let mut context_module_group_map = context.module_group_map.write();
  context_module_group_map.replace(module_group_map);

  let resource_pot_graph =
    generate_resource_pot_graph(&mut *context_module_group_map, context, hook_context)?;
  // insert the resource pot graph into the context
  let mut g = context.resource_pot_graph.write();
  g.replace(resource_pot_graph);

  Ok(())
}

fn analyze_module_graph(
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<ModuleGroupMap> {
  let mut module_graph = context.module_graph.write();

  println!("analyze module graph start");

  let mut module_group_map = context
    .plugin_driver
    .analyze_module_graph(&mut *module_graph, context, hook_context)?
    .ok_or(CompilationError::PluginHookResultCheckError {
      hook_name: "analyze_module_graph".to_string(),
    })?;

  drop(module_graph);

  println!("module group map len: {}", module_group_map.len());
  Ok(module_group_map)
}

fn generate_resource_pot_graph(
  module_group_map: &mut ModuleGroupMap,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<ResourcePotGraph> {
  let mut resource_pot_graph = ResourcePotGraph::new();
  // TODO: parallel generate resource pots
  for g in module_group_map.module_groups_mut() {
    let resources_pots = call_partial_bundling_hook(g, context, hook_context)?;

    for resource_pot in resources_pots {
      resource_pot_graph.add_resource_pot(resource_pot);
    }
  }

  let module_graph = context.module_graph.read();
  let mut edges = HashMap::<ResourcePotId, Vec<ResourcePotId>>::new();

  // analyze resource pot graph dependencies
  for resource_pot in resource_pot_graph.resource_pots_mut() {
    for module_id in resource_pot.modules() {
      let deps = module_graph.dependencies_ids(module_id);

      for dep in deps {
        let dep_module = module_graph.module(&dep).unwrap();

        if let Some(dep_resource_pot) = &dep_module.resource_pot {
          if &resource_pot.id != dep_resource_pot {
            let dep_edges = edges
              .entry(resource_pot.id.clone())
              .or_insert_with(Vec::new);

            if !dep_edges.contains(dep_resource_pot) {
              dep_edges.push(dep_resource_pot.clone());
            }
          }
        } else {
          panic!("dep module's({:?}) resource pot is none", dep);
        }
      }
    }
  }

  for (resource_pot_id, deps) in &edges {
    for dep in deps {
      resource_pot_graph.add_edge(resource_pot_id, dep);
    }
  }

  Ok(resource_pot_graph)
}

pub fn call_partial_bundling_hook(
  g: &mut ModuleGroup,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<Vec<ResourcePot>> {
  context
    .plugin_driver
    .partial_bundling(g, context, hook_context)?
    .ok_or(CompilationError::PluginHookResultCheckError {
      hook_name: "partial_bundling".to_string(),
    })
}
