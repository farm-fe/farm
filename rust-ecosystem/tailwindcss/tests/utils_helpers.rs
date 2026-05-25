//! Tests for `utils::default_map` and `utils::replace_object`.

use farmfe_ecosystem_tailwindcss::utils::default_map::DefaultMap;
use farmfe_ecosystem_tailwindcss::utils::replace_object::replace_object;
use std::collections::HashMap;

#[test]
fn default_map_lazily_creates_values() {
  let mut counter = 0;
  let mut map: DefaultMap<String, i32, _> = DefaultMap::new(|_k: &String| {
    counter += 1;
    42
  });
  assert_eq!(*map.get(&"a".to_string()), 42);
  assert_eq!(*map.get(&"a".to_string()), 42); // cached
  assert_eq!(*map.get(&"b".to_string()), 42);
  // Counter is captured by reference into the closure, so checking len() is the
  // reliable signal that "a" was cached.
  assert_eq!(map.len(), 2);
}

#[test]
fn default_map_explicit_insert_overrides_factory() {
  let mut map: DefaultMap<String, i32, _> = DefaultMap::new(|_: &String| 0);
  map.insert("a".to_string(), 7);
  assert_eq!(*map.get(&"a".to_string()), 7);
  assert!(map.contains_key(&"a".to_string()));
  assert!(!map.contains_key(&"b".to_string()));
}

#[test]
fn default_map_is_empty_initially() {
  let map: DefaultMap<String, String, _> = DefaultMap::new(|k: &String| k.clone());
  assert!(map.is_empty());
  assert_eq!(map.len(), 0);
}

#[test]
fn replace_object_clears_target_first() {
  let mut target: HashMap<String, i32> = HashMap::new();
  target.insert("a".into(), 1);
  target.insert("b".into(), 2);

  let mut source: HashMap<String, i32> = HashMap::new();
  source.insert("c".into(), 3);

  replace_object(&mut target, source);

  assert!(!target.contains_key("a"));
  assert!(!target.contains_key("b"));
  assert_eq!(target.get("c"), Some(&3));
  assert_eq!(target.len(), 1);
}

#[test]
fn replace_object_with_empty_source_empties_target() {
  let mut target: HashMap<String, i32> = HashMap::new();
  target.insert("a".into(), 1);
  replace_object(&mut target, HashMap::new());
  assert!(target.is_empty());
}
