use dashmap::{mapref::one::MappedRefMut, DashMap};

use crate::{
  module::{CustomMetaDataMap, ModuleId},
  Cacheable, HashMap,
};

#[derive(Default)]
pub struct ModuleMetadataStore {
  /// Map<plugin name, Map<ModuleId, CustomMetaDataMap>>
  module_metadata: DashMap<ModuleId, CustomMetaDataMap>,
}

impl ModuleMetadataStore {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn take_metadata(&self) -> HashMap<ModuleId, CustomMetaDataMap> {
    let keys = self
      .module_metadata
      .iter()
      .map(|i| i.key().clone())
      .collect::<Vec<_>>();
    let mut v = HashMap::default();

    for item in keys {
      let (_, data) = self.module_metadata.remove(&item).unwrap();
      v.insert(item, data);
    }

    v
  }

  pub fn set_map(&self, key: ModuleId, value: CustomMetaDataMap) {
    self.module_metadata.insert(key, value);
  }

  pub fn get_metadata<V: Cacheable>(&self, key: &ModuleId, name: &str) -> Option<Box<V>> {
    self
      .module_metadata
      .get_mut(key)
      .and_then(|mut v| v.get_cache(name))
  }

  pub fn read_ref<V: Cacheable>(
    &self,
    key: &ModuleId,
    name: &str,
  ) -> Option<MappedRefMut<'_, ModuleId, CustomMetaDataMap, V>> {
    Some(
      self
        .module_metadata
        .get_mut(key)?
        .map(|v| v.get_mut::<V>(name.as_ref()).unwrap()),
    )
  }

  pub fn write_metadata(&self, key: ModuleId, name: String, metadata: Box<dyn Cacheable>) {
    if !self.module_metadata.contains_key(&key) {
      self
        .module_metadata
        .insert(key.clone(), CustomMetaDataMap::default());
    }

    let Some(mut namespace) = self.module_metadata.get_mut(&key) else {
      return;
    };

    namespace.value_mut().insert(name, metadata);
  }

  pub fn invalidate(&self, key: &ModuleId) {
    self.module_metadata.remove(key);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn t1() {
    // let store_factory: Rc<Box<dyn CacheStoreFactory>> =
    //   Rc::new(Box::new(MemoryCacheFactory::new()));
    let store = ModuleMetadataStore::new();

    let module1_id = ModuleId::from("module1");
    let cached_value = "hello world".to_string();
    store.write_metadata(
      module1_id.clone(),
      "content".to_string(),
      Box::new(cached_value.clone()),
    );

    // store.write_cache();

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
