use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  hashbrown::HashMap,
  module::module_group::{ModuleGroup, ModuleGroupGraph},
  plugin::PluginHookContext,
  resource::{
    resource_pot::{ResourcePot, ResourcePotId},
    resource_pot_graph::ResourcePotGraph,
  },
};
use farmfe_toolkit::tracing;

#[tracing::instrument(skip_all)]
pub fn partial_bundling(
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<()> {
  tracing::debug!("partial_bundling");

  let module_group_graph = analyze_module_graph(context, hook_context)?;
  // insert the module group map into the context
  let mut context_module_group_graph = context.module_group_graph.write();
  context_module_group_graph.replace(module_group_graph);

  let resource_pot_graph =
    generate_resource_pot_graph(&mut *context_module_group_graph, context, hook_context)?;
  // insert the resource pot graph into the context
  let mut g = context.resource_pot_graph.write();
  g.replace(resource_pot_graph);

  tracing::debug!("partial_bundling finished");

  Ok(())
}

#[tracing::instrument(skip_all)]
fn analyze_module_graph(
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<ModuleGroupGraph> {
  tracing::debug!("Starting analyze_module_graph");

  let mut module_graph = context.module_graph.write();

  let module_group_graph = context
    .plugin_driver
    .analyze_module_graph(&mut *module_graph, context, hook_context)?
    .ok_or(CompilationError::PluginHookResultCheckError {
      hook_name: "analyze_module_graph".to_string(),
    })?;

  tracing::debug!("analyze_module_graph finished");
  Ok(module_group_graph)
}

#[tracing::instrument(skip_all)]
fn generate_resource_pot_graph(
  module_group_graph: &mut ModuleGroupGraph,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<ResourcePotGraph> {
  tracing::debug!("Starting generate_resource_pot_graph");

  let mut resource_pot_graph = ResourcePotGraph::new();
  // TODO: parallel generate resource pots
  for g in module_group_graph.module_groups_mut() {
    let resources_pots = call_partial_bundling_hook(g, context, hook_context)?;

    for resource_pot in resources_pots {
      g.add_resource_pot(resource_pot.id.clone());
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

  tracing::debug!("generate_resource_pot_graph finished");
  Ok(resource_pot_graph)
}

#[tracing::instrument(skip_all)]
pub fn call_partial_bundling_hook(
  g: &mut ModuleGroup,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<Vec<ResourcePot>> {
  tracing::debug!("Starting call_partial_bundling_hook");

  let res = context
    .plugin_driver
    .partial_bundling(g, context, hook_context)?
    .ok_or(CompilationError::PluginHookResultCheckError {
      hook_name: "partial_bundling".to_string(),
    })?;

  tracing::debug!("call_partial_bundling_hook finished");

  Ok(res)
}
