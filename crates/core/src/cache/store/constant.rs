use dashmap::mapref::multiple::RefMulti;

use crate::HashMap;

use super::{error::CacheError, CacheStoreKey};

pub trait CacheStoreTrait {
  fn has_cache(&self, name: &str) -> bool;
  fn get_store_keys(&self) -> Vec<RefMulti<String, String>>;
  fn is_cache_changed(&self, store_key: &CacheStoreKey) -> bool;
  fn write_single_cache(&self, store_key: CacheStoreKey, bytes: Vec<u8>) -> Result<(), CacheError>;
  fn write_manifest(&self);
  fn write_cache(&self, cache_map: HashMap<CacheStoreKey, Vec<u8>>);
  fn read_cache(&self, name: &str) -> Option<Vec<u8>>;
  fn remove_cache(&self, name: &str);
}

pub const FARM_CACHE_VERSION: &str = "0.6.1";
pub const FARM_CACHE_MANIFEST_FILE: &str = "farm-cache.json";
