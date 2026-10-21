use crate::HashMap;

use super::{error::CacheError, CacheStoreKey};

pub type CacheStoreItemRef<'a> = dashmap::mapref::one::MappedRef<'a, String, Vec<u8>, Vec<u8>>;

pub trait CacheStoreTrait: Send + Sync {
  fn has_cache(&self, name: &str) -> bool;
  fn is_cache_changed(&self, store_key: &CacheStoreKey) -> bool;
  fn write_single_cache(&self, store_key: CacheStoreKey, bytes: Vec<u8>) -> Result<(), CacheError>;
  #[allow(unused_variables)]
  fn write_cache(&self, cache_map: HashMap<CacheStoreKey, Vec<u8>>) {}
  fn read_cache(&self, name: &str) -> Option<Vec<u8>>;
  fn read_cache_ref(&self, name: &str) -> Option<CacheStoreItemRef<'_>>;
  fn remove_cache(&self, name: &str) -> Option<Vec<u8>>;
  /// Invalidate an entry without restoring its payload from persistent storage.
  fn remove_cache_only(&self, name: &str);
  fn shutdown(&self) {}
}

// Multi-file emit metadata cannot be reconstructed from the old single-file
// cache. A new directory format forces one rebuild after upgrading.
// Hybrid async modules must also be rendered again after the dependency-await
// fix; old rendered resource pots contain synchronous imports of async modules.
// Logical namespaces now have independent disk shards. Older shared-shard
// directories remain on disk but must not be read with the new layout.
// Earlier generated metadata/resources may be stale after a same-source update.
// Preserve the shard layout but require one fresh build after this fix.
pub const FARM_CACHE_VERSION: &str = "0.9.5";
pub const FARM_CACHE_MANIFEST_FILE: &str = "farm-cache.json";

pub trait CacheStoreFactory: Send + Sync {
  fn create_cache_store(&self, name: &str) -> Box<dyn CacheStoreTrait>;
}
