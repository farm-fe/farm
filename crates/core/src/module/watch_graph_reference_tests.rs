// Original implementation without indexes retained only as a semantic oracle.
#[allow(dead_code)]
mod legacy {
  use std::collections::VecDeque;
  use std::hash::Hash;

  use petgraph::{
    EdgeDirection,
    graph::{DefaultIx, NodeIndex},
    stable_graph::StableDiGraph,
  };

  use crate::error::{CompilationError, Result};
  use crate::{HashMap, HashSet};

  use crate::module::ModuleId;

  /// ```md
  ///     a
  ///    /
  ///  v_b
  /// ```
  #[derive(Debug, Hash, PartialEq, Eq)]
  pub(super) enum EdgeMode {
    /// v_b -> a
    RootImport,
    /// a -> v_b
    WatchImport,
  }

  impl EdgeMode {
    fn is_root_import(&self) -> bool {
      *self == Self::RootImport
    }

    pub fn is_watch_import(&self) -> bool {
      *self == Self::WatchImport
    }
  }

  #[derive(Default)]
  pub struct WatchGraph {
    pub(super) g: StableDiGraph<ModuleId, EdgeMode>,
    id_index_map: HashMap<ModuleId, NodeIndex<DefaultIx>>,
  }

  impl WatchGraph {
    pub fn new() -> Self {
      Self {
        g: StableDiGraph::new(),
        id_index_map: HashMap::default(),
      }
    }

    pub fn add_node(&mut self, node: ModuleId) {
      if !self.has_module(&node) {
        let index = self.g.add_node(node.clone());
        self.id_index_map.insert(node, index);
      }
    }

    pub fn add_edge(&mut self, from: &ModuleId, to: &ModuleId) -> Result<()> {
      let from_index = self.id_index_map.get(from).ok_or_else(|| {
        CompilationError::GenericError(format!(
          r#"from node "{}" does not exist in the module graph when add edge"#,
          from.to_string()
        ))
      })?;

      let to_index = self.id_index_map.get(to).ok_or_else(|| {
        CompilationError::GenericError(format!(
          r#"to node "{}" does not exist in the module graph when add edge"#,
          to.to_string()
        ))
      })?;
      //         a                          h               c
      //       /   \                      /               /
      //     b      v_c                v_e              v_e
      //   /       /   \
      // v_d     v_e   v_f
      //           \    /
      //            v_g
      // e change causes a、f、c update
      // c change causes a
      // g change causes a update
      //
      // m  |  root
      // a -> []
      // b -> []
      // c -> [d]
      // d -> [b]
      // e -> [a,h,c]
      // f -> [a]
      // g -> [a]
      // h -> []
      let roots: Vec<ModuleId> = self.relation_roots(from).into_iter().cloned().collect();
      let roots: Vec<ModuleId> = [roots, vec![from.clone()]].concat();

      for root in roots.into_iter() {
        let root_id = self.id_index_map.get(&root).unwrap();

        self
          .g
          .update_edge(*to_index, *root_id, EdgeMode::RootImport);
      }

      self
        .g
        .update_edge(*from_index, *to_index, EdgeMode::WatchImport);

      Ok(())
    }

    pub fn modules(&self) -> Vec<&ModuleId> {
      let mut res = HashSet::default();

      for node in self.g.edge_indices() {
        if matches!(self.g.edge_weight(node), Some(EdgeMode::WatchImport))
          && let Some((_root, to)) = self.g.edge_endpoints(node)
        {
          res.insert(self.g.node_weight(to).unwrap());
        }
      }

      res.into_iter().collect()
    }

    pub fn relation_roots(&self, dep: &ModuleId) -> Vec<&ModuleId> {
      let mut result = HashSet::default();

      if let Some(index) = self.id_index_map.get(dep) {
        let mut edges = self
          .g
          .neighbors_directed(*index, EdgeDirection::Outgoing)
          .detach();

        while let Some((edge, node)) = edges.next(&self.g) {
          if !self.g.edge_weight(edge).is_some_and(|e| e.is_root_import()) {
            continue;
          };

          result.insert(self.g.node_weight(node).unwrap());
        }
      };

      result.into_iter().collect()
    }

