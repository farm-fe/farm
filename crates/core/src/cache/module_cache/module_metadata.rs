use std::rc::Rc;

use dashmap::{
  mapref::one::{MappedRef, RefMut},
  DashMap,
};
use farmfe_utils::hash::sha256;
use rkyv::Deserialize;

use crate::{
  cache::store::{
    constant::{CacheStoreFactory, CacheStoreTrait},
    CacheStoreKey,
  },
  deserialize,
  module::CustomMetaDataMap,
  serialize, Cacheable, HashMap,
};

pub struct ModuleMetadataStore {
  pub store: Box<dyn CacheStoreTrait>,
  module_metadata: DashMap<String, CustomMetaDataMap>,
}

impl ModuleMetadataStore {
  pub fn new(store_factory: Rc<Box<dyn CacheStoreFactory>>) -> Self {
    let store = store_factory.create_cache_store("module-metadata");
    Self {
      store,
      module_metadata: DashMap::default(),
    }
  }

  fn metadata_from_store(&self, key: &str) -> Option<()> {
    if self.module_metadata.contains_key(key) {
      return None;
    }

    let cache = self.store.read_cache(key.to_string().as_str())?;

    let metadata_item = deserialize!(&cache, CustomMetaDataMap);

    self.module_metadata.insert(key.to_string(), metadata_item);

    None
  }

  pub fn read_mut(&self, key: &str) -> Option<RefMut<String, CustomMetaDataMap>> {
    self.metadata_from_store(key);

    self.module_metadata.get_mut(key)
  }

  pub fn read_mut_or_entry(&self, key: &str) -> RefMut<String, CustomMetaDataMap> {
    self.metadata_from_store(key);

    if !self.module_metadata.contains_key(key) {
      self
        .module_metadata
        .insert(key.to_string(), CustomMetaDataMap::default());
    }

    self.module_metadata.get_mut(key).unwrap()
  }

  pub fn read_ref<V: Cacheable>(
    &self,
    key: &str,
    name: &str,
  ) -> Option<MappedRef<'_, String, CustomMetaDataMap, V>> {
    self.metadata_from_store(key);

    Some(
      self
        .module_metadata
        .get(key)?
        .map(|v| v.get_ref::<V>(name.as_ref()).unwrap()),
    )
  }

  pub fn write_metadata(&self, key: String, name: String, metadata: Box<dyn Cacheable>) {
    self.metadata_from_store(&key);

    if !self.module_metadata.contains_key(&key) {
      self
        .module_metadata
        .insert(key.clone(), CustomMetaDataMap::default());
    }

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
          name: sha256(&value, 8),
        },
        value,
      );
    }
    self.store.write_cache(map);
  }

  pub fn invalidate(&self, key: &str) {
    self.module_metadata.remove(key);
    self.store.remove_cache(key.to_string().as_str());
  }
}

#[cfg(test)]
mod tests {
  use crate::cache::store::memory::MemoryCacheFactory;

  use super::*;

  #[test]
  fn t1() {
    let store_factory: Rc<Box<dyn CacheStoreFactory>> =
      Rc::new(Box::new(MemoryCacheFactory::new()));
    let store = ModuleMetadataStore::new(store_factory);

    let module1_id = "module1".to_string();
    let cached_value = "hello world".to_string();
    store.write_metadata(
      module1_id.clone(),
      "content".to_string(),
      Box::new(cached_value.clone()),
    );

    store.write_cache();

    let v = store
      .read_ref::<String>(&module1_id, "content")
      .map(|v| v.value().clone())
      .unwrap();

    assert_eq!(v, cached_value.clone());

    store.invalidate(&module1_id);

    let v = store.read_ref::<String>(&module1_id, "content");

    assert!(v.is_none());
  }
}
