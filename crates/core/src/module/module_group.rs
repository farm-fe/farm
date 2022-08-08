use hashbrown::HashMap;

use super::ModuleId;

/// A `entry_module_id -> ModuleGroup` map
pub struct ModuleGroupMap {
  groups: HashMap<ModuleGroupId, ModuleGroup>,
}

impl ModuleGroupMap {
  pub fn new() -> Self {
    Self {
      groups: HashMap::new(),
    }
  }

  pub fn add_module_group(&mut self, module_group: ModuleGroup) {
    self.groups.insert(module_group.id.clone(), module_group);
  }

  pub fn module_group(&self, id: &ModuleGroupId) -> Option<&ModuleGroup> {
    self.groups.get(id)
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

pub struct ModuleGroup {
  /// the module group's id is the same as its entry module's id.
  pub id: ModuleGroupId,
  /// the modules that this group has
  modules: Vec<ModuleId>,
}

impl ModuleGroup {
  pub fn new(id: ModuleGroupId) -> Self {
    Self {
      modules: vec![id.clone()],
      id,
    }
  }

  pub fn add_module(&mut self, module_id: ModuleId) {
    self.modules.push(module_id);
  }

  pub fn modules(&self) -> &Vec<ModuleId> {
    &self.modules
  }
}
