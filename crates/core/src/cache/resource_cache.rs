use std::rc::Rc;

use self::resource_memory_store::{CachedResourcePot, ResourceMemoryStore};
use self::resource_pot::ResourcePotMemoryStore;

use super::store::constant::CacheStoreFactory;

pub mod resource_memory_store;
pub mod resource_pot;

pub struct ResourceCacheManager {
  resource_pot_store: ResourcePotMemoryStore,
}

impl ResourceCacheManager {
  pub fn new(store_factory: Rc<Box<dyn CacheStoreFactory>>) -> Self {
    Self {
      resource_pot_store: ResourcePotMemoryStore::new(store_factory),
    }
  }

  pub fn is_cache_changed(&self, name: String, hash: String) -> bool {
    self.resource_pot_store.is_cache_changed(name, hash)
  }

  pub fn has_cache(&self, name: &str) -> bool {
    self.resource_pot_store.has_cache(name)
  }

  pub fn set_cache(&self, name: &str, resource: CachedResourcePot) {
    self.resource_pot_store.set_cache(name, resource);
  }

  pub fn get_cache(&self, name: &str) -> Option<CachedResourcePot> {
    self.resource_pot_store.get_cache(name)
  }

  /// Write the cache map to the disk.
  pub fn write_cache(&self) {
    self.resource_pot_store.write_cache();
  }
}
