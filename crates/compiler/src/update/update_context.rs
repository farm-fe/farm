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
  pub fn new(context: &Arc<CompilationContext>, paths: &Vec<(String, UpdateType)>) -> Self {
    let existing_module_graph = context.module_graph.read();
    let mut module_graph = ModuleGraph::new();

    existing_module_graph.dfs_breakable(
      existing_module_graph
        .entries
        .clone()
        .into_iter()
        .map(|(entry, _)| entry)
        .collect(),
      &mut |parent_id, module_id| {
        if paths
          .iter()
          .any(|(path, _)| path == &module_id.resolved_path(&context.config.root))
        {
          return false;
        }

        let existing_module = existing_module_graph.module(module_id).unwrap();
        let module = Compiler::create_module(
          module_id.clone(),
          existing_module.external,
          existing_module.immutable,
        );

        module_graph.add_module(module);

        if let Some(parent_id) = parent_id {
          module_graph
            .add_edge(parent_id, module_id, ModuleGraphEdge::default())
            .unwrap();
        }

        true
      },
    );

    Self {
      module_graph: RwLock::new(module_graph),
    }
  }
}
