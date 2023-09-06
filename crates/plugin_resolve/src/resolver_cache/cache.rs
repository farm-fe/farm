// use crate::resolver::ResolveNodeModuleCacheKey;
use farmfe_core::plugin::{PluginResolveHookResult, ResolveKind};
use std::collections::HashMap;
use std::sync::Mutex;
pub struct ResolveCache {
  cache: Mutex<HashMap<ResolveNodeModuleCacheKey, Option<PluginResolveHookResult>>>,
}
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ResolveNodeModuleCacheKey {
  pub source: String,
  pub base_dir: String,
  pub kind: ResolveKind,
}

impl ResolveCache {
  pub fn new() -> ResolveCache {
    // Whether it is necessary to distinguish the cache of different modules
    // qs: Whether the resolver module only needs to be instantiated once ？
    // Caching only three cases: the first alias, the second absolute path, the third node_modules third party path
    ResolveCache {
      // the first alias, the second absolute path, the third node_modules third party path
      cache: Mutex::new(HashMap::new()),
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

  pub fn contains(&self, key: &ResolveNodeModuleCacheKey) -> bool {
    // Lock the mutex and access the cache
    let cache_contains_key = self.cache.lock().unwrap();
    cache_contains_key.contains_key(key)
  }
}
