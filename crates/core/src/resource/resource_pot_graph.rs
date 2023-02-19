use hashbrown::HashMap;

use petgraph::{graph::NodeIndex, stable_graph::StableDiGraph};

use super::resource_pot::{ResourcePot, ResourcePotId};

pub struct ResourcePotGraphEdge {}

pub struct ResourcePotGraph {
  g: StableDiGraph<ResourcePot, ResourcePotGraphEdge>,
  id_index_map: HashMap<ResourcePotId, NodeIndex>,
}

impl ResourcePotGraph {
  pub fn new() -> Self {
    Self {
      g: StableDiGraph::new(),
      id_index_map: HashMap::new(),
    }
  }

  /// replace current graph's content with other [ResourcePotGraph]'s content
  pub fn replace(&mut self, other: ResourcePotGraph) {
    self.g = other.g;
    self.id_index_map = other.id_index_map;
  }

  pub fn resource_pot(&self, id: &ResourcePotId) -> Option<&ResourcePot> {
    let id = self.id_index_map.get(id).unwrap_or_else(|| {
      panic!(
        "ResourcePotGraph::resource_pot: id not found: {}",
        id.to_string()
      )
    });
    self.g.node_weight(*id)
  }

  pub fn resource_pot_mut(&mut self, id: &ResourcePotId) -> Option<&mut ResourcePot> {
    let id = self.id_index_map.get(id).unwrap();
    self.g.node_weight_mut(*id)
  }

  pub fn add_resource_pot(&mut self, resource: ResourcePot) {
    let name = resource.id.clone();
    let index = self.g.add_node(resource);
    self.id_index_map.insert(name, index);
  }

  pub fn remove_resource_pot(&mut self, id: &ResourcePotId) -> Option<ResourcePot> {
    if let Some(ndx) = self.id_index_map.get(id) {
      let resource = self.g.remove_node(*ndx);
      self.id_index_map.remove(id);
      resource
    } else {
      None
    }
  }

  pub fn add_edge(&mut self, from: &ResourcePotId, to: &ResourcePotId) {
    let from = self.id_index_map.get(from).unwrap();
    let to = self.id_index_map.get(to).unwrap();
    self.g.add_edge(*from, *to, ResourcePotGraphEdge {});
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
