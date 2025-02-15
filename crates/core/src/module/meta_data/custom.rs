use crate::{Cacheable, HashMap};
use dashmap::DashMap;
use rkyv::*;
use std::{
  collections::hash_map::Iter,
  fmt::{Debug, Formatter},
};

#[derive(Default)]
pub struct CustomMetaDataMap {
  map: HashMap<String, Box<dyn Cacheable>>,
  /// The bytes map is used to store the serialized data of the map above
  bytes_map: DashMap<String, Vec<u8>>,
}

impl Debug for CustomMetaDataMap {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CustomMetaDataMap")
      .field(
        "map_keys",
        &self.map.iter().map(|item| item.0).collect::<Vec<&String>>() as _,
      )
      .field(
        "bytes_map_keys",
        &self
          .bytes_map
          .iter()
          .map(|item| item.key().to_string())
          .collect::<Vec<String>>() as _,
      )
      .finish()
  }
}

impl serde::Serialize for CustomMetaDataMap {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut map = HashMap::<String, Vec<u8>>::default();

    for (k, v) in self.map.iter() {
      let cloned_data = v.serialize_bytes().unwrap();
      map.insert(k.clone(), cloned_data);
    }

    serde::Serialize::serialize(&map, serializer)
  }
}

impl<'de> serde::Deserialize<'de> for CustomMetaDataMap {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let map: HashMap<String, Vec<u8>> = serde::Deserialize::deserialize(deserializer)?;
    let mut res = CustomMetaDataMap {
      map: HashMap::default(),
      bytes_map: DashMap::new(),
    };

    res.bytes_map = map.into_iter().collect();
    Ok(res)
  }
}

impl CustomMetaDataMap {
  pub fn new() -> Self {
    Self {
      map: HashMap::default(),
      bytes_map: DashMap::new(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

  pub fn iter(&self) -> Iter<String, Box<dyn Cacheable>> {
    self.map.iter()
  }

  pub fn get_mut<T: Cacheable + Default>(&mut self, key: &str) -> Option<&mut T> {
    if let Some((_, bytes)) = self.bytes_map.remove(key) {
      let value = T::deserialize_bytes(&T::default(), bytes).unwrap();
      self.map.insert(key.to_string(), value);
    }

    self.map.get_mut(key).and_then(|v| v.downcast_mut::<T>())
  }

  pub fn insert<T: Cacheable>(&mut self, key: String, value: Box<T>) {
    self.map.insert(key, value);
  }

  pub fn remove(&mut self, key: &str) {
    self.map.remove(key);
  }
}

impl From<HashMap<String, Box<dyn Cacheable>>> for CustomMetaDataMap {
  fn from(map: HashMap<String, Box<dyn Cacheable>>) -> Self {
    Self {
      map,
      bytes_map: DashMap::new(),
    }
  }
}

impl Clone for CustomMetaDataMap {
  fn clone(&self) -> Self {
    let custom = if self.map.is_empty() {
      HashMap::default()
    } else {
      let mut custom = HashMap::default();
      for (k, v) in self.map.iter() {
        let cloned_data = v.serialize_bytes().unwrap();
        let cloned_custom = v.deserialize_bytes(cloned_data).unwrap();
        custom.insert(k.clone(), cloned_custom);
      }
      custom
    };

    Self {
      map: custom,
      bytes_map: self.bytes_map.clone(),
    }
  }
}

impl<__D: Fallible + ?Sized> Deserialize<CustomMetaDataMap, __D> for Archived<CustomMetaDataMap> {
  #[inline]
  fn deserialize(
    &self,
    deserializer: &mut __D,
  ) -> ::core::result::Result<CustomMetaDataMap, __D::Error> {
    let map = Deserialize::<HashMap<String, Vec<u8>>, __D>::deserialize(&self.map, deserializer)?;
    let mut res = CustomMetaDataMap {
      map: HashMap::default(),
      bytes_map: DashMap::new(),
    };

    res.bytes_map = map.into_iter().collect();
    Ok(res)
  }
}

impl<__S: Fallible + ?Sized> Serialize<__S> for CustomMetaDataMap
where
  __S: rkyv::ser::Serializer + rkyv::ser::ScratchSpace,
{
  #[inline]
  fn serialize(&self, serializer: &mut __S) -> ::core::result::Result<Self::Resolver, __S::Error> {
    let mut map = HashMap::<String, Vec<u8>>::default();

    for (k, v) in self.map.iter() {
      let cloned_data = v.serialize_bytes().unwrap();
      map.insert(k.clone(), cloned_data);
    }

    let resolver_map = Serialize::<__S>::serialize(&map, serializer)?;

    for (k, v) in map {
      self.bytes_map.insert(k, v);
    }

    Ok(CustomMetaDataMapResolver { map: resolver_map })
  }
}

pub struct ArchivedCustomMetaDataMap {
  ///The archived counterpart of [`CustomMetaDataMap::map`]
  pub map: ::rkyv::Archived<HashMap<String, Vec<u8>>>,
}

pub struct CustomMetaDataMapResolver {
  map: ::rkyv::Resolver<HashMap<String, Vec<u8>>>,
}

impl Archive for CustomMetaDataMap {
  type Archived = ArchivedCustomMetaDataMap;
  type Resolver = CustomMetaDataMapResolver;
  #[allow(clippy::unit_arg)]
  #[inline]
  unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
    let (fp, fo) = {
      #[allow(unused_unsafe)]
      unsafe {
        let fo = &raw mut (*out).map;
        (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
      }
    };
    let mut map = HashMap::<String, Vec<u8>>::default();
    let mut keys = vec![];

    for item in self.bytes_map.iter() {
      keys.push(item.key().clone());
    }

    for key in keys {
      let (k, v) = self.bytes_map.remove(&key).unwrap();
      map.insert(k, v);
    }

    ::rkyv::Archive::resolve(&map, pos + fp, resolver.map, fo);
  }
}
