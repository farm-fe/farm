//! Minimal SFC descriptor metadata used as the Phase 2 HMR foundation.
//!
//! Farm's current Rust plugin hook surface can observe update paths, but it
//! does not expose enough graph-level operations here to safely implement
//! Vue's full template-only/style-only hot replacement. This cache records
//! the stable block metadata needed for that later invalidation step without
//! changing Phase A output.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use farmfe_core::parking_lot::Mutex;
use fxhash::FxHashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleDescriptor {
  pub index: usize,
  pub lang: String,
  pub scoped: bool,
  pub content_hash: u64,
  pub virtual_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomBlockDescriptor {
  pub index: usize,
  pub tag_name: String,
  pub content_hash: u64,
  pub virtual_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfcDescriptor {
  pub source_hash: String,
  pub is_custom_element: bool,
  pub styles: Vec<StyleDescriptor>,
  pub custom_blocks: Vec<CustomBlockDescriptor>,
}

#[derive(Default)]
pub struct DescriptorCache {
  inner: Mutex<FxHashMap<String, SfcDescriptor>>,
}

impl DescriptorCache {
  pub fn insert(&self, module_id: String, descriptor: SfcDescriptor) {
    self.inner.lock().insert(module_id, descriptor);
  }

  pub fn get(&self, module_id: &str) -> Option<SfcDescriptor> {
    self.inner.lock().get(module_id).cloned()
  }
}

pub fn content_hash(content: &str) -> u64 {
  let mut hasher = DefaultHasher::new();
  content.hash(&mut hasher);
  hasher.finish()
}
