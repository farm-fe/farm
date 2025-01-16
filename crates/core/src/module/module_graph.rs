use std::cmp::Ordering;

use crate::{HashMap, HashSet};
use farmfe_macro_cache_item::cache_item;

use petgraph::{
  graph::{DefaultIx, NodeIndex},
  stable_graph::StableDiGraph,
  visit::{Bfs, Dfs},
  EdgeDirection,
};

use crate::{
  error::{CompilationError, Result},
  plugin::ResolveKind,
};

use super::{Module, ModuleId};

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cache_item]
pub struct ModuleGraphEdgeDataItem {
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cache_item]
pub struct ModuleGraphEdge(pub(crate) Vec<ModuleGraphEdgeDataItem>);

impl ModuleGraphEdge {
  pub fn new(items: Vec<ModuleGraphEdgeDataItem>) -> Self {
    Self(items)
  }

  pub fn items(&self) -> &[ModuleGraphEdgeDataItem] {
    &self.0
  }

  pub fn update_kind(&mut self, kind: ResolveKind) {
    for item in &mut self.0 {
      item.kind = kind.clone();
    }
  }

  pub fn iter(&self) -> impl Iterator<Item = &ModuleGraphEdgeDataItem> {
    self.0.iter()
  }

  pub fn contains(&self, item: &ModuleGraphEdgeDataItem) -> bool {
    self.0.contains(item)
  }

  pub fn push(&mut self, item: ModuleGraphEdgeDataItem) {
    self.0.push(item);
  }

  /// true if all of the edge data items are dynamic import
  pub fn is_dynamic_import(&self) -> bool {
    if self.0.is_empty() {
      return false;
    }

    self.0.iter().all(|item| item.kind.is_dynamic_import())
  }

  /// true if all of the edge data items are dynamic entry
  pub fn is_dynamic_entry(&self) -> bool {
    if self.0.is_empty() {
      return false;
    }

    self.0.iter().all(|item| item.kind.is_dynamic_entry())
  }

  /// true if all of the edge data items are not dynamic entry or dynamic import
  pub fn is_static(&self) -> bool {
    if self.0.is_empty() {
      return false;
    }

    self
      .0
      .iter()
      .all(|item| !item.kind.is_dynamic_entry() && !item.kind.is_dynamic_import())
  }

  pub fn contains_export_from(&self) -> bool {
    if self.0.is_empty() {
      return false;
    }

    self.0.iter().any(|item| item.kind.is_export_from())
  }

  pub fn contains_require(&self) -> bool {
    if self.0.is_empty() {
      return false;
    }

    self.0.iter().any(|item| item.kind.is_require())
  }

  pub fn contains_dynamic_import(&self) -> bool {
    if self.0.is_empty() {
      return false;
    }

    self.0.iter().any(|item| item.kind.is_dynamic_import())
  }

  pub fn contains_dynamic_entry(&self) -> bool {
    if self.0.is_empty() {
      return false;
    }

    self.0.iter().any(|item| item.kind.is_dynamic_entry())
  }

