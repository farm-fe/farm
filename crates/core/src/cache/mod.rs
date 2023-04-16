use hashbrown::HashMap;
use parking_lot::Mutex;

use crate::{module::ModuleId, plugin::PluginResolveHookResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolveResultCacheKey {
  pub source: String,
  pub importer: Option<ModuleId>,
}

/// All cache related operation are charged by [CacheManager]
pub struct CacheManager {
  pub resolve_result_cache: Mutex<HashMap<ResolveResultCacheKey, PluginResolveHookResult>>,
}

impl CacheManager {
  pub fn new() -> Self {
    Self {
      resolve_result_cache: Mutex::new(HashMap::new()),
    }
  }

  pub fn get_resolve_result_cache_by_key(
    &self,
    key: &ResolveResultCacheKey,
  ) -> Option<PluginResolveHookResult> {
    self.resolve_result_cache.lock().get(key).cloned()
  }

  pub fn set_resolve_result_cache_by_key(
    &self,
    key: ResolveResultCacheKey,
    value: PluginResolveHookResult,
  ) {
    self.resolve_result_cache.lock().insert(key, value);
  }
}

impl Default for CacheManager {
  fn default() -> Self {
    Self::new()
  }
}
