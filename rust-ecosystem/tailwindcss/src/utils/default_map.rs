//! `DefaultMap`, ported from `packages/tailwindcss/src/utils/default-map.ts`.
//!
//! A map that lazily generates default values via a factory closure.

use std::collections::HashMap;
use std::hash::Hash;

/// A map that generates default values for keys that don't exist and caches
/// the result.
///
/// Differs slightly from the JS API: the factory is supplied at construction
/// time and the map only owns its values (no `&self` references to the
/// factory are leaked).
pub struct DefaultMap<K: Eq + Hash + Clone, V, F: FnMut(&K) -> V> {
  map: HashMap<K, V>,
  factory: F,
}

impl<K: Eq + Hash + Clone, V, F: FnMut(&K) -> V> DefaultMap<K, V, F> {
  pub fn new(factory: F) -> Self {
    Self { map: HashMap::new(), factory }
  }

  /// Get a reference to the value, generating it on first access.
  pub fn get(&mut self, key: &K) -> &V {
    if !self.map.contains_key(key) {
      let value = (self.factory)(key);
      self.map.insert(key.clone(), value);
    }
    self.map.get(key).expect("just inserted")
  }

  /// Insert a key/value pair directly, bypassing the factory.
  pub fn insert(&mut self, key: K, value: V) -> Option<V> {
    self.map.insert(key, value)
  }

  /// Return `true` if the underlying map already contains `key`.
  pub fn contains_key(&self, key: &K) -> bool {
    self.map.contains_key(key)
  }

  /// Number of cached entries.
  pub fn len(&self) -> usize {
    self.map.len()
  }

  /// Whether the underlying cache is empty.
  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

  /// Iterate over the cached `(key, value)` pairs.
  pub fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
    self.map.iter()
  }
}
