use hashbrown::{HashMap, HashSet};

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
  /// entry modules of this module graph
  pub entries: Vec<ModuleId>,
}

impl ModuleGraph {
  pub fn new() -> Self {
    Self {
      g: StableDiGraph::new(),
      id_index_map: HashMap::new(),
      entries: vec![],
    }
  }

  /// Get the dep module of the specified module which imports the dep module using the specified source.
  /// Used to get module by (module, source) pair, for example, for `module a`:
  /// ```js
  /// import b from './b';
  /// ```
  /// we can get `module b` by `(module a, "./b")`.
  ///
  /// Panic if the dep does not exist or the source is not correct
  pub fn get_dep_by_source(&self, module_id: &ModuleId, source: &str) -> ModuleId {
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
        return self.g[node_index].id.clone();
      }
    }

    panic!("source `{}` is not a edge of `{:?}`", source, module_id);
  }

  pub fn module(&self, module_id: &ModuleId) -> Option<&Module> {
    let i = self.id_index_map.get(module_id);

    if let Some(i) = i {
      self.g.node_weight(*i)
    } else {
      None
    }
  }

  pub fn module_mut(&mut self, module_id: &ModuleId) -> Option<&mut Module> {
    let i = self.id_index_map.get(module_id);

    if let Some(i) = i {
      self.g.node_weight_mut(*i)
    } else {
      None
    }
  }

  pub fn modules(&self) -> Vec<&Module> {
    self.g.node_weights().into_iter().collect()
  }

  pub fn modules_mut(&mut self) -> Vec<&mut Module> {
    self.g.node_weights_mut().into_iter().collect()
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

  /// get dependencies of the specific module, sorted by the order of the edge.
  /// for example, for `module a`:
  /// ```js
  /// import c from './c';
  /// import b from './b';
  /// ```
  /// return `['module c', 'module b']`, ensure the order of original imports.
  pub fn dependencies(&self, module_id: &ModuleId) -> Vec<ModuleId> {
    let i = self
      .id_index_map
      .get(module_id)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", module_id));
    let mut edges = self
      .g
      .neighbors_directed(*i, EdgeDirection::Outgoing)
      .detach();

    let mut deps = vec![];

    while let Some((edge_index, node_index)) = edges.next(&self.g) {
      deps.push((self.g[edge_index].order, self.g[node_index].id.clone()));
    }

    deps.sort_by_key(|dep| dep.0);
    deps.into_iter().map(|dep| dep.1).collect()
  }

  /// get dependent of the specific module.
  /// don't ensure the result's order.
  pub fn dependents(&self, module_id: &ModuleId) -> Vec<ModuleId> {
    let i = self
      .id_index_map
      .get(module_id)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", module_id));
    let mut edges = self
      .g
      .neighbors_directed(*i, EdgeDirection::Incoming)
      .detach();

    let mut deps = vec![];

    while let Some(node_index) = edges.next_node(&self.g) {
      deps.push(self.g[node_index].id.clone());
    }

    deps
  }

  /// sort the module graph topologically using post order dfs
  /// return (topologically sorted modules, cyclic modules)
  pub fn toposort(&self) -> (Vec<ModuleId>, Vec<ModuleId>) {
    fn dfs(
      entry: &ModuleId,
      graph: &ModuleGraph,
      stack: &mut Vec<ModuleId>,
      visited: &mut HashSet<ModuleId>,
      result: &mut Vec<ModuleId>,
      cyclic: &mut Vec<ModuleId>,
    ) {
      // cycle detected
      if stack.contains(&entry) {
        cyclic.push(entry.clone());
        return;
      } else if visited.contains(&entry) {
        // skip visited module
        return;
      }
      visited.insert(entry.clone());
      stack.push(entry.clone());

      for dep in &graph.dependencies(entry) {
        dfs(dep, graph, stack, visited, result, cyclic)
      }

      // visit current entry
      result.push(stack.pop().unwrap());
    }

    let mut result = vec![];
    let mut cyclic = vec![];
    let mut stack = vec![];
    let mut visited = HashSet::new();

    for entry in &self.entries {
      dfs(
        entry,
        self,
        &mut stack,
        &mut visited,
        &mut result,
        &mut cyclic,
      );
    }

    result.reverse();

    (result, cyclic)
  }
}

impl Default for ModuleGraph {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    module::{Module, ModuleId, ModuleType},
    plugin::ResolveKind,
  };

  use super::{ModuleGraph, ModuleGraphEdge};

  /// construct a test module graph like below:
  /// ```plain
  ///           A   B
  ///          / \ / \
  ///         C   D   E
  ///          \ /    |
  ///           F     G
  /// ```
  /// * **dynamic dependencies**: `A -> D`, `C -> G`, `D -> G`, `E -> H`
  /// * others are static dependencies
  /// * cyclic dependencies from `F -> A`
  fn construct_test_module_graph() -> ModuleGraph {
    let module_ids = vec!["A", "B", "C", "D", "E", "F", "G"]
      .into_iter()
      .map(|i| i.into());
    let mut graph = ModuleGraph::new();

    for id in module_ids {
      let m = Module::new(id, ModuleType::Js);

      graph.add_module(m);
    }

    let static_edges = vec![("A", "C", 0), ("B", "D", 0), ("B", "E", 1)];
    let dynamic_edges = vec![("A", "D", 1), ("C", "F", 0), ("D", "F", 0), ("E", "G", 0)];

    for (from, to, order) in static_edges {
      graph
        .add_edge(
          &from.into(),
          &to.into(),
          ModuleGraphEdge {
            source: format!("./{}", to),
            kind: ResolveKind::Import,
            order,
          },
        )
        .unwrap();
    }

    for (from, to, order) in dynamic_edges {
      graph
        .add_edge(
          &from.into(),
          &to.into(),
          ModuleGraphEdge {
            source: format!("./{}", to),
            kind: ResolveKind::DynamicImport,
            order,
          },
        )
        .unwrap();
    }

    graph
      .add_edge(
        &"F".into(),
        &"A".into(),
        ModuleGraphEdge {
          source: "./F".to_string(),
          kind: ResolveKind::Import,
          order: 0,
        },
      )
      .unwrap();

    graph.entries = vec!["A".into(), "B".into()];

    graph
  }

  #[test]
  fn toposort() {
    let graph = construct_test_module_graph();

    let (sorted, cycle) = graph.toposort();
    println!("{:?} {:?}", sorted, cycle);
    assert_eq!(cycle, vec!["A".into()]);
    assert_eq!(
      sorted,
      vec!["B", "E", "G", "A", "D", "C", "F"]
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<ModuleId>>()
    );
  }
}
