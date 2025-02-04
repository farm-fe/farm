use dashmap::{
  mapref::one::{Ref, RefMut},
  DashMap,
};
use rkyv::Deserialize;

use crate::{
  cache::store::{constant::CacheStoreTrait, CacheStore, CacheStoreKey},
  config::Mode,
  deserialize,
  error::Result,
  module::{CustomMetaDataMap, ModuleId},
  serialize, Cacheable, HashMap,
};

pub struct ModuleMetadataStore {
  pub store: CacheStore,
  module_metadata: DashMap<ModuleId, CustomMetaDataMap>,
}

impl ModuleMetadataStore {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    Self {
      store: CacheStore::new(cache_dir_str, namespace, mode, "module-metadata"),
      module_metadata: DashMap::default(),
    }
  }

  fn metadata_from_store(&self, key: &ModuleId) -> Result<Option<()>> {
    let Some(cache) = self.store.read_cache(key.to_string().as_str()) else {
      return Ok(None);
    };

    let metadata_item = deserialize!(&cache, CustomMetaDataMap);

    self.module_metadata.insert(key.clone(), metadata_item);

    Ok(None)
  }

  pub fn read_mut(&self, key: &ModuleId) -> Option<RefMut<ModuleId, CustomMetaDataMap>> {
    if !self.module_metadata.contains_key(key) {
      self.metadata_from_store(key);
    };

    self.module_metadata.get_mut(key)
  }

  pub fn read_ref(&self, key: &ModuleId) -> Option<Ref<ModuleId, CustomMetaDataMap>> {
    if !self.module_metadata.contains_key(key) {
      self.metadata_from_store(key);
    }

    self.module_metadata.get(key)
  }

  pub fn write_metadata(&self, key: ModuleId, name: String, metadata: Box<dyn Cacheable>) {
    let Some(mut namespace) = self.read_mut(&key) else {
      return;
    };

    namespace.value_mut().insert(name, metadata);
  }

  pub fn write_cache(&self) {
    let mut map = HashMap::default();
    for item in self.module_metadata.iter() {
      let key = item.key();
      let value = serialize!(item.value());

      map.insert(
        CacheStoreKey {
          key: key.to_string(),
          name: key.to_string(),
        },
        value,
      );
    }
    self.store.write_cache(map);
  }

  pub fn invalidate(&self, key: &ModuleId) {
    self.module_metadata.remove(key);
    self.store.remove_cache(key.to_string().as_str());
  }
}
