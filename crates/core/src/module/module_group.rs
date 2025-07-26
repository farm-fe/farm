use farmfe_macro_cache_item::cache_item;
use petgraph::{
  stable_graph::{DefaultIx, NodeIndex, StableDiGraph},
  visit::{Bfs, Dfs, DfsPostOrder, EdgeRef, IntoEdgeReferences},
};

use crate::resource::{resource_pot::ResourcePotId, resource_pot_map::ResourcePotMap};
use crate::{HashMap, HashSet};

use super::{module_graph::ModuleGraph, ModuleId};

/// A `entry_module_id -> ModuleGroup` map
#[derive(Debug)]
pub struct ModuleGroupGraph {
  /// internal graph
  g: StableDiGraph<ModuleGroup, ()>,
  /// to index module in the graph using [ModuleId]
  id_index_map: HashMap<ModuleGroupId, NodeIndex<DefaultIx>>,
}

impl ModuleGroupGraph {
  pub fn new() -> Self {
    Self {
      g: StableDiGraph::new(),
      id_index_map: HashMap::default(),
    }
  }

  pub fn replace(&mut self, other: ModuleGroupGraph) {
    self.g = other.g;
    self.id_index_map = other.id_index_map;
  }

  pub fn add_module_group(&mut self, module_group: ModuleGroup) {
    let module_group_id = module_group.id.clone();

    if self.has(&module_group_id) {
      panic!("module group already exists: {module_group_id:?}");
    }

    let node_index = self.g.add_node(module_group);
    self.id_index_map.insert(module_group_id, node_index);
  }

  pub fn add_edge(&mut self, from: &ModuleGroupId, to: &ModuleGroupId) {
    let from_node_index = self.id_index_map.get(from).unwrap();
    let to_node_index = self.id_index_map.get(to).unwrap();

    self.g.add_edge(*from_node_index, *to_node_index, ());
  }

  pub fn remove_edge(&mut self, from: &ModuleGroupId, to: &ModuleGroupId) {
    let from_node_index = self.id_index_map.get(from).unwrap_or_else(|| {
      panic!(
        "ModuleGroupGraph::remove_edge: from {} to {}. Not found: {}",
        from.to_string(),
        to.to_string(),
        from.to_string()
      )
    });
    let to_node_index = self.id_index_map.get(to).unwrap_or_else(|| {
      panic!(
        "ModuleGroupGraph::remove_edge: from {} to {}. Not found: {}",
        from.to_string(),
        to.to_string(),
        to.to_string()
      )
    });
    let edge_index = self.g.find_edge(*from_node_index, *to_node_index).unwrap();
    self.g.remove_edge(edge_index);
  }

  pub fn remove_module_group(&mut self, id: &ModuleGroupId) -> Option<ModuleGroup> {
    let node_index = self.id_index_map.remove(id).unwrap();
    self.g.remove_node(node_index)
  }

  pub fn module_group(&self, id: &ModuleGroupId) -> Option<&ModuleGroup> {
    let node_index = self.id_index_map.get(id).unwrap();
    self.g.node_weight(*node_index)
  }

  pub fn module_group_mut(&mut self, id: &ModuleGroupId) -> Option<&mut ModuleGroup> {
    let node_index = self.id_index_map.get(id)?;

    self.g.node_weight_mut(*node_index)
  }

  /// get the topologically sorted module groups
  pub fn module_groups(&self) -> Vec<&ModuleGroup> {
    self.g.node_weights().collect()
  }

  pub fn edges(&self) -> Vec<(ModuleGroupId, ModuleGroupId)> {
    self
      .g
      .edge_references()
      .map(|edge| {
        let source = self.g[edge.source()].id.clone();
        let target = self.g[edge.target()].id.clone();

        (source, target)
      })
      .collect()
  }

  /// the same as [ModuleGroupGraph::module_groups], but mutable.
  pub fn module_groups_mut(&mut self) -> Vec<&mut ModuleGroup> {
    self.g.node_weights_mut().collect()
  }

  pub fn has(&self, id: &ModuleGroupId) -> bool {
    self.id_index_map.contains_key(id)
  }

  pub fn has_edge(&self, from: &ModuleGroupId, to: &ModuleGroupId) -> bool {
    let from_node_index = self.id_index_map.get(from);
    let to_node_index = self.id_index_map.get(to);

    if from_node_index.is_none() || to_node_index.is_none() {
      return false;
    }

    self
      .g
      .find_edge(*from_node_index.unwrap(), *to_node_index.unwrap())
      .is_some()
  }

