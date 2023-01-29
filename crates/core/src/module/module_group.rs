use hashbrown::{HashMap, HashSet};

use crate::resource::resource_pot::ResourcePotId;

use super::ModuleId;

/// A `entry_module_id -> ModuleGroup` map
#[derive(Debug, PartialEq, Eq)]
pub struct ModuleGroupMap {
  groups: HashMap<ModuleGroupId, ModuleGroup>,
}

impl ModuleGroupMap {
  pub fn new() -> Self {
    Self {
      groups: HashMap::new(),
    }
  }

  pub fn replace(&mut self, other: ModuleGroupMap) {
    self.groups = other.groups;
  }

  pub fn add_module_group(&mut self, module_group: ModuleGroup) {
    self.groups.insert(module_group.id.clone(), module_group);
  }

  pub fn remove_module_group(&mut self, id: &ModuleGroupId) {
    self.groups.remove(id);
  }

  pub fn module_group(&self, id: &ModuleGroupId) -> Option<&ModuleGroup> {
    self.groups.get(id)
  }

  pub fn module_group_mut(&mut self, id: &ModuleGroupId) -> Option<&mut ModuleGroup> {
    self.groups.get_mut(id)
  }

  /// get the topologically sorted module groups
  pub fn module_groups(&self) -> Vec<&ModuleGroup> {
    self.groups.iter().map(|(_, v)| v).collect()
  }

  /// the same as [ModuleGroupMap::module_groups], but mutable.
  pub fn module_groups_mut(&mut self) -> Vec<&mut ModuleGroup> {
    self.groups.iter_mut().map(|(_, v)| v).collect()
  }

  pub fn has(&self, id: &ModuleGroupId) -> bool {
    self.groups.contains_key(id)
  }

  pub fn len(&self) -> usize {
    self.groups.len()
  }

  pub fn is_empty(&self) -> bool {
    self.groups.is_empty()
  }
}

impl Default for ModuleGroupMap {
  fn default() -> Self {
    Self::new()
  }
}

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
}
