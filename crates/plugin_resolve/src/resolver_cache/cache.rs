use farmfe_core::plugin::{PluginResolveHookResult, ResolveKind};
use std::collections::HashMap;
use std::sync::{PoisonError, RwLock};

// define a struct `ResolveCache` used to cache parser result
pub struct ResolveCache {
  //  use `RwLock` to protect the cache, allowing multiple threads to read at the same time, but exclusive when writing
  cache: RwLock<HashMap<ResolveNodeModuleCacheKey, Option<PluginResolveHookResult>>>,
}

#[derive(Debug)]
pub enum CacheError {
  LockError(String),
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
      // use `RwLock` to wrap the hash map to initialize the cache
      // the first alias, the second absolute path, the third node_modules third party path
      cache: RwLock::new(HashMap::new()),
    }
  }

  // insert fn use `entry` Method to avoid locking the cache multiple times. This can reduce lock competition and improve performance.
  pub fn insert(
    &self,
    key: ResolveNodeModuleCacheKey,
    value: Option<PluginResolveHookResult>,
  ) -> Result<(), CacheError> {
    let mut cache = self.cache.write().map_err(|e| {
      self.handle_cache_error(
        e,
        "resolver Cache Module When get cache module data Failed to lock cache:",
      )
    })?;
    cache.entry(key).or_insert(value);
    Ok(())
  }

  pub fn get(
    &self,
    key: &ResolveNodeModuleCacheKey,
  ) -> Result<Option<PluginResolveHookResult>, CacheError> {
    let cache = self.cache.read().map_err(|e| {
      self.handle_cache_error(
        e,
        "resolver Cache Module When get cache module data Failed to lock cache:",
      )
    })?;
    match cache.get(key) {
      Some(result) => Ok(result.clone()),
      None => Ok(None),
    }
  }

  // Check whether the cache contains the specified key, and return a `Option` indicating the success or failure of the operation.
  pub fn contains(&self, key: &ResolveNodeModuleCacheKey) -> Result<bool, CacheError> {
    let cache_contains_key = self.cache.read().map_err(|e| {
      self.handle_cache_error(
        e,
        "resolver Cache Module When contains keys module data Failed to lock cache:",
      )
    })?;
    Ok(cache_contains_key.contains_key(key))
  }

  fn handle_cache_error<T, S, E>(&self, error: PoisonError<E>, context: S) -> CacheError
  where
    S: Into<String>,
    E: std::ops::Deref<Target = T>,
  {
    let error_msg = format!("{}: Failed to lock cache: {:?}", context.into(), error);
    CacheError::LockError(error_msg)
  }
}