  pub fn len(&self) -> usize {
    self.g.node_count()
  }

  pub fn is_empty(&self) -> bool {
    self.g.node_count() == 0
  }

  pub fn dfs(&self, entry: &ModuleGroupId, op: &mut dyn FnMut(&ModuleGroupId)) {
    let mut dfs = Dfs::new(&self.g, *self.id_index_map.get(entry).unwrap());

    while let Some(node_index) = dfs.next(&self.g) {
      op(&self.g[node_index].id);
    }
  }

  pub fn dfs_post_order(&self, entry: &ModuleGroupId, op: &mut dyn FnMut(&ModuleGroupId)) {
    let mut dfs = DfsPostOrder::new(&self.g, *self.id_index_map.get(entry).unwrap());

    while let Some(node_index) = dfs.next(&self.g) {
      op(&self.g[node_index].id);
    }
  }

  pub fn bfs(&self, entry: &ModuleGroupId, op: &mut dyn FnMut(&ModuleGroupId)) {
    let mut bfs = Bfs::new(&self.g, *self.id_index_map.get(entry).unwrap());

    while let Some(node_index) = bfs.next(&self.g) {
      op(&self.g[node_index].id);
    }
  }

  pub fn dependencies(&self, module_id: &ModuleGroupId) -> Vec<&ModuleGroup> {
    let node_index = self.id_index_map.get(module_id).unwrap();
    let mut dependencies = Vec::new();

    for edge in self.g.edges(*node_index) {
      dependencies.push(&self.g[edge.target()]);
    }

    dependencies
  }

  pub fn dependencies_ids(&self, module_id: &ModuleGroupId) -> Vec<ModuleGroupId> {
    let node_index = self.id_index_map.get(module_id).unwrap();
    let mut dependencies = Vec::new();

    for edge in self.g.edges(*node_index) {
      dependencies.push(self.g[edge.target()].id.clone());
    }

    dependencies
  }

  pub fn dependents(&self, module_id: &ModuleGroupId) -> Vec<&ModuleGroup> {
    let node_index = self.id_index_map.get(module_id).unwrap();
    let mut dependents = Vec::new();

    for edge in self
      .g
      .edges_directed(*node_index, petgraph::Direction::Incoming)
    {
      dependents.push(&self.g[edge.source()]);
    }

    dependents
  }

  pub fn toposort(&self, entries: Vec<ModuleGroupId>) -> Vec<ModuleGroupId> {
    let mut sorted = Vec::new();
    let mut visited = HashSet::default();

    for entry in entries {
      self.dfs_post_order(&entry, &mut |id| {
        if !visited.contains(id) {
          sorted.push(id.clone());
          visited.insert(id.clone());
        }
      });
    }

    sorted.reverse();
    sorted
  }

  pub fn print_graph(&self) -> PrintedModuleGroupGraph {
    let mut graph = PrintedModuleGroupGraph {
      module_groups: Vec::new(),
      edges: Vec::new(),
    };

    for module_group in self.module_groups() {
      graph.module_groups.push(module_group.clone());
    }

    for edge in self.g.edge_references() {
      let source = self.g[edge.source()].id.to_string();
      let target = self.g[edge.target()].id.to_string();
      graph.edges.push((source, target));
    }

    graph
  }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrintedModuleGroupGraph {
  pub module_groups: Vec<ModuleGroup>,
  pub edges: Vec<(String, String)>,
}

impl Default for ModuleGroupGraph {
  fn default() -> Self {
    Self::new()
  }
}

impl PartialEq for ModuleGroupGraph {
  fn eq(&self, other: &Self) -> bool {
    let mut self_module_groups = self.module_groups();
    self_module_groups.sort_by_key(|g| g.id.clone());

    let mut other_module_groups = other.module_groups();
    other_module_groups.sort_by_key(|g| g.id.clone());

    let mut self_edges = self
      .g
      .edge_references()
      .map(|e| {
        let source = self.g[e.source()].id.clone();
        let target = self.g[e.target()].id.clone();

        (source, target)
      })
      .collect::<Vec<_>>();
    self_edges.sort();

    let mut other_edges = other
      .g
      .edge_references()
      .map(|e| {
        let source = other.g[e.source()].id.clone();
        let target = other.g[e.target()].id.clone();

        (source, target)
      })
      .collect::<Vec<_>>();
    other_edges.sort();

    // println!("self_module_groups: {:#?}", self_module_groups);
    // println!("other_module_groups: {:#?}", other_module_groups);
    // println!("self_edges: {:#?}", self_edges);
    // println!("other_edges: {:#?}", other_edges);
    // println!(
    //   "self_module_groups == other_module_groups: {}",
    //   self_module_groups == other_module_groups
    // );
    // println!("self_edges == other_edges: {}", self_edges == other_edges);

    self_module_groups == other_module_groups && self_edges == other_edges
  }
}

impl Eq for ModuleGroupGraph {}

#[cache_item]
#[derive(
  Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, serde::Serialize, serde::Deserialize,
)]
#[rkyv(derive(Hash, Eq, PartialEq))]
pub struct ModuleGroupId(String);