    pub fn dependencies(&self, module_id: &ModuleId) -> Vec<&ModuleId> {
      let mut result = vec![];

      if let Some(index) = self.id_index_map.get(module_id) {
        let mut edges = self
          .g
          .neighbors_directed(*index, EdgeDirection::Outgoing)
          .detach();

        while let Some((edge, node)) = edges.next(&self.g) {
          if !self
            .g
            .edge_weight(edge)
            .is_some_and(|e| e.is_watch_import())
          {
            continue;
          };

          result.push(self.g.node_weight(node).unwrap());
        }
      }

      result
    }

    pub fn relation_dependencies(&self, root: &ModuleId) -> Vec<&ModuleId> {
      let mut queue = self.dependencies(root).into_iter().collect::<VecDeque<_>>();
      let mut visited = HashSet::default();
      let mut result = vec![];

      while !queue.is_empty() {
        let item = queue.pop_front().unwrap();
        if !visited.insert(item) {
          continue;
        }

        result.push(item);

        for dep in self.dependencies(item) {
          if !visited.contains(dep) {
            queue.push_back(dep);
          }
        }
      }

      result
    }

    pub fn has_module(&self, module_id: &ModuleId) -> bool {
      self.id_index_map.contains_key(module_id)
    }

    pub fn delete_module(&mut self, module_id: &ModuleId) {
      if !self.id_index_map.contains_key(module_id) {
        return;
      }

      let index = self.id_index_map.remove(module_id).unwrap();

      let mut walk = self.g.neighbors_undirected(index).detach();

      while let Some((edge, _)) = walk.next(&self.g) {
        self.g.remove_edge(edge);
      }

      self.g.remove_node(index);
    }
  }
}

fn assert_same_watch_graph(actual: &WatchGraph, expected: &legacy::WatchGraph) {
  let mut actual_edges = actual
    .g
    .edge_indices()
    .map(|edge| {
      let (from, to) = actual.g.edge_endpoints(edge).unwrap();
      (
        actual.g[from].clone(),
        actual.g[to].clone(),
        actual.g[edge].is_root_import(),
      )
    })
    .collect::<Vec<_>>();
  let mut expected_edges = expected
    .g
    .edge_indices()
    .map(|edge| {
      let (from, to) = expected.g.edge_endpoints(edge).unwrap();
      (
        expected.g[from].clone(),
        expected.g[to].clone(),
        matches!(expected.g[edge], legacy::EdgeMode::RootImport),
      )
    })
    .collect::<Vec<_>>();
  actual_edges.sort();
  expected_edges.sort();
  assert_eq!(actual_edges, expected_edges);
  assert_eq!(actual.edge_indices.len(), actual.g.edge_count());
  for (&(from, to), &edge) in &actual.edge_indices {
    assert_eq!(actual.g.edge_endpoints(edge), Some((from, to)));
  }
  let root_count: usize = actual.root_edges.values().map(|roots| roots.len()).sum();
  assert_eq!(
    root_count,
    actual_edges.iter().filter(|(_, _, root)| *root).count()
  );
  for id in actual.id_index_map.keys() {
    let mut roots = actual.relation_roots(id);
    roots.sort();
    let mut reference_roots = expected.relation_roots(id);
    reference_roots.sort();
    assert_eq!(roots, reference_roots);
    assert_eq!(actual.dependencies(id), expected.dependencies(id));
    assert_eq!(
      actual.relation_dependencies(id),
      expected.relation_dependencies(id)
    );
    let mut deps = actual.dependencies(id);
    deps.sort();
    let mut reference_deps = expected.dependencies(id);
    reference_deps.sort();
    assert_eq!(deps, reference_deps);
    let mut all_deps = actual.relation_dependencies(id);
    all_deps.sort();
    let mut reference_all = expected.relation_dependencies(id);
    reference_all.sort();
    assert_eq!(all_deps, reference_all);
  }
  let mut modules = actual.modules();
  modules.sort();
  let mut reference_modules = expected.modules();
  reference_modules.sort();
  assert_eq!(modules, reference_modules);
}

