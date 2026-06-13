use std::collections::VecDeque;
use std::hash::Hash;

use petgraph::{
  graph::{DefaultIx, NodeIndex},
  stable_graph::StableDiGraph,
  EdgeDirection,
};

use crate::error::{CompilationError, Result};
use crate::{HashMap, HashSet};

use super::ModuleId;

/// ```md
///     a
///    /
///  v_b
/// ```
#[derive(Debug, Hash, PartialEq, Eq)]
enum EdgeMode {
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
  g: StableDiGraph<ModuleId, EdgeMode>,
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
      visited.insert(item);

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

#[cfg(test)]
mod tests {

  use crate::module::ModuleId;

  use super::WatchGraph;
  ///```md
  ///    a            a          v_c
  ///      \        /   \        /
  ///      v_c     v_c  v_d   v_d
  ///      /
  ///   v_d
  ///```
  fn create_watch_graph_instance() -> WatchGraph {
    let mut watch_graph = WatchGraph::new();
    let a: ModuleId = "a".into();
    let v_c: ModuleId = "v_c".into();
    let v_d: ModuleId = "v_d".into();
    watch_graph.add_node(a.clone());
    watch_graph.add_node(v_c.clone());
    watch_graph.add_node(v_d.clone());

    watch_graph.add_edge(&a, &v_c).unwrap();
    watch_graph.add_edge(&v_c, &v_d).unwrap();

    watch_graph
  }

  #[test]
  fn modules() {
    let v_c = "v_c".into();
    let v_d = "v_d".into();
    let watch_graph = create_watch_graph_instance();

    let expect = vec![&v_c, &v_d];
    let mut resources = watch_graph.modules();
    resources.sort();
    assert_eq!(resources, expect)
  }

  #[test]
  fn relation_roots() {
    //          a
    //            \
    //            v_c
    //            /
    //          v_d
    let watch_graph = create_watch_graph_instance();

    assert_eq!(watch_graph.relation_roots(&"v_c".into()), vec![&"a".into()]);

    let mut r = watch_graph.relation_roots(&"v_d".into());
    r.sort();
    assert_eq!(r, [&"a".into(), &"v_c".into()])
  }

  #[test]
  fn relation_dependencies() {
    let watch_graph = create_watch_graph_instance();

    assert_eq!(watch_graph.dependencies(&"v_c".into()), vec![&"v_d".into()]);
    let result = watch_graph.dependencies(&ModuleId::from("v_d"));
    assert_eq!(result, Vec::<&ModuleId>::new());

    let mut r = watch_graph.relation_dependencies(&"a".into());
    r.sort();
    assert_eq!(r, [&"v_c".into(), &"v_d".into()])
  }
}
