use std::{borrow::Cow, sync::Arc};

use crate::HashMap;

use super::{constant::CacheStoreTrait, error::CacheError, CacheStoreKey};

pub struct NamespaceStore {
  store: Arc<Box<dyn CacheStoreTrait>>,
  name: String,
}

impl NamespaceStore {
  pub fn new(store: Arc<Box<dyn CacheStoreTrait>>, name: String) -> Self {
    Self { store, name }
  }
}

impl NamespaceStore {
  fn format_name(&self, name: &str) -> String {
    format!("{}|{}", self.name, name)
  }

  fn map_store_key(&self, store_key: Cow<CacheStoreKey>) -> CacheStoreKey {
    let mut store_key = store_key.into_owned();

    store_key.name = self.format_name(store_key.name.as_str());

    store_key
  }
}

impl CacheStoreTrait for NamespaceStore {
  fn has_cache(&self, name: &str) -> bool {
    self.store.has_cache(self.format_name(name).as_str())
  }

  fn is_cache_changed(&self, store_key: &CacheStoreKey) -> bool {
    self
      .store
      .is_cache_changed(&self.map_store_key(Cow::Borrowed(store_key)))
  }

  fn write_single_cache(&self, store_key: CacheStoreKey, bytes: Vec<u8>) -> Result<(), CacheError> {
    self
      .store
      .write_single_cache(self.map_store_key(Cow::Borrowed(&store_key)), bytes)
  }

  fn write_manifest(&self) {
    self.store.write_manifest()
  }

  fn write_cache(&self, cache_map: HashMap<CacheStoreKey, Vec<u8>>) {
    self.store.write_cache(
      cache_map
        .into_iter()
        .map(|(k, v)| (self.map_store_key(Cow::Owned(k)), v))
        .collect(),
    );
  }

  fn read_cache(&self, name: &str) -> Option<Vec<u8>> {
    self.store.read_cache(self.format_name(name).as_str())
  }

  fn remove_cache(&self, name: &str) {
    self.store.remove_cache(self.format_name(name).as_str())
  }

  fn shutdown(&self) {
    self.store.shutdown();
  }
}

#[cfg(test)]
mod tests {
  use crate::cache::store::memory::MemoryCacheStore;

  use super::*;

  #[test]
  fn t1() {
    let store: Arc<Box<dyn CacheStoreTrait>> = Arc::new(Box::new(MemoryCacheStore::new()));
    let n1 = NamespaceStore::new(store.clone(), "n1".to_string());
    let n2 = NamespaceStore::new(store.clone(), "n2".to_string());

    let data = vec![1, 2, 3];
    assert_eq!(n1.read_cache("namespace"), None);

    n1.write_single_cache(("name1", "hash").into(), data.clone())
      .unwrap();

    assert_eq!(n1.has_cache("name1"), true);
    assert_eq!(n1.read_cache("name1").unwrap(), data);

    assert_eq!(n2.has_cache("name1"), false);
    assert_eq!(n2.read_cache("name1"), None);

    n2.write_single_cache(("name2", "hash").into(), data.clone())
      .unwrap();

    assert_eq!(n2.has_cache("name2"), true);
    assert_eq!(n2.read_cache("name2").unwrap(), data);

    assert_eq!(store.has_cache("n1|name1"), true);
    assert_eq!(store.has_cache("n2|name2"), true);
  }
}
