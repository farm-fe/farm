use crate::resolver::ResolveNodeModuleCacheKey;
use farmfe_core::plugin::PluginResolveHookResult;
use std::collections::HashMap;
use std::sync::Mutex;
pub struct ResolveCache {
  cache: Mutex<HashMap<ResolveNodeModuleCacheKey, Option<PluginResolveHookResult>>>,
}

impl ResolveCache {
  pub fn new() -> ResolveCache {
    // Whether it is necessary to distinguish the cache of different modules
    // qs: Whether the resolver module only needs to be instantiated once ？
    ResolveCache {
      cache: Mutex::new(HashMap::new()),
      // resolve_node_modules_cache: Mutex::new(HashMap::new()),
    }
  }

  pub fn insert(&self, key: ResolveNodeModuleCacheKey, value: Option<PluginResolveHookResult>) {
    let mut cache = self.cache.lock().unwrap();
    cache.insert(key, value);
  }

  pub fn get(&self, key: &ResolveNodeModuleCacheKey) -> Option<Option<PluginResolveHookResult>> {
    let cache = self.cache.lock().unwrap();
    cache.get(key).cloned()
  }
}
