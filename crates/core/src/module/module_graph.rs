use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use petgraph::{
  graph::{DefaultIx, NodeIndex},
  stable_graph::StableDiGraph,
};

use crate::{
  error::{CompilationError, Result},
  plugin::ResolveKind,
};

use super::{Module, ModuleId};

pub struct ModuleGraphEdge {
  pub kind: ResolveKind,
  pub order: usize,
}

pub struct ModuleGraph {
  g: StableDiGraph<Module, ModuleGraphEdge>,
  id_index_map: HashMap<ModuleId, NodeIndex<DefaultIx>>,
}

impl ModuleGraph {
  pub fn new() -> Self {
    Self {
      g: StableDiGraph::new(),
      id_index_map: HashMap::new(),
    }
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
