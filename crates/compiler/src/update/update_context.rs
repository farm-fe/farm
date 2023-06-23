use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::module_graph::{ModuleGraph, ModuleGraphEdge},
  parking_lot::RwLock,
};

use crate::Compiler;

use super::UpdateType;

/// Context for the update process, it will be re-created during each update.
pub struct UpdateContext {
  pub module_graph: RwLock<ModuleGraph>, // partial graph, constructed during the hmr update
}

impl UpdateContext {
  pub fn new() -> Self {
    let mut module_graph = ModuleGraph::new();

    Self {
      module_graph: RwLock::new(module_graph),
    }
  }
}
