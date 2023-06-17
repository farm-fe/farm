use hashbrown::HashMap;

use super::resource_pot::{ResourcePot, ResourcePotId};

// TODO: Change Graph to Map, as the resource pot should be related to module group, and we already have resource group graph
pub struct ResourcePotMap {
  map: HashMap<ResourcePotId, ResourcePot>,
}

impl ResourcePotMap {
  pub fn new() -> Self {
    Self {
      map: HashMap::new(),
    }
  }

  /// replace current graph's content with other [ResourcePotMap]'s content
  pub fn replace(&mut self, other: ResourcePotMap) {
    self.map = other.map;
  }

  pub fn resource_pot(&self, id: &ResourcePotId) -> Option<&ResourcePot> {
    self.map.get(id)
  }

  pub fn resource_pot_mut(&mut self, id: &ResourcePotId) -> Option<&mut ResourcePot> {
    self.map.get_mut(id)
  }

  pub fn add_resource_pot(&mut self, resource: ResourcePot) {
    self.map.insert(resource.id.clone(), resource);
  }

  pub fn remove_resource_pot(&mut self, id: &ResourcePotId) -> Option<ResourcePot> {
    self.map.remove(id)
  }

  pub fn resource_pots(&self) -> Vec<&ResourcePot> {
    self.map.values().collect()
  }

  pub fn resource_pots_mut(&mut self) -> Vec<&mut ResourcePot> {
    self.map.values_mut().collect()
  }

  pub fn has_resource_pot(&self, id: &ResourcePotId) -> bool {
    self.map.contains_key(id)
  }
}

impl Default for ResourcePotMap {
  fn default() -> Self {
    Self::new()
  }
}
