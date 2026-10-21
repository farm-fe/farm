use crate::{Cacheable, HashMap};
use farmfe_macro_cache_item::cache_item;
use std::{
  collections::hash_map::Iter,
  fmt::{Debug, Formatter},
};

#[derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
#[rkyv(remote = InternalCustomMetaDataMap)]
#[rkyv(archived = ArchivedInternalCustomMetaDataMap)]
struct CacheableCustomMetaDataMap {
  #[rkyv(getter = get_bytes_map)]
  bytes_map: HashMap<String, Vec<u8>>,
}

fn get_bytes_map(value: &InternalCustomMetaDataMap) -> HashMap<String, Vec<u8>> {
  let mut bytes = value.bytes_map.clone();
  for (key, data) in &value.map {
    bytes.insert(key.clone(), data.serialize_bytes().unwrap());
  }
  bytes
}

impl From<CacheableCustomMetaDataMap> for InternalCustomMetaDataMap {
  fn from(value: CacheableCustomMetaDataMap) -> Self {
    Self {
      bytes_map: value.bytes_map.clone(),
      map: Default::default(),
    }
  }
}

#[derive(Default)]
struct InternalCustomMetaDataMap {
  map: HashMap<String, Box<dyn Cacheable>>,
  bytes_map: HashMap<String, Vec<u8>>,
}

impl InternalCustomMetaDataMap {
  #[inline]
  fn remove_cache(&mut self, key: &str) {
    self.map.remove(key);
    self.bytes_map.remove(key);
  }

  #[inline]
  fn get_mut<T: Cacheable>(&mut self, key: &str) -> Option<&mut T> {
    if let Some(bytes) = self.bytes_map.remove(key) {
      let value = T::deserialize_bytes(bytes).unwrap();
      self.map.insert(key.to_string(), value);
    }

    self.map.get_mut(key).and_then(|v| v.downcast_mut::<T>())
  }

  #[inline]
  fn get_cache<T: Cacheable>(&mut self, key: &str) -> Option<Box<T>> {
    if let Some(v) = self.map.get(key) {
      let bytes = v.serialize_bytes().ok()?;
      return T::deserialize_bytes(bytes).ok()?.downcast::<T>().ok();
    }

    if let Some(bytes) = self.bytes_map.get(key) {
      let value = T::deserialize_bytes(bytes.clone()).unwrap();
      return value.downcast::<T>().ok();
    }

    None
  }

  #[inline]
  fn get_ref<T: Cacheable>(&mut self, key: &str) -> Option<&T> {
    if let Some(v) = self.bytes_map.remove(key) {
      let value = T::deserialize_bytes(v).unwrap();
      self.map.insert(key.to_string(), value);
    }

    self.map.get(key).and_then(|v| v.downcast_ref::<T>())
  }

  #[inline]
  fn insert(&mut self, key: String, value: Box<dyn Cacheable>) {
    self.bytes_map.remove(&key);
    self.map.insert(key, value);
  }
}

#[derive(Default)]
#[cache_item]
pub struct CustomMetaDataMap {
  #[rkyv(with = CacheableCustomMetaDataMap)]
  internal: InternalCustomMetaDataMap,
}

impl Debug for CustomMetaDataMap {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CustomMetaDataMap")
      .field(
        "map_keys",
        &self
          .internal
          .map
          .iter()
          .map(|item| item.0)
          .collect::<Vec<&String>>() as _,
      )
      .finish()
  }
}

impl CustomMetaDataMap {
  pub fn new() -> Self {
    Self {
      internal: Default::default(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.internal.map.is_empty() && self.internal.bytes_map.is_empty()
  }

  pub fn iter(&self) -> Iter<'_, String, Box<dyn Cacheable>> {
    self.internal.map.iter()
  }

  pub fn get_mut<T: Cacheable>(&mut self, key: &str) -> Option<&mut T> {
    self.internal.get_mut::<T>(key)
  }

  pub fn get_cache<T: Cacheable>(&mut self, key: &str) -> Option<Box<T>> {
    self.internal.get_cache(key)
  }

  pub fn get_ref<T: Cacheable>(&mut self, key: &str) -> Option<&T> {
    self.internal.get_ref(key)
  }

  pub fn insert(&mut self, key: String, value: Box<dyn Cacheable>) {
    self.internal.insert(key, value);
  }

  pub fn remove(&mut self, key: &str) {
    self.internal.remove_cache(key);
  }
}

impl From<HashMap<String, Box<dyn Cacheable>>> for CustomMetaDataMap {
  fn from(map: HashMap<String, Box<dyn Cacheable>>) -> Self {
    Self {
      internal: InternalCustomMetaDataMap {
        map,
        bytes_map: HashMap::default(),
      },
    }
  }
}

impl Clone for CustomMetaDataMap {
  fn clone(&self) -> Self {
    let mut map = self.internal.bytes_map.clone();

    self.internal.map.iter().for_each(|(k, v)| {
      let cloned_data = v.serialize_bytes().unwrap();
      map.insert(k.clone(), cloned_data);
    });

    Self {
      internal: InternalCustomMetaDataMap {
        map: Default::default(),
        bytes_map: map,
      },
    }
  }
}

#[cfg(test)]
mod persistence_tests {
  use super::*;

  #[cache_item]
  struct PluginData {
    generated_css: String,
  }

  fn fixture() -> CustomMetaDataMap {
    let mut metadata = CustomMetaDataMap::new();
    metadata.insert(
      "plugin".into(),
      Box::new(PluginData {
        generated_css: ".generated{display:grid}".into(),
      }),
    );
    metadata
  }

  fn restore(metadata: &CustomMetaDataMap) -> Box<CustomMetaDataMap> {
    CustomMetaDataMap::deserialize_bytes(metadata.serialize_bytes().unwrap())
      .unwrap()
      .downcast::<CustomMetaDataMap>()
      .ok()
      .unwrap()
  }

  #[test]
  fn clone_preserves_runtime_plugin_metadata_independently() {
    let mut original = fixture();
    let mut cloned = original.clone();
    original
      .get_mut::<PluginData>("plugin")
      .unwrap()
      .generated_css = ".changed{}".into();
    assert_eq!(
      cloned
        .get_ref::<PluginData>("plugin")
        .map(|data| data.generated_css.as_str()),
      Some(".generated{display:grid}"),
      "clone must preserve runtime plugin metadata"
    );
  }

  #[test]
  fn unread_plugin_metadata_survives_second_serialization() {
    let unread = restore(&fixture());
    assert!(unread.internal.map.is_empty());
    let mut restored = restore(&unread);
    assert_eq!(
      restored
        .get_ref::<PluginData>("plugin")
        .map(|data| data.generated_css.as_str()),
      Some(".generated{display:grid}"),
      "unread plugin metadata must survive reserialization"
    );
  }

  #[test]
  fn unread_plugin_metadata_is_not_empty() {
    let unread = restore(&fixture());
    assert!(
      !unread.is_empty(),
      "unread plugin metadata must not report empty"
    );
  }
}
