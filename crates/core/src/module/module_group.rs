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

  pub fn module_group(&self, id: &ModuleGroupId) -> Option<&ModuleGroup> {
    self.groups.get(id)
  }

  /// get the topologically sorted module groups
  pub fn module_groups(&self) -> Vec<&ModuleGroup> {
    vec![]
  }

  /// the same as [ModuleGroupMap::module_groups], but mutable.
  pub fn module_groups_mut(&mut self) -> Vec<&mut ModuleGroup> {
    vec![]
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
  pub modules: Vec<ModuleId>,
}

impl ModuleGroup {
  pub fn new(id: ModuleGroupId) -> Self {
    Self {
      id,
      modules: vec![],
    }
  }

  pub fn add_module(&mut self, module_id: ModuleId) {
    self.modules.push(module_id);
  }
}
