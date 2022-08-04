use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use petgraph::{
  graph::{DefaultIx, NodeIndex},
  stable_graph::StableDiGraph,
  EdgeDirection,
};

use crate::{
  error::{CompilationError, Result},
  plugin::ResolveKind,
};

use super::{Module, ModuleId};

pub struct ModuleGraphEdge {
  /// the source of this edge, for example, `./index.css`
  pub source: String,
  pub kind: ResolveKind,
  /// the order of this edge, for example, for:
  /// ```js
  /// import a from './a';
  /// import b from './b';
  /// ```
  /// the edge `./a`'s order is 0 and `./b`'s order is 1 (starting from 0).
  pub order: usize,
}

pub struct ModuleGraph {
  /// internal graph
  g: StableDiGraph<Module, ModuleGraphEdge>,
  /// to index module in the graph using [ModuleId]
  id_index_map: HashMap<ModuleId, NodeIndex<DefaultIx>>,
}

impl ModuleGraph {
  pub fn new() -> Self {
    Self {
      g: StableDiGraph::new(),
      id_index_map: HashMap::new(),
    }
  }

  pub fn get_dep_by_source(&self, module_id: &ModuleId, source: &str) -> &Module {
    let i = self
      .id_index_map
      .get(module_id)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", module_id));
    let mut edges = self
      .g
      .neighbors_directed(*i, EdgeDirection::Outgoing)
      .detach();

    while let Some((edge_index, node_index)) = edges.next(&self.g) {
      if &self.g[edge_index].source == source {
        return &self.g[node_index];
      }
    }

    panic!("source `{}` is not a edge of `{:?}`", source, module_id);
  }

  pub fn add_module(&mut self, module: Module) {
    let id = module.id.clone();
    let index = self.g.add_node(module);
    self.id_index_map.insert(id, index);
  }

  pub fn add_edge(
    &mut self,
    from: &ModuleId,
    to: &ModuleId,
    edge_info: ModuleGraphEdge,
  ) -> Result<()> {
    let from = self.id_index_map.get(from).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"from node "{}" does not exist in the module graph when add edge"#,
        from.path()
      ))
    })?;

    let to = self.id_index_map.get(to).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"to node "{}" does not exist in the module graph when add edge"#,
        to.path()
      ))
    })?;

    self.g.add_edge(*from, *to, edge_info);

    Ok(())
  }
}

impl Default for ModuleGraph {
  fn default() -> Self {
    Self::new()
  }
}
