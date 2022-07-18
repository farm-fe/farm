use dashmap::DashMap;

use super::ModuleId;

/// A entry_module_id -> ModuleGroup map
pub struct ModuleGroupMap {
  groups: DashMap<ModuleId, ModuleGroup>,
}

impl ModuleGroupMap {
  pub fn new() -> Self {
    Self {
      groups: DashMap::new(),
    }
  }
}

impl Default for ModuleGroupMap {
  fn default() -> Self {
    Self::new()
  }
}

pub struct ModuleGroup {
  entry_module: ModuleId,
}
