use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{self, module_graph::ModuleGraph},
};

pub fn mangle_exports_from_module_graph(
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) {
  let (sorted_modules, _) = module_graph.toposort();
}

fn dfs_module_graph(module_graph: &mut ModuleGraph, context: &Arc<CompilationContext>) {}
