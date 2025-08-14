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
  value
    .map
    .iter()
    .map(|(k, v)| (k.clone(), v.serialize_bytes().unwrap()))
    .collect()
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
    self.internal.map.is_empty()
  }

  pub fn iter(&self) -> Iter<String, Box<dyn Cacheable>> {
    self.internal.map.iter()
  }

  pub fn get_mut<T: Cacheable>(&mut self, key: &str) -> Option<&mut T> {
    if let Some(bytes) = self.internal.bytes_map.remove(key) {
      let value = T::deserialize_bytes(bytes).unwrap();
      self.internal.map.insert(key.to_string(), value);
    }

    self
      .internal
      .map
      .get_mut(key)
      .and_then(|v| v.downcast_mut::<T>())
  }

  pub fn get_cache<T: Cacheable>(&mut self, key: &str) -> Option<Box<T>> {
    if let Some(v) = self.internal.map.get(key) {
      let bytes = v.serialize_bytes().ok()?;

      return T::deserialize_bytes(bytes).ok()?.downcast::<T>().ok();
    }

    if let Some(bytes) = self.internal.bytes_map.get(key) {
      let value = T::deserialize_bytes(bytes.clone()).unwrap();
      return value.downcast::<T>().ok();
    }

    None
  }

  pub fn get_ref<T: Cacheable>(&self, key: &str) -> Option<&T> {
    self
      .internal
      .map
      .get(key)
      .and_then(|v| v.downcast_ref::<T>())
  }

  pub fn insert(&mut self, key: String, value: Box<dyn Cacheable>) {
    self.internal.map.insert(key, value);
    // pub fn insert<T: Cacheable>(&mut self, key: String, value: Box<T>) {
    //   self.internal.map.insert(key, value);
  }

  pub fn remove(&mut self, key: &str) {
    self.internal.map.remove(key);
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
        bytes_map: self.internal.bytes_map.clone(),
      },
    }
  }
}
