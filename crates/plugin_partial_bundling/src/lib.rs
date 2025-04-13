use std::sync::Arc;

pub use analyze_module_graph::module_group_graph_from_entries;
pub use analyze_module_graph::module_group_graph_from_module_graph;
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::{module_graph::ModuleGraph, module_group::ModuleGroupGraph, ModuleId},
  plugin::{Plugin, PluginHookContext},
  resource::resource_pot::ResourcePot,
};
use generate_module_buckets::{generate_module_buckets_map, group_module_buckets_by_module_group};
use generate_resource_pots::generate_resource_pots;

// mod module_bucket;
mod analyze_module_graph;
mod generate_module_buckets;
mod generate_module_pots;
mod generate_resource_pots;
mod merge_module_pots;
mod module_bucket;
mod module_pot;
mod utils;

/// Partial Bundling implementation for Farm.
/// See https://github.com/farm-fe/rfcs/pull/9
pub struct FarmPluginPartialBundling {}

impl Plugin for FarmPluginPartialBundling {
  fn name(&self) -> &str {
    "FarmPluginPartialBundling"
  }

  fn priority(&self) -> i32 {
    99
  }

  fn analyze_module_graph(
    &self,
    module_graph: &mut ModuleGraph,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ModuleGroupGraph>> {
    let module_group_graph = module_group_graph_from_module_graph(module_graph);

    Ok(Some(module_group_graph))
  }

  /// The partial bundling algorithm's result should not be related to the order of the module group.
  /// Whatever the order of the module group is, the result should be the same.
  /// See https://github.com/farm-fe/rfcs/blob/main/rfcs/003-partial-bundling/rfc.md for more design details.
  fn partial_bundling(
    &self,
    modules: &Vec<ModuleId>,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<ResourcePot>>> {
    // 1. get module group graph and module graph
    let module_graph = context.module_graph.read();
    let module_group_graph = context.module_group_graph.read();
    // 2. generate module buckets and group by module group
    let module_buckets_map = generate_module_buckets_map(modules, &module_graph);
    let module_group_buckets =
      group_module_buckets_by_module_group(&module_buckets_map, &module_group_graph, &module_graph);

    // 3. generate resource pots
    let resource_pots = generate_resource_pots(
      module_group_buckets,
      module_buckets_map,
      &module_graph,
      &module_group_graph,
      &context.config,
    );

    Ok(Some(resource_pots))
  }
}

impl FarmPluginPartialBundling {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}
