use std::sync::Arc;

use dashmap::mapref::one::Ref;
use farmfe_macro_cache_item::{cache_item, cache_item_options};
use farmfe_utils::{hash, nanoid::nanoid};
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
  cache::{
    store::{constant::CacheStoreTrait, CacheStoreKey},
    CacheContext,
  },
  serialize, Cacheable, DashMap,
};

use std::collections::{HashMap, HashSet};

type CacheReferenceId = String;
type ScopeId = String;
type CustomId = String;
type Name = String;
type QueryScope = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cache_item]
#[rkyv(derive(PartialEq, Eq, Hash))]
pub enum IdType {
  Reference(String),
  Scope(QueryScope),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheItemId {
  id: Option<Vec<IdType>>,
  name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ReversedCacheItemId {
  id: IdType,
  name: Name,
}

#[cache_item]
#[derive(Debug)]
struct ScopeManifest {
  name_keys: HashMap<String, String>,
  scope_keys: HashMap<IdType, HashSet<String>>,
}

pub struct CacheScopeStore {
  store: Box<dyn CacheStoreTrait>,
  data: DashMap<String, Box<dyn Cacheable>>,
  name_map: DashMap<Name, CacheReferenceId>,
  scope_map: DashMap<IdType, HashSet<CacheReferenceId>>,
}

pub type ScopeRef<'a, V> = dashmap::mapref::one::MappedRef<'a, String, Box<dyn Cacheable>, V>;

impl CacheScopeStore {
  pub fn new(context: Arc<CacheContext>) -> Self {
    let scope = Self {
      store: context.store_factory.create_cache_store("farm:scope"),
      data: Default::default(),
      scope_map: DashMap::default(),
      name_map: DashMap::default(),
    };

    scope.restore_manifest();

    scope
  }

  fn insert_combined<V: Cacheable>(&self, CacheItemId { id, name }: CacheItemId, data: V) {
    let reference_id: String = nanoid!();

    let mut is_write_only_name = false;
    if let Some(id_list) = id {
      for id in id_list {
        let mut composite_name = None;
        match &id {
          IdType::Reference(key) => {
            composite_name = Some(key);
          }
          _ => is_write_only_name = true,
        }

        if !self.scope_map.contains_key(&id) {
          self.scope_map.insert(id.clone(), Default::default());
        }

        self
          .scope_map
          .get_mut(&id)
          .unwrap()
          .insert(reference_id.clone());

        if let Some(composite_name) = composite_name {
          self.name_map.insert(
            format!("{}+{}", name.clone(), composite_name),
            reference_id.clone(),
          );
        }
      }
    } else {
      is_write_only_name = true
    }

    if is_write_only_name {
      self.name_map.insert(name.clone(), reference_id.clone());
    }

    self.data.insert(reference_id, Box::new(data));
  }

  fn get_data<V: Cacheable>(&self, reference_id: &str) -> Option<Ref<String, Box<dyn Cacheable>>> {
    if let Some(data) = self.data.get(reference_id) {
      return Some(data);
    }

    if !self.store.has_cache(reference_id) {
      return None;
    };

    if let Some(v) = self.store.read_cache(reference_id) {
      let v = V::deserialize_bytes(v).ok()?;
      self.data.insert(reference_id.to_string(), v);
    }

    self.data.get(reference_id)
  }

  pub fn set<V: Cacheable>(&self, name: impl ToString, data: V, id: Option<Vec<IdType>>) {
    self.insert_combined(
      CacheItemId {
        id,
        name: name.to_string(),
      },
      data,
    );
  }

  pub fn get_scope_ref<V: Cacheable>(&self, scope: &str) -> Vec<ScopeRef<V>> {
    let mut result = vec![];

    if let Some(references) = self.scope_map.get(&IdType::Scope(scope.to_string())) {
      for reference in references.value() {
        if let Some(data) = self.get_data::<V>(reference) {
          result.push(data.map(|v| v.downcast_ref::<V>().unwrap()));
        };
      }
    }

    result
  }

  pub fn get<V: Cacheable>(
    &self,
    name: impl AsRef<str>,
    id: Option<&Vec<IdType>>,
  ) -> Option<Box<V>> {
    let name = name.as_ref();
    let reference_id = if let Some(v) = id {
      let mut reference_id = None;
      for id_type in v {
        if let IdType::Reference(composite) = id_type {
          reference_id = self.name_map.get(&format!("{}+{}", name, composite));
        }

        if reference_id.is_some() {
          break;
        }
      }
      reference_id
    } else {
      self.name_map.get(name)
    };

    reference_id
      .map(|v| {
        let Some(v) = self.get_data::<V>(v.value()) else {
          return None;
        };

        let bytes = v.serialize_bytes().ok()?;

        V::deserialize_bytes(bytes).unwrap().downcast::<V>().ok()
      })
      .flatten()
  }

  pub fn remove_by_reference(&self, reference: &str) {
    let reference = IdType::Reference(reference.to_string());
    if let Some(references) = self.scope_map.get(&reference) {
      for reference in references.value() {
        self.data.remove(reference);
      }

      drop(references);

      self.scope_map.remove(&reference);
    }
  }

  fn restore_manifest(&self) {
    if let Some(manifest) = self.store.read_cache("scopeManifest") {
      let Ok(box ScopeManifest {
        name_keys,
        scope_keys,
      }) = ScopeManifest::deserialize_bytes(manifest)
        .unwrap()
        .downcast::<ScopeManifest>()
      else {
        return;
      };

      for (k, v) in name_keys {
        self.name_map.insert(k, v);
      }

      for (k, v) in scope_keys {
        self.scope_map.insert(k, v);
      }
    }
  }

  pub(crate) fn write_cache(&self) {
    let name_keys = self
      .name_map
      .iter()
      .map(|v| (v.key().clone(), v.value().clone()))
      .collect::<HashMap<String, String>>();
    let scope_keys = self
      .scope_map
      .iter()
      .map(|v| (v.key().clone(), v.value().clone()))
      .collect::<HashMap<IdType, HashSet<String>>>();

    let manifest = ScopeManifest {
      name_keys,
      scope_keys,
    };
    let manifest_scopes = serialize!(&manifest);
    self
      .store
      .write_single_cache(
        CacheStoreKey {
          name: "scopeManifest".to_string(),
          key: hash::sha256(&manifest_scopes, 32),
        },
        manifest_scopes,
      )
      .expect("write name map cache failed");

    self
      .data
      .iter()
      .par_bridge()
      .fold(
        || HashMap::<CacheStoreKey, Vec<u8>>::default(),
        |mut res, item| {
          let v = item.serialize_bytes().unwrap();

          res.insert(
            CacheStoreKey {
              name: item.key().clone(),
              key: hash::sha256(&v, 8),
            },
            v,
          );
          res
        },
      )
      .for_each(|item| {
        self.store.write_cache(item.into_iter().collect());
      });
  }
}

#[cfg(test)]
mod tests {
  use farmfe_macro_cache_item::cache_item;

  use crate::{
    cache::scope::{CacheScopeStore, IdType},
    module::{Module, ModuleId},
  };

  #[test]
  fn t1() {
    let c1 = CacheScopeStore::new(Default::default());

    let mn1 = ModuleId::from("a.js");
    let m1 = Module::new(mn1.clone());
    let mn2 = ModuleId::from("b.js");
    let m2 = Module::new(mn2.clone());
    let mn3 = ModuleId::from("c.js");
    let m3 = Module::new(mn3.clone());

    c1.set(
      "module1".to_string(),
      m1,
      Some(vec![IdType::Scope("s1".to_string())]),
    );

    c1.set(
      "module2".to_string(),
      m2,
      Some(vec![IdType::Scope("s1".to_string())]),
    );

    c1.set(
      "module3".to_string(),
      m3,
      Some(vec![IdType::Scope("s2".to_string())]),
    );

    let s1_data = c1.get_scope_ref::<Module>(&"s1".to_string());
    assert_eq!(s1_data.len(), 2);

    let s2_data = c1.get_scope_ref::<Module>(&"s2".to_string());
    assert_eq!(s2_data.len(), 1);

    let m1_data = c1.get::<Module>("module1", None);

    assert_eq!(m1_data.unwrap().id, mn1);
  }

  #[cache_item]
  #[derive(Debug)]
  struct TwToken {
    value: Vec<String>,
  }

  #[test]
  fn scope() {
    let c1 = CacheScopeStore::new(Default::default());
    c1.set(
      "tw-token".to_string(),
      TwToken {
        value: vec!["mt-1".to_string()],
      },
      Some(vec![
        IdType::Reference("index.tsx".to_string()),
        IdType::Scope("tailwind-token".to_string()),
      ]),
    );

    c1.set(
      "tw-token".to_string(),
      TwToken {
        value: vec!["mb-1".to_string()],
      },
      Some(vec![
        IdType::Reference("index2.tsx".to_string()),
        IdType::Scope("tailwind-token".to_string()),
      ]),
    );

    let tw_scope_values = c1.get_scope_ref::<TwToken>(&"tailwind-token".to_string());

    println!(
      "{:#?}",
      tw_scope_values
        .iter()
        .map(|v| v.value())
        .collect::<Vec<_>>()
    );

    let i_v = c1.get::<TwToken>(
      "tw-token",
      Some(&vec![IdType::Reference("index.tsx".to_string())]),
    );

    println!("{:#?}", i_v);

    let i2_v = c1.get::<TwToken>(
      "tw-token",
      Some(&vec![IdType::Reference("index2.tsx".to_string())]),
    );

    println!("{:#?}", i2_v);
    // c1.
  }

  #[test]
  fn remove() {
    let c1 = CacheScopeStore::new(Default::default());
    c1.set(
      "tw-token".to_string(),
      TwToken {
        value: vec!["mt-1".to_string()],
      },
      Some(vec![
        IdType::Reference("index.tsx".to_string()),
        IdType::Scope("tailwind-token".to_string()),
      ]),
    );

    let i_v = c1.get::<TwToken>(
      "tw-token",
      Some(&vec![IdType::Reference("index.tsx".to_string())]),
    );

    assert!(i_v.is_some());

    let i_v = c1.get::<TwToken>(
      "tw-token",
      Some(&vec![IdType::Reference("index.tsx".to_string())]),
    );

    assert!(i_v.is_some());

    c1.remove_by_reference("index.tsx");

    let i_v = c1.get::<TwToken>(
      "tw-token",
      Some(&vec![IdType::Reference("index.tsx".to_string())]),
    );

    assert!(i_v.is_none());
  }

  #[test]
  fn rkyv_des() {
    let store = CacheScopeStore::new(Default::default());
    let data = "hello world".to_string();
    store.set(
      "n1",
      data,
      Some(vec![
        IdType::Reference("r1".to_string()),
        IdType::Scope("s1".to_string()),
      ]),
    );

    store.set(
      "n2",
      "hello world2".to_string(),
      Some(vec![
        IdType::Reference("r2".to_string()),
        IdType::Scope("s1".to_string()),
      ]),
    );

    store.write_cache();

    store.restore_manifest();
  }
}
