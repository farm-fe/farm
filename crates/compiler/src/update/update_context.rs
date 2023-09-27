use farmfe_core::{module::module_graph::ModuleGraph, parking_lot::RwLock};

/// Context for the update process, it will be re-created during each update.
pub struct UpdateContext {
  pub module_graph: RwLock<ModuleGraph>, // partial graph, constructed during the hmr update
}

impl UpdateContext {
  pub fn new() -> Self {
    let module_graph = ModuleGraph::new();

    Self {
      module_graph: RwLock::new(module_graph),
    }
  }
}
