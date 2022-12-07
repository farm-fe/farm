use std::collections::HashSet;

use parking_lot::Mutex;

/// All cache related operation are charged by [CacheManager]
pub struct CacheManager {
  /// the modules which are already handled, a module(the resolved id is same) won't be processed twice
  handled_modules: Mutex<HashSet<String>>,
}

impl CacheManager {
  pub fn new() -> Self {
    Self {
      handled_modules: Mutex::new(HashSet::new()),
    }
  }

  pub fn mark_module_handled(&self, id: &str) {
    let mut hm = self.handled_modules.lock();
    hm.insert(id.to_string());
  }

  pub fn is_module_handled(&self, id: &str) -> bool {
    let hm = self.handled_modules.lock();
    hm.contains(id)
  }
}

impl Default for CacheManager {
  fn default() -> Self {
    Self::new()
  }
}