  pub fn contains_static(&self) -> bool {
    if self.0.is_empty() {
      return false;
    }

    self
      .0
      .iter()
      .any(|item| !item.kind.is_dynamic_entry() && !item.kind.is_dynamic_import())
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

impl Default for ModuleGraphEdge {
  fn default() -> Self {
    Self::new(vec![ModuleGraphEdgeDataItem::default()])
  }
}

#[derive(Debug, Default)]
pub struct CircleRecord {
  sets: HashSet<ModuleId>,
}

impl CircleRecord {
  pub fn new(circles: Vec<Vec<ModuleId>>) -> Self {
    Self {
      sets: circles.into_iter().flatten().collect(),
    }
  }

  pub fn is_in_circle(&self, module_id: &ModuleId) -> bool {
    self.sets.contains(module_id)
  }
}

pub struct ModuleGraph {
  /// internal graph
  g: StableDiGraph<Module, ModuleGraphEdge>,
  /// to index module in the graph using [ModuleId]
  id_index_map: HashMap<ModuleId, NodeIndex<DefaultIx>>,
  /// file path to module ids, e.g src/index.scss -> [src/index.scss, src/index.scss?raw]
  file_module_ids_map: HashMap<ModuleId, Vec<ModuleId>>,
  /// entry modules of this module graph.
  /// (Entry Module Id, Entry Name)
  pub entries: HashMap<ModuleId, String>,
  pub dynamic_entries: HashMap<ModuleId, String>,
  pub circle_record: CircleRecord,
}

impl ModuleGraph {
  pub fn new() -> Self {
    Self {
      g: StableDiGraph::new(),
      id_index_map: HashMap::default(),
      file_module_ids_map: HashMap::default(),
      entries: HashMap::default(),
      dynamic_entries: HashMap::default(),
      circle_record: CircleRecord::default(),
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
  pub fn get_dep_by_source_optional(
    &self,
    module_id: &ModuleId,
    source: &str,
    kind: Option<ResolveKind>,
  ) -> Option<ModuleId> {
    let i = self
      .id_index_map
      .get(module_id)
      .unwrap_or_else(|| panic!("module_id {module_id:?} should in the module graph"));
    let mut edges = self
      .g
      .neighbors_directed(*i, EdgeDirection::Outgoing)
      .detach();

    while let Some((edge_index, node_index)) = edges.next(&self.g) {
      if self.g[edge_index]
        .iter()
        .any(|e| e.source == *source && (kind.is_none() || e.kind == *kind.as_ref().unwrap()))
      {
        return Some(self.g[node_index].id.clone());
      }
    }

    None
  }

  pub fn get_edges_of_module(&self, module_id: &ModuleId) -> Vec<&ModuleGraphEdgeDataItem> {
    let i = self
      .id_index_map
      .get(module_id)
      .unwrap_or_else(|| panic!("module_id {module_id:?} should in the module graph"));
    let mut edges = self
      .g
      .neighbors_directed(*i, EdgeDirection::Outgoing)
      .detach();

    let mut deps = vec![];

    while let Some((edge_index, _)) = edges.next(&self.g) {
      deps.extend(self.g[edge_index].iter());
    }

    deps
  }

  pub fn get_dep_by_source(
    &self,
    module_id: &ModuleId,
    source: &str,
    kind: Option<ResolveKind>,
  ) -> ModuleId {
    if let Some(id) = self.get_dep_by_source_optional(module_id, source, kind.clone()) {
      id
    } else {
      panic!(
        "source `{}`(kind {:?}, module type {:?}) is not a edge of `{:?}`, available edges: {:?}",
        source,
        kind,
        self.module(module_id).unwrap().module_type,
        module_id,
        self.get_edges_of_module(module_id)
      );
    }
  }

  pub fn update_edge(
    &mut self,
    from: &ModuleId,
    to: &ModuleId,
    edge_info: ModuleGraphEdge,
  ) -> Result<()> {
    let from = self.id_index_map.get(from).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"from node "{}" does not exist in the module graph when update edge"#,
        from.relative_path()
      ))
    })?;

    let to = self.id_index_map.get(to).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"to node "{}" does not exist in the module graph when update edge"#,
        to.relative_path()
      ))
    })?;

    self.g.update_edge(*from, *to, edge_info);

    Ok(())
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
    self.g.node_weights().collect()
  }

  pub fn modules_mut(&mut self) -> Vec<&mut Module> {
    self.g.node_weights_mut().collect()
  }

  pub fn has_module(&self, module_id: &ModuleId) -> bool {
    self.id_index_map.contains_key(module_id)
  }

  pub fn update_module(&mut self, module: Module) {
    let id = module.id.clone();
    let index = self.id_index_map.get(&id).unwrap();
    self.g[*index] = module;
  }

  pub fn module_ids_by_file(&self, module_id: &ModuleId) -> Vec<ModuleId> {
    if let Some(ids) = self.file_module_ids_map.get(module_id) {
      return ids.clone();
    }

    vec![]
  }

  pub fn add_module(&mut self, module: Module) {
    let id = module.id.clone();
    let index = self.g.add_node(module);

    if !id.query_string().is_empty() {
      let rel_path = id.relative_path();
      self
        .file_module_ids_map
        .entry(rel_path.into())
        .or_default()
        .push(id.clone())
    }

    self.id_index_map.insert(id, index);
  }

  pub fn remove_module(&mut self, module_id: &ModuleId) -> Module {
    let index = self
      .id_index_map
      .remove(module_id)
      .unwrap_or_else(|| panic!("module_id {module_id:?} should in the module graph"));

    if !module_id.query_string().is_empty() {
      if let Some(ids) = self
        .file_module_ids_map
        .get_mut(&module_id.relative_path().into())
      {
        ids.retain(|id| id != module_id);
      }
    }
    self.g.remove_node(index).unwrap()
  }

  pub fn add_edge_item(
    &mut self,
    from: &ModuleId,
    to: &ModuleId,
    edge_info: ModuleGraphEdgeDataItem,
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
    // if the edge already exists, we should update the edge info
    if let Some(edge_index) = self.g.find_edge(*from, *to) {
      if !self.g[edge_index].contains(&edge_info) {
        self.g[edge_index].push(edge_info);
      }
      return Ok(());
    }
    // using update_edge instead of add_edge to avoid duplicated edges, see https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html#method.update_edge
    self
      .g
      .update_edge(*from, *to, ModuleGraphEdge(vec![edge_info]));

    Ok(())
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

  pub fn remove_edge(&mut self, from: &ModuleId, to: &ModuleId) -> Result<Option<ModuleGraphEdge>> {
    let from_index = self.id_index_map.get(from).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"from node "{}" does not exist in the module graph when remove edge"#,
        from.relative_path()
      ))
    })?;

    let to_index = self.id_index_map.get(to).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"to node "{}" does not exist in the module graph when remove edge"#,
        to.relative_path()
      ))
    })?;

    let edge = self.g.find_edge(*from_index, *to_index).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"edge "{}" -> "{}" does not exist in the module graph when remove edge"#,
        from.relative_path(),
        to.relative_path()
      ))
    })?;

    let edge = self.g.remove_edge(edge);

    Ok(edge)
  }

  pub fn has_edge(&self, from: &ModuleId, to: &ModuleId) -> bool {
    let from = self.id_index_map.get(from);
    let to = self.id_index_map.get(to);

    if from.is_none() || to.is_none() {
      return false;
    }

    self.g.find_edge(*from.unwrap(), *to.unwrap()).is_some()
  }

  pub fn edge_info(&self, from: &ModuleId, to: &ModuleId) -> Option<&ModuleGraphEdge> {
    let from = self
      .id_index_map
      .get(from)
      .unwrap_or_else(|| panic!("module_id {from:?} should in the module graph"));
    let to = self
      .id_index_map
      .get(to)
      .unwrap_or_else(|| panic!("module_id {to:?} should in the module graph"));

    if let Some(edge_index) = self.g.find_edge(*from, *to) {
      self.g.edge_weight(edge_index)
    } else {
      None
    }
  }

  pub fn edge_count(&self) -> usize {
    self.g.edge_count()
  }

  /// get dependencies of the specific module, sorted by the order of the edge.
  /// for example, for `module a`:
  /// ```js
  /// import c from './c';
  /// import b from './b';
  /// ```
  /// return `['module c', 'module b']`, ensure the order of original imports.
  pub fn dependencies(&self, module_id: &ModuleId) -> Vec<(ModuleId, &ModuleGraphEdge)> {
    let i = self
      .id_index_map
      .get(module_id)
      .unwrap_or_else(|| panic!("module_id {module_id:?} should in the module graph"));
    let mut edges = self
      .g
      .neighbors_directed(*i, EdgeDirection::Outgoing)
      .detach();

    let mut deps = vec![];

    while let Some((edge_index, node_index)) = edges.next(&self.g) {
      deps.push((self.g[node_index].id.clone(), &self.g[edge_index]));
    }

    deps.sort_by(|a, b| {
      if a.1.is_empty() || b.1.is_empty() {
        return Ordering::Equal;
      }

      let a_minimum_order = a.1.iter().map(|item| item.order).min().unwrap();
      let b_minimum_order = b.1.iter().map(|item| item.order).min().unwrap();

      a_minimum_order.cmp(&b_minimum_order)
    });

    deps
  }

  /// Same as `dependencies`, but only return the module id.
  pub fn dependencies_ids(&self, module_id: &ModuleId) -> Vec<ModuleId> {
    self
      .dependencies(module_id)
      .into_iter()
      .map(|(id, _)| id)
      .collect()
  }

  /// get dependent of the specific module.
  /// don't ensure the result's order.
  ///
  /// ```js
  /// // a.js
  /// import c from './c.js';
  /// // b.js
  /// import c from './c.js';
  /// ```
  /// dependents("./c.js") return `['module a', 'module b']`, ensure the order of original imports.
  pub fn dependents(&self, module_id: &ModuleId) -> Vec<(ModuleId, &ModuleGraphEdge)> {
    let i = self
      .id_index_map
      .get(module_id)
      .unwrap_or_else(|| panic!("module_id {module_id:?} should in the module graph"));
    let mut edges = self
      .g
      .neighbors_directed(*i, EdgeDirection::Incoming)
      .detach();

    let mut deps = vec![];

    while let Some((edge_index, node_index)) = edges.next(&self.g) {
      deps.push((self.g[node_index].id.clone(), &self.g[edge_index]));
    }

    deps
  }

  pub fn dependents_ids(&self, module_id: &ModuleId) -> Vec<ModuleId> {
    self
      .dependents(module_id)
      .into_iter()
      .map(|(id, _)| id)
      .collect()
  }

  /// sort the module graph topologically using post order dfs, note this topo sort also keeps the original import order.
  /// return (topologically sorted modules, cyclic modules stack)
  ///
  /// **Unsupported Situation**: if the two input entries depend on the same dependencies but the import order is not the same, may cause one entry don't keep original import order, this may bring problems in css as css depends on the order.
  /// for example:
  /// ```js
  /// // entry input a.js
  /// import c from './c.js';
  /// import d from './d.js';
  ///
  /// // entry input b.js
  /// import d from './d.js';
  /// import c from './c.js';
  /// ```
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
      if let Some(pos) = stack.iter().position(|m| m == entry) {
        cyclic.push(stack.clone()[pos..].to_vec());
        return;
      } else if visited.contains(entry) {
        // skip visited module
        return;
      }

      visited.insert(entry.clone());
      stack.push(entry.clone());

      let deps = graph.dependencies(entry);

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

    let mut visited = HashSet::default();

    for (entry, _) in entries {
      let mut res = vec![];
      dfs(entry, self, &mut stack, &mut visited, &mut res, &mut cyclic);

      result.extend(res);
    }

    result.reverse();

    (result, cyclic)
  }

  pub fn update_execution_order_for_modules(&mut self) -> Vec<ModuleId> {
    let (mut topo_sorted_modules, circles) = self.toposort();

    topo_sorted_modules.reverse();

    topo_sorted_modules
      .iter()
      .enumerate()
      .for_each(|(order, module_id)| {
        let module = self.module_mut(module_id).unwrap();
        module.execution_order = order;
      });

    self.circle_record = CircleRecord::new(circles);

    topo_sorted_modules
  }

  pub fn internal_graph(&self) -> &StableDiGraph<Module, ModuleGraphEdge> {
    &self.g
  }

  pub fn dfs(&self, entry: &ModuleId, op: &mut dyn FnMut(&ModuleId)) {
    let mut dfs = Dfs::new(&self.g, *self.id_index_map.get(entry).unwrap());

    while let Some(node_index) = dfs.next(&self.g) {
      op(&self.g[node_index].id);
    }
  }

  pub fn bfs(&self, entry: &ModuleId, op: &mut dyn FnMut(&ModuleId)) {
    let mut bfs = Bfs::new(&self.g, *self.id_index_map.get(entry).unwrap());

    while let Some(node_index) = bfs.next(&self.g) {
      op(&self.g[node_index].id);
    }
  }

  pub fn take_edge_and_module(
    &mut self,
    from: &ModuleId,
    to: &ModuleId,
  ) -> (ModuleGraphEdge, Module) {
    let i = self
      .id_index_map
      .remove(to)
      .unwrap_or_else(|| panic!("module_id {to:?} should in the module graph"));
    let edge_index = self
      .g
      .find_edge(*self.id_index_map.get(from).unwrap(), i)
      .unwrap_or_else(|| panic!("edge {from:?} -> {to:?} should in the module graph"));

    let edge = self.g.remove_edge(edge_index).unwrap();
    let module = self.g.remove_node(i).unwrap();
    (edge, module)
  }

  pub fn take_module(&mut self, module_id: &ModuleId) -> Module {
    self.remove_module(module_id)
  }

  pub fn replace_module(&mut self, module: Module) {
    let i = self
      .id_index_map
      .get(&module.id)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", module.id));
    self.g[*i] = module;
  }

  pub fn is_dynamic_import(&self, module_id: &ModuleId) -> bool {
    self
      .dependents(module_id)
      .iter()
      .any(|(_, edge)| edge.is_dynamic_import())
  }

  pub fn copy_to(&self, other: &mut Self, overwrite: bool) -> Result<()> {
    let mut new_modules = Vec::<ModuleId>::new();
    for module in self.modules() {
      if overwrite || !other.has_module(&module.id) {
        other.add_module(module.clone());
        new_modules.push(module.id.clone());
      }
    }

    for module_id in &new_modules {
      for (dep, edge) in self.dependencies(module_id) {
        other.add_edge(module_id, &dep, edge.clone())?;
      }
    }

    Ok(())
  }
}

