use hashbrown::HashMap;

use petgraph::{graph::NodeIndex, stable_graph::StableDiGraph};

use super::resource_pot::{ResourcePot, ResourcePotId};

pub struct ResourcePotGraphEdge {}

pub struct ResourcePotGraph {
  g: StableDiGraph<ResourcePot, ResourcePotGraphEdge>,
  name_index_map: HashMap<ResourcePotId, NodeIndex>,
}

impl ResourcePotGraph {
  pub fn new() -> Self {
    Self {
      g: StableDiGraph::new(),
      name_index_map: HashMap::new(),
    }
  }

  /// replace current graph's content with other [ResourcePotGraph]'s content
  pub fn replace(&mut self, other: ResourcePotGraph) {
    self.g = other.g;
    self.name_index_map = other.name_index_map;
  }

  pub fn add_resource_pot(&mut self, resource: ResourcePot) {
    let name = resource.id.clone();
    let index = self.g.add_node(resource);
    self.name_index_map.insert(name, index);
  }

  pub fn resource_pots(&self) -> Vec<&ResourcePot> {
    self.g.node_weights().into_iter().collect()
  }

  pub fn resource_pots_mut(&mut self) -> Vec<&mut ResourcePot> {
    self.g.node_weights_mut().into_iter().collect()
  }
}

impl Default for ResourcePotGraph {
  fn default() -> Self {
    Self::new()
  }
}
