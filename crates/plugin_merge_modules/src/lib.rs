use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{module_graph::ModuleGraph, module_group::ModuleGroupMap, Module},
  parking_lot::RwLock,
  plugin::{Plugin, PluginHookContext},
  resource::resource_pot_graph::ResourcePotGraph,
};

struct FarmPluginMergeModules {}

impl Plugin for FarmPluginMergeModules {
  fn name(&self) -> &str {
    "FarmPluginMergeModules"
  }

  fn analyze_module_graph(
    &self,
    module_graph: &RwLock<ModuleGraph>,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ModuleGroupMap>> {
    let mut module_graph = module_graph.write();
    let (topo_sorted_modules, _) = module_graph.toposort();

    Ok(None)
  }

  fn merge_modules(
    &self,
    _module_group: &ModuleGroupMap,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ResourcePotGraph>> {
    Ok(None)
  }
}
