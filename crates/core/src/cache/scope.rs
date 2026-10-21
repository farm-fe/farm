use std::sync::Arc;

use dashmap::mapref::one::Ref;
use farmfe_macro_cache_item::cache_item;
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

#[cache_item]
#[derive(Debug)]
struct ScopeManifest {
  name_keys: HashMap<String, String>,
  scope_keys: HashMap<IdType, HashSet<String>>,
}

impl ScopeManifest {
  fn cache_key(&self) -> String {
    // Hash logical relationships, not randomized HashMap/HashSet archive
    // ordering. Only identifiers are copied; cached payloads are untouched.
    let mut names = self
      .name_keys
      .iter()
      .map(|(name, reference)| (name.clone(), reference.clone()))
      .collect::<Vec<_>>();
    names.sort_unstable();
    let mut scopes = self
      .scope_keys
      .iter()
      .map(|(id, references)| {
        let mut references = references.iter().cloned().collect::<Vec<_>>();
        references.sort_unstable();
        match id {
          IdType::Reference(value) => (0u8, value.clone(), references),
          IdType::Scope(value) => (1u8, value.clone(), references),
        }
      })
      .collect::<Vec<_>>();
    scopes.sort_unstable();
    hash::sha256(&serialize!(&(names, scopes)), 32)
  }
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
      data: DashMap::default(),
      name_map: DashMap::default(),
      scope_map: DashMap::default(),
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

  fn get_data_ref<V: Cacheable>(
    &self,
    reference_id: &str,
  ) -> Option<Ref<'_, String, Box<dyn Cacheable>>> {
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

  fn get_data<V: Cacheable>(&self, reference_id: &str) -> Option<Box<V>> {
    let data = self.get_data_ref::<V>(reference_id)?;

    let bytes = data.serialize_bytes().ok()?;

    V::deserialize_bytes(bytes).unwrap().downcast::<V>().ok()
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

  pub fn get_scope<V: Cacheable>(&self, scope: &str) -> Vec<V> {
    let mut result = vec![];

    if let Some(references) = self.scope_map.get(&IdType::Scope(scope.to_string())) {
      for reference in references.value() {
        if let Some(data) = self.get_data::<V>(reference) {
          result.push(*data);
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
          reference_id = self.name_map.get(&format!("{name}+{composite}"));
        }

        if reference_id.is_some() {
          break;
        }
      }
      reference_id
    } else {
      self.name_map.get(name)
    };

    reference_id.and_then(|v| {
      let v = self.get_data_ref::<V>(v.value())?;

      let bytes = v.serialize_bytes().ok()?;

      V::deserialize_bytes(bytes).unwrap().downcast::<V>().ok()
    })
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
          key: manifest.cache_key(),
        },
        manifest_scopes,
      )
      .expect("write name map cache failed");

    self
      .data
      .iter()
      .par_bridge()
      .fold(
        HashMap::<CacheStoreKey, Vec<u8>>::default,
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
  use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
  };

  use farmfe_macro_cache_item::cache_item;

  use crate::{
    cache::scope::{CacheScopeStore, IdType},
    module::{Module, ModuleId},
    Cacheable,
  };

  #[test]
  fn scope_manifest_key_ignores_map_and_set_insertion_order() {
    let make_manifest = |reverse: bool| {
      let mut indices = (0..32).collect::<Vec<_>>();
      if reverse {
        indices.reverse();
      }
      super::ScopeManifest {
        name_keys: indices
          .iter()
          .map(|index| (format!("name-{index}"), format!("ref-{index}")))
          .collect(),
        scope_keys: [
          IdType::Reference("same".to_string()),
          IdType::Scope("same".to_string()),
        ]
        .into_iter()
        .map(|id| {
          (
            id,
            indices.iter().map(|index| format!("ref-{index}")).collect(),
          )
        })
        .collect(),
      }
    };
    assert_eq!(
      make_manifest(false).cache_key(),
      make_manifest(true).cache_key()
    );
  }

