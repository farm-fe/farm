use std::collections::HashMap;

use petgraph::{graph::NodeIndex, stable_graph::StableDiGraph};

use super::Resource;

pub struct ResourceGraphEdge {}

pub struct ResourceGraph {
  g: StableDiGraph<Resource, ResourceGraphEdge>,
  name_index_map: HashMap<String, NodeIndex>,
}

impl ResourceGraph {
  pub fn new() -> Self {
    Self {
      g: StableDiGraph::new(),
      name_index_map: HashMap::new(),
    }
  }

  pub fn add_resource(&mut self, resource: Resource) {
    let name = resource.name.clone();
    let index = self.g.add_node(resource);
    self.name_index_map.insert(name, index);
  }

  pub fn resources(&self) -> Vec<&Resource> {
    self.g.node_weights().into_iter().collect()
  }

  pub fn resources_mut(&mut self) -> Vec<&mut Resource> {
    self.g.node_weights_mut().into_iter().collect()
  }
}

impl Default for ResourceGraph {
  fn default() -> Self {
    Self::new()
  }
}
