use hashbrown::{HashMap, HashSet};
use petgraph::{
  stable_graph::{DefaultIx, NodeIndex, StableDiGraph},
  visit::{Bfs, Dfs},
};

use crate::resource::resource_pot::ResourcePotId;

use super::ModuleId;

/// A `entry_module_id -> ModuleGroup` map
#[derive(Debug)]
pub struct ModuleGroupGraph {
  /// internal graph
  g: StableDiGraph<ModuleGroup, ()>,
  /// to index module in the graph using [ModuleId]
  id_index_map: HashMap<ModuleId, NodeIndex<DefaultIx>>,
}

impl ModuleGroupGraph {
  pub fn new() -> Self {
    Self {
      g: StableDiGraph::new(),
      id_index_map: HashMap::new(),
    }
  }

  pub fn replace(&mut self, other: ModuleGroupGraph) {
    self.g = other.g;
    self.id_index_map = other.id_index_map;
  }

  pub fn add_module_group(&mut self, module_group: ModuleGroup) {
    let module_group_id = module_group.id.clone();
    let node_index = self.g.add_node(module_group);
    self
      .id_index_map
      .insert(module_group_id.clone(), node_index);
  }

  pub fn add_edge(&mut self, from: &ModuleId, to: &ModuleId) {
    let from_node_index = self.id_index_map.get(from).unwrap();
    let to_node_index = self.id_index_map.get(to).unwrap();
    self.g.add_edge(*from_node_index, *to_node_index, ());
  }

  pub fn remove_edge(&mut self, from: &ModuleId, to: &ModuleId) {
    let from_node_index = self.id_index_map.get(from).unwrap();
    let to_node_index = self.id_index_map.get(to).unwrap();
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
    let node_index = self.id_index_map.get(id).unwrap();
    self.g.node_weight_mut(*node_index)
  }

  /// get the topologically sorted module groups
  pub fn module_groups(&self) -> Vec<&ModuleGroup> {
    self.g.node_weights().collect()
  }

  /// the same as [ModuleGroupGraph::module_groups], but mutable.
  pub fn module_groups_mut(&mut self) -> Vec<&mut ModuleGroup> {
    self.g.node_weights_mut().collect()
  }

  pub fn has(&self, id: &ModuleGroupId) -> bool {
    self.id_index_map.contains_key(id)
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

  pub fn bfs(&self, entry: &ModuleGroupId, op: &mut dyn FnMut(&ModuleGroupId)) {
    let mut bfs = Bfs::new(&self.g, *self.id_index_map.get(entry).unwrap());

    while let Some(node_index) = bfs.next(&self.g) {
      op(&self.g[node_index].id);
    }
  }
}

impl Default for ModuleGroupGraph {
  fn default() -> Self {
    Self::new()
  }
}

impl PartialEq for ModuleGroupGraph {
  fn eq(&self, other: &Self) -> bool {
    self
      .module_groups()
      .iter()
      .all(|module_group| other.module_groups().contains(module_group))
      && other
        .module_groups()
        .iter()
        .all(|module_group| self.module_groups().contains(module_group))
  }
}

impl Eq for ModuleGroupGraph {}

pub type ModuleGroupId = ModuleId;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModuleGroup {
  /// the module group's id is the same as its entry module's id.
  pub id: ModuleGroupId,
  /// the modules that this group has
  modules: HashSet<ModuleId>,
  /// the [ResourcePot]s this group merged to
  resource_pots: HashSet<ResourcePotId>,
}

impl ModuleGroup {
  pub fn new(id: ModuleGroupId) -> Self {
    Self {
      modules: HashSet::from([id.clone()]),
      id,
      resource_pots: HashSet::new(),
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

  pub fn resource_pots(&self) -> &HashSet<ResourcePotId> {
    &self.resource_pots
  }

  pub fn set_resource_pots(&mut self, resource_pots: HashSet<ResourcePotId>) {
    self.resource_pots = resource_pots;
  }
}
