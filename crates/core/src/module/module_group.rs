use std::collections::{HashMap, HashSet};

use petgraph::{stable_graph::{DefaultIx, NodeIndex, StableDiGraph}, visit::EdgeRef};
use serde::{Deserialize, Serialize};

use crate::resource::{resource_pot::ResourcePotId, resource_pot_map::ResourcePotMap};
use super::{module_graph::ModuleGraph, ModuleId};

#[derive(Debug)]
pub struct ModuleGroupGraph {
  g: StableDiGraph<ModuleGroup, ()>,
  id_index_map: HashMap<ModuleId, NodeIndex<DefaultIx>>,
}

impl ModuleGroupGraph {
  pub fn new() -> Self { Self { g: StableDiGraph::new(), id_index_map: HashMap::new() } }
  pub fn replace(&mut self, other: ModuleGroupGraph) { self.g = other.g; self.id_index_map = other.id_index_map; }
  pub fn add_module_group(&mut self, group: ModuleGroup) {
    let id = group.id.clone();
    if self.has(&id) { panic!("module group already exists: {id:?}"); }
    let index = self.g.add_node(group);
    self.id_index_map.insert(id, index);
  }
  pub fn add_edge(&mut self, from: &ModuleId, to: &ModuleId) {
    let (Some(from), Some(to)) = (self.id_index_map.get(from), self.id_index_map.get(to)) else { return; };
    self.g.add_edge(*from, *to, ());
  }
  pub fn remove_edge(&mut self, from: &ModuleId, to: &ModuleId) {
    let (Some(from), Some(to)) = (self.id_index_map.get(from), self.id_index_map.get(to)) else { return; };
    if let Some(edge) = self.g.find_edge(*from, *to) { self.g.remove_edge(edge); }
  }
  pub fn remove_module_group(&mut self, id: &ModuleGroupId) -> Option<ModuleGroup> {
    let index = self.id_index_map.remove(id)?;
    self.g.remove_node(index)
  }
  pub fn module_group(&self, id: &ModuleGroupId) -> Option<&ModuleGroup> {
    self.id_index_map.get(id).and_then(|i| self.g.node_weight(*i))
  }
  pub fn module_group_mut(&mut self, id: &ModuleGroupId) -> Option<&mut ModuleGroup> {
    self.id_index_map.get(id).and_then(|i| self.g.node_weight_mut(*i))
  }
  pub fn module_groups(&self) -> Vec<&ModuleGroup> { self.g.node_weights().collect() }
  pub fn edges(&self) -> Vec<(ModuleGroupId, ModuleGroupId)> { self.g.edge_references().map(|e| (self.g[e.source()].id.clone(), self.g[e.target()].id.clone())).collect() }
  pub fn module_groups_mut(&mut self) -> Vec<&mut ModuleGroup> { self.g.node_weights_mut().collect() }
  pub fn has(&self, id: &ModuleGroupId) -> bool { self.id_index_map.contains_key(id) }
  pub fn has_edge(&self, from: &ModuleId, to: &ModuleId) -> bool { self.id_index_map.get(from).zip(self.id_index_map.get(to)).and_then(|(a,b)| self.g.find_edge(*a,*b)).is_some() }
  pub fn len(&self) -> usize { self.g.node_count() }
  pub fn is_empty(&self) -> bool { self.g.node_count() == 0 }
}

pub type ModuleGroupId = ModuleId;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ModuleGroup { pub id: ModuleGroupId, modules: HashSet<ModuleId>, resource_pots: HashSet<ResourcePotId> }
impl ModuleGroup {
  pub fn new(id: ModuleGroupId) -> Self { Self { modules: HashSet::from([id.clone()]), id, resource_pots: HashSet::new() } }
  pub fn add_module(&mut self, id: ModuleId) { self.modules.insert(id); }
  pub fn remove_module(&mut self, id: &ModuleId) { self.modules.remove(id); }
  pub fn modules(&self) -> &HashSet<ModuleId> { &self.modules }
  pub fn add_resource_pot(&mut self, id: ResourcePotId) { self.resource_pots.insert(id); }
  pub fn remove_resource_pot(&mut self, id: &ResourcePotId) { self.resource_pots.remove(id); }
  pub fn resource_pots(&self) -> &HashSet<ResourcePotId> { &self.resource_pots }
  pub fn sorted_resource_pots(&self, module_graph: &ModuleGraph, resource_pot_map: &ResourcePotMap) -> Vec<ResourcePotId> { let mut result = self.resource_pots.iter().cloned().collect::<Vec<_>>(); result.sort_by_key(|id| resource_pot_map.resource_pot(id).and_then(|p| p.modules().iter().filter_map(|m| module_graph.module(m).map(|x| x.execution_order)).min()).unwrap_or(0)); result }
  pub fn set_resource_pots(&mut self, pots: HashSet<ResourcePotId>) { self.resource_pots = pots; }
  pub fn has_resource_pot(&self, id: &ResourcePotId) -> bool { self.resource_pots.contains(id) }
}
