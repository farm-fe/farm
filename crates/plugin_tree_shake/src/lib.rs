use farmfe_core::plugin::Plugin;

mod statement_graph;

pub struct FarmPluginTreeShake;

impl Plugin for FarmPluginTreeShake {
  fn name(&self) -> &'static str {
    "FarmPluginTreeShake"
  }

  /// tree shake useless modules and code, steps:
  /// 1. create a statement graph from module graph, start from entry module to all modules in topological order
  /// 2.
  fn optimize_module_graph(
    &self,
    _module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }
}