impl ModuleGroupId {
  pub fn new(id: &ModuleId, ty: &ModuleGroupType) -> Self {
    Self(format!("{}_{:?}", id.to_string(), ty))
  }
}

impl ToString for ModuleGroupId {
  fn to_string(&self) -> String {
    self.0.clone()
  }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleGroup {
  pub id: ModuleGroupId,
  /// the module group's id is the same as its entry module's id.
  pub entry_module_id: ModuleId,
  pub module_group_type: ModuleGroupType,
  /// the modules that this group has
  modules: HashSet<ModuleId>,
  /// the [ResourcePot]s this group merged to
  resource_pots: HashSet<ResourcePotId>,
}

impl ModuleGroup {
  pub fn new(module_id: ModuleId, ty: ModuleGroupType) -> Self {
    Self {
      id: ModuleGroupId::new(&module_id, &ty),
      modules: HashSet::from_iter([module_id.clone()]),
      entry_module_id: module_id,
      module_group_type: ty,
      resource_pots: HashSet::default(),
    }
  }

  pub fn add_module(&mut self, module_id: ModuleId) {
    self.modules.insert(module_id);
  }

  pub fn remove_module(&mut self, module_id: &ModuleId) {
    self.modules.retain(|id| id != module_id);
  }

  pub fn modules(&self) -> &HashSet<ModuleId> {
    &self.modules
  }

  pub fn add_resource_pot(&mut self, resource_pot_id: ResourcePotId) {
    self.resource_pots.insert(resource_pot_id);
  }

  pub fn remove_resource_pot(&mut self, resource_pot_id: &ResourcePotId) {
    self.resource_pots.retain(|id| id != resource_pot_id);
  }

  pub fn resource_pots(&self) -> &HashSet<ResourcePotId> {
    &self.resource_pots
  }

  pub fn sorted_resource_pots(
    &self,
    module_graph: &ModuleGraph,
    resource_pot_map: &ResourcePotMap,
  ) -> Vec<ResourcePotId> {
    let mut resource_pots_order_map = HashMap::<String, usize>::default();
    let mut sorted_resource_pots = self.resource_pots().iter().cloned().collect::<Vec<_>>();

    sorted_resource_pots.iter().for_each(|rp| {
      let rp = resource_pot_map.resource_pot(rp).unwrap();
      let min_order = rp
        .modules()
        .iter()
        .map(|m| {
          let module = module_graph.module(m).unwrap();
          module.execution_order
        })
        .min()
        .unwrap_or(0);

      resource_pots_order_map.insert(rp.id.to_string(), min_order);
    });

    sorted_resource_pots.sort_by(|a, b| {
      let a_order = resource_pots_order_map.get(a).unwrap_or(&0);
      let b_order = resource_pots_order_map.get(b).unwrap_or(&0);

      a_order.cmp(b_order)
    });

    sorted_resource_pots
  }

  pub fn set_resource_pots(&mut self, resource_pots: HashSet<ResourcePotId>) {
    self.resource_pots = resource_pots;
  }

  pub fn has_resource_pot(&self, resource_pot_id: &ResourcePotId) -> bool {
    self.resource_pots.contains(resource_pot_id)
  }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum ModuleGroupType {
  /// module group created from config.input
  Entry,
  /// module group created from dynamic input return by analyze_deps or finalize_module hook. e.g. ResolveKind::DynamicEntry { name: "a", output_filename: None }
  DynamicEntry,
  /// module group created from dynamic import. e.g. `import('./a').then(module => console.log(module))`
  DynamicImport,
}