  #[test]
  fn scope_manifest_key_tracks_names_reference_ids_kinds_and_members() {
    let make_manifest = |id, name: &str, reference: &str, members: &[&str]| super::ScopeManifest {
      name_keys: HashMap::from([(name.to_string(), reference.to_string())]),
      scope_keys: HashMap::from([(
        id,
        members
          .iter()
          .map(|value| value.to_string())
          .collect::<HashSet<_>>(),
      )]),
    };
    let reference = || IdType::Reference("same".to_string());
    let baseline = make_manifest(reference(), "entry", "ref", &["a", "b"]).cache_key();
    for (label, changed) in [
      (
        "name",
        make_manifest(reference(), "other", "ref", &["a", "b"]),
      ),
      (
        "reference",
        make_manifest(reference(), "entry", "other", &["a", "b"]),
      ),
      (
        "kind",
        make_manifest(
          IdType::Scope("same".to_string()),
          "entry",
          "ref",
          &["a", "b"],
        ),
      ),
      (
        "scope id",
        make_manifest(
          IdType::Reference("other".to_string()),
          "entry",
          "ref",
          &["a", "b"],
        ),
      ),
      (
        "member",
        make_manifest(reference(), "entry", "ref", &["a", "c"]),
      ),
      (
        "member count",
        make_manifest(reference(), "entry", "ref", &["a", "b", "c"]),
      ),
    ] {
      assert_ne!(
        changed.cache_key(),
        baseline,
        "Lost manifest change: {label}"
      );
    }
  }

  #[test]
  fn unchanged_scope_manifest_does_not_replace_cached_bytes() {
    let context = Arc::new(crate::cache::CacheContext::default());
    let store = CacheScopeStore::new(context.clone());
    for index in 0..32 {
      store.set(
        format!("metadata-{index}"),
        format!("value-{index}"),
        Some(vec![
          IdType::Reference(format!("group-{}", index % 4)),
          IdType::Scope(format!("group-{}", index % 4)),
        ]),
      );
    }
    store.write_cache();
    let before = store.store.read_cache("scopeManifest").unwrap();
    let baseline = super::ScopeManifest::deserialize_bytes(before.clone())
      .unwrap()
      .downcast::<super::ScopeManifest>()
      .ok()
      .unwrap();
    for _ in 0..8 {
      store.write_cache();
      let after = store.store.read_cache("scopeManifest").unwrap();
      let restored = super::ScopeManifest::deserialize_bytes(after.clone())
        .unwrap()
        .downcast::<super::ScopeManifest>()
        .ok()
        .unwrap();
      assert_eq!(restored.name_keys, baseline.name_keys);
      assert_eq!(restored.scope_keys, baseline.scope_keys);
      assert!(
        after == before,
        "Unchanged logical manifest replaced cached bytes"
      );
    }
    store.set("extra", "new value".to_string(), None);
    store.write_cache();
    assert_ne!(store.store.read_cache("scopeManifest").unwrap(), before);
    let reopened = CacheScopeStore::new(context);
    assert_eq!(
      &*reopened.get::<String>("extra", None).unwrap(),
      "new value"
    );
    assert_eq!(reopened.get_scope::<String>("group-0").len(), 8);
  }

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

    let s1_data = c1.get_scope::<Module>("s1");
    assert_eq!(s1_data.len(), 2);

    let s2_data = c1.get_scope::<Module>("s2");
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

    let tw_scope_values = c1.get_scope::<TwToken>("tailwind-token");

    assert_eq!(tw_scope_values.len(), 2);

    let i_v = c1.get::<TwToken>(
      "tw-token",
      Some(&vec![IdType::Reference("index.tsx".to_string())]),
    );

    assert!(i_v.is_some());
    assert!(i_v.unwrap().value.contains(&"mt-1".to_string()));

    let i2_v = c1.get::<TwToken>(
      "tw-token",
      Some(&vec![IdType::Reference("index2.tsx".to_string())]),
    );

    assert!(i2_v.is_some());
    assert!(i2_v.as_ref().unwrap().value.contains(&"mb-1".to_string()));
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
