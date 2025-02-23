use dashmap::{
  mapref::one::{MappedRef, MappedRefMut, RefMut},
  DashMap,
};

use crate::{
  module::{CustomMetaDataMap, ModuleId},
  Cacheable,
};

#[derive(Default)]
pub struct ModuleMatedataStore {
  /// Map<plugin name, Map<ModuleId, CustomMetaDataMap>>
  module_matedata: DashMap<ModuleId, CustomMetaDataMap>,
}

impl ModuleMatedataStore {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn get_module_matedata(&self) -> DashMap<ModuleId, CustomMetaDataMap> {
    self.module_matedata.clone()
  }

  pub fn get_map(&self, key: &ModuleId) -> Option<CustomMetaDataMap> {
    self.module_matedata.remove(key).map(|(_, v)| v)
  }

  pub fn get_map_mut_ref(&self, key: &ModuleId) -> Option<RefMut<'_, ModuleId, CustomMetaDataMap>> {
    self.module_matedata.get_mut(key)
  }

  pub fn get_matedata<V: Cacheable>(&self, key: &ModuleId, name: &str) -> Option<Box<V>> {
    self
      .module_matedata
      .get_mut(key)
      .map(|mut v| v.get_cache(name))
      .flatten()
  }

  pub fn read_ref<V: Cacheable>(
    &self,
    key: &ModuleId,
    name: &str,
  ) -> Option<MappedRef<'_, ModuleId, CustomMetaDataMap, V>> {
    Some(
      self
        .module_matedata
        .get(key)?
        .map(|v| v.get_ref::<V>(name.as_ref()).unwrap()),
    )
  }

  pub fn read_ref_mut<V: Cacheable>(
    &self,
    key: &ModuleId,
    name: &str,
  ) -> Option<MappedRefMut<'_, ModuleId, CustomMetaDataMap, V>> {
    Some(
      self
        .module_matedata
        .get_mut(key)?
        .map(|v| v.get_mut::<V>(name.as_ref()).unwrap()),
    )
  }

  pub fn write_metadata(&self, key: ModuleId, name: String, metadata: Box<dyn Cacheable>) {
    if !self.module_matedata.contains_key(&key) {
      self
        .module_matedata
        .insert(key.clone(), CustomMetaDataMap::default());
    }

    let Some(mut namespace) = self.module_matedata.get_mut(&key) else {
      return;
    };

    namespace.value_mut().insert(name, metadata);
  }

  pub fn invalidate(&self, key: &ModuleId) {
    self.module_matedata.remove(key);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn t1() {
    // let store_factory: Rc<Box<dyn CacheStoreFactory>> =
    //   Rc::new(Box::new(MemoryCacheFactory::new()));
    let store = ModuleMatedataStore::new();

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