#[derive(Clone, Copy)]
enum WatchOp {
  Node(usize),
  Edge(usize, usize),
  Delete(usize),
}

fn replay_watch_ops(ops: &[WatchOp]) {
  let mut actual = WatchGraph::new();
  let mut expected = legacy::WatchGraph::new();
  for op in ops {
    let id = |n: usize| ModuleId::from(format!("module-{n}"));
    match *op {
      WatchOp::Node(n) => {
        actual.add_node(id(n));
        expected.add_node(id(n));
      }
      WatchOp::Edge(a, b) => {
        actual.add_edge(&id(a), &id(b)).unwrap();
        expected.add_edge(&id(a), &id(b)).unwrap();
      }
      WatchOp::Delete(n) => {
        actual.delete_module(&id(n));
        expected.delete_module(&id(n));
      }
    }
    assert_same_watch_graph(&actual, &expected);
  }
}

#[test]
fn indexed_watch_edges_preserve_refresh_and_type_overwrites() {
  use WatchOp::*;
  let cases = [
    vec![Edge(0, 1), Edge(2, 0), Edge(0, 1)], // Late upstream must propagate on repeat.
    vec![
      Edge(2, 0),
      Edge(0, 1),
      Delete(2),
      Node(2),
      Edge(2, 0),
      Edge(0, 1),
    ],
    vec![Edge(2, 0), Edge(3, 0), Edge(0, 1), Edge(1, 4), Edge(0, 1)],
    vec![Edge(0, 1), Edge(1, 2), Edge(2, 0), Edge(0, 1), Edge(1, 0)],
    vec![Edge(0, 0), Edge(2, 0), Edge(0, 0), Edge(0, 1)],
    vec![
      Edge(2, 0),
      Edge(0, 1),
      Delete(0),
      Node(4),
      Node(0),
      Edge(2, 0),
      Edge(0, 1),
    ],
    vec![Edge(2, 0), Edge(0, 2), Edge(0, 1)], // Batch roots cannot be reused.
    vec![
      Edge(0, 1),
      Delete(1),
      Node(1),
      Edge(0, 1),
      Delete(1),
      Delete(1),
    ],
  ];
  for case in cases {
    let mut ops = (0..5).map(Node).collect::<Vec<_>>();
    ops.extend(case);
    replay_watch_ops(&ops);
  }
}

#[test]
fn indexed_watch_edges_match_legacy_random_mutations() {
  use WatchOp::*;
  for seed in 0..30_u64 {
    let mut state = seed + 1;
    let mut present = [false; 8];
    let mut ops = vec![];
    for _ in 0..150 {
      state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
      let a = ((state >> 32) % 8) as usize;
      if !present[a] {
        ops.push(Node(a));
        present[a] = true;
      } else if state % 7 == 0 {
        ops.push(Delete(a));
        present[a] = false;
      } else {
        let live = (0..8).filter(|&n| present[n]).collect::<Vec<_>>();
        let b = live[((state >> 40) as usize) % live.len()];
        ops.push(Edge(a, b));
      }
    }
    replay_watch_ops(&ops);
  }
}

#[test]
fn indexed_watch_star_repeated_registration_matches_legacy() {
  use WatchOp::*;
  let mut ops = (0..66).map(Node).collect::<Vec<_>>();
  ops.push(Edge(65, 0));
  for _ in 0..2 {
    for source in 1..65 {
      ops.push(Edge(0, source));
    }
  }
  replay_watch_ops(&ops);
}
