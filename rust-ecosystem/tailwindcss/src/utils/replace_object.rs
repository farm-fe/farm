//! `replace_object`, ported from
//! `packages/tailwindcss/src/utils/replace-object.ts`.
//!
//! Upstream replaces all keys in-place on a mutable JS object. In Rust we
//! model that with `HashMap::clear()` followed by `extend(...)`.

use std::collections::HashMap;
use std::hash::Hash;

/// Clear `target` and replace its contents with `source`, mirroring the
/// upstream JS helper.
pub fn replace_object<K: Eq + Hash, V>(target: &mut HashMap<K, V>, source: HashMap<K, V>) {
  target.clear();
  target.extend(source);
}