impl Default for ModuleGraph {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use crate::HashMap;

  use crate::{
    module::{Module, ModuleId},
    plugin::ResolveKind,
  };

  use super::{ModuleGraph, ModuleGraphEdge, ModuleGraphEdgeDataItem};

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
        .add_edge_item(
          &from.into(),
          &to.into(),
          ModuleGraphEdgeDataItem {
            source: format!("./{to}"),
            kind: ResolveKind::Import,
            order,
          },
        )
        .unwrap();
    }

    for (from, to, order) in dynamic_edges {
      graph
        .add_edge_item(
          &from.into(),
          &to.into(),
          ModuleGraphEdgeDataItem {
            source: format!("./{to}"),
            kind: ResolveKind::DynamicImport,
            order,
          },
        )
        .unwrap();
    }

    graph
      .add_edge_item(
        &"F".into(),
        &"A".into(),
        ModuleGraphEdgeDataItem {
          source: "./F".to_string(),
          kind: ResolveKind::Import,
          order: 0,
        },
      )
      .unwrap();

    graph.entries =
      HashMap::from_iter([("A".into(), "A".to_string()), ("B".into(), "B".to_string())]);

    graph
  }

  #[test]
  fn toposort() {
    let graph = construct_test_module_graph();
    let (sorted, cycle) = graph.toposort();

    assert_eq!(cycle, vec![vec!["A".into(), "C".into(), "F".into()],]);
    assert_eq!(
      sorted,
      vec!["B", "E", "G", "A", "D", "C", "F"]
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<ModuleId>>()
    );
  }

  #[test]
  fn dependencies() {
    let graph = construct_test_module_graph();

    let deps = graph.dependencies(&"A".into());
    assert_eq!(
      deps,
      vec![
        (
          "C".into(),
          &ModuleGraphEdge(vec![ModuleGraphEdgeDataItem {
            source: "./C".to_string(),
            kind: ResolveKind::Import,
            order: 0
          }])
        ),
        (
          "D".into(),
          &ModuleGraphEdge(vec![ModuleGraphEdgeDataItem {
            source: "./D".to_string(),
            kind: ResolveKind::DynamicImport,
            order: 1
          }])
        ),
      ]
    );
  }

  #[test]
  fn dependents() {
    let graph = construct_test_module_graph();

    let deps = graph.dependents(&"F".into());
    assert_eq!(
      deps.into_iter().map(|dep| dep.0).collect::<Vec<ModuleId>>(),
      vec!["D".into(), "C".into(),]
    );
  }

  #[test]
  fn remove_module() {
    let mut graph = construct_test_module_graph();

    graph.remove_module(&"A".into());
    assert!(!graph.has_module(&"A".into()));
  }

  #[test]
  fn has_edge() {
    let mut graph = construct_test_module_graph();

    graph.remove_module(&"A".into());
    assert!(!graph.has_edge(&"A".into(), &"D".into()));
    assert!(graph.has_edge(&"B".into(), &"D".into()));
  }
}
