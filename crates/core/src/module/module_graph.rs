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
  pub entries: HashSet<ModuleId>,
}

impl ModuleGraph {
  pub fn new() -> Self {
    Self {
      g: StableDiGraph::new(),
      id_index_map: HashMap::new(),
      entries: HashSet::new(),
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
      if self.g[edge_index].source == source {
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

  pub fn has_module(&self, module_id: &ModuleId) -> bool {
    self.id_index_map.contains_key(module_id)
  }

  pub fn update_module(&mut self, module: Module) {
    let id = module.id.clone();
    let index = self.id_index_map.get(&id).unwrap();
    self.g[*index] = module;
  }

  pub fn add_module(&mut self, module: Module) {
    let id = module.id.clone();
    let index = self.g.add_node(module);
    self.id_index_map.insert(id, index);
  }

  pub fn remove_module(&mut self, module_id: &ModuleId) -> Module {
    let index = self.id_index_map.get(module_id).unwrap();
    self.g.remove_node(*index).unwrap()
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
        from.relative_path()
      ))
    })?;

    let to = self.id_index_map.get(to).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"to node "{}" does not exist in the module graph when add edge"#,
        to.relative_path()
      ))
    })?;
    // using update_edge instead of add_edge to avoid duplicated edges, see https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html#method.update_edge
    self.g.update_edge(*from, *to, edge_info);

    Ok(())
  }

  pub fn edge_info(&self, from: &ModuleId, to: &ModuleId) -> Option<&ModuleGraphEdge> {
    let from = self.id_index_map.get(from).unwrap();
    let to = self.id_index_map.get(to).unwrap();

    if let Some(edge_index) = self.g.find_edge(*from, *to) {
      self.g.edge_weight(edge_index)
    } else {
      None
    }
  }

  /// get dependencies of the specific module, sorted by the order of the edge.
  /// for example, for `module a`:
  /// ```js
  /// import c from './c';
  /// import b from './b';
  /// ```
  /// return `['module c', 'module b']`, ensure the order of original imports.
  pub fn dependencies(&self, module_id: &ModuleId) -> Vec<(ModuleId, ResolveKind)> {
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
      deps.push((
        self.g[edge_index].order,
        self.g[node_index].id.clone(),
        self.g[edge_index].kind.clone(),
      ));
    }

    deps.sort_by_key(|dep| dep.0);
    deps.into_iter().map(|dep| (dep.1, dep.2)).collect()
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

  /// sort the module graph topologically using post order dfs, note this topo sort also keeps the original import order.
  /// return (topologically sorted modules, cyclic modules stack)
  ///
  /// **Unsupported Situation**: if the two entries shares the same dependencies but the import order is not the same, may cause one entry don't keep original import order, this may bring problems in css as css depends on the order.
  pub fn toposort(&self) -> (Vec<ModuleId>, Vec<Vec<ModuleId>>) {
    fn dfs(
      entry: &ModuleId,
      graph: &ModuleGraph,
      stack: &mut Vec<ModuleId>,
      visited: &mut HashSet<ModuleId>,
      result: &mut Vec<ModuleId>,
      cyclic: &mut Vec<Vec<ModuleId>>,
    ) {
      // cycle detected
      if stack.contains(entry) {
        let pos = stack.iter().position(|m| m == entry).unwrap();
        cyclic.push(stack.clone()[pos..].to_vec());
        return;
      } else if visited.contains(entry) {
        // skip visited module
        return;
      }

      visited.insert(entry.clone());
      stack.push(entry.clone());

      let mut deps = graph.dependencies(entry);
      deps.reverse(); // reverse it as we use post order traverse

      for (dep, _) in &deps {
        dfs(dep, graph, stack, visited, result, cyclic)
      }

      // visit current entry
      result.push(stack.pop().unwrap());
    }

    let mut result = vec![];
    let mut cyclic = vec![];
    let mut stack = vec![];

    // sort entries to make sure it is stable
    let mut entries = self.entries.iter().collect::<Vec<_>>();
    entries.sort();

    for entry in entries {
      let mut res = vec![];
      let mut visited = HashSet::new();

      dfs(entry, self, &mut stack, &mut visited, &mut res, &mut cyclic);
      res.reverse();

      // merge the topo sort result of the entries.
      if result.is_empty() {
        result.extend(res);
      } else {
        self.merge_topo_sorted_vec(&mut result, &res);
      }
    }

    (result, cyclic)
  }

  fn merge_topo_sorted_vec(&self, base: &mut Vec<ModuleId>, new: &[ModuleId]) {
    if new.is_empty() {
      return;
    }

    if let Some(pos) = new.iter().position(|nm| base.contains(nm)) {
      if pos > 0 {
        let nm = &new[pos];
        let base_pos = base.iter().position(|bm| bm == nm).unwrap();
        base.splice(base_pos..(base_pos + 1), new[0..(pos + 1)].to_vec());

        self.merge_topo_sorted_vec(base, &new[(pos + 1)..]);
      } else {
        self.merge_topo_sorted_vec(base, &new[(pos + 1)..]);
      }
    } else {
      base.extend(new.to_vec());
    }
  }
}

impl Default for ModuleGraph {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use hashbrown::HashSet;

  use crate::{
    module::{Module, ModuleId},
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
      let m = Module::new(id);

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

    graph.entries = HashSet::from(["A".into(), "B".into()]);

    graph
  }

  #[test]
  fn toposort() {
    let graph = construct_test_module_graph();

    let (sorted, cycle) = graph.toposort();
    println!("{:?} \n\n {:?}", sorted, cycle);
    assert_eq!(
      cycle,
      vec![
        vec!["A".into(), "D".into(), "F".into()],
        // vec!["A".into(), "C".into(), "F".into()],
        vec!["D".into(), "F".into(), "A".into()],
        vec!["F".into(), "A".into(), "C".into()]
      ]
    );
    assert_eq!(
      sorted,
      vec!["A", "C", "B", "D", "F", "E", "G"]
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<ModuleId>>()
    );
  }
}
