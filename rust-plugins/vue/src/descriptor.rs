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
  pub owner_module_id: String,
  pub owner_resolved_path: String,
  pub scope_id: String,
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
  pub main_content_hash: u64,
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

fn same_style_shape(a: &[StyleDescriptor], b: &[StyleDescriptor]) -> bool {
  a.len() == b.len()
    && a.iter().zip(b).all(|(a, b)| {
      a.index == b.index && a.lang == b.lang && a.scoped == b.scoped && a.virtual_id == b.virtual_id
    })
}

fn same_custom_block_shape(a: &[CustomBlockDescriptor], b: &[CustomBlockDescriptor]) -> bool {
  a.len() == b.len()
    && a
      .iter()
      .zip(b)
      .all(|(a, b)| a.index == b.index && a.tag_name == b.tag_name && a.virtual_id == b.virtual_id)
}

pub fn narrow_virtual_updates(
  previous: &SfcDescriptor,
  next: &SfcDescriptor,
) -> Option<Vec<String>> {
  if previous.is_custom_element != next.is_custom_element
    || previous.main_content_hash != next.main_content_hash
    || !same_style_shape(&previous.styles, &next.styles)
    || !same_custom_block_shape(&previous.custom_blocks, &next.custom_blocks)
  {
    return None;
  }

  let mut updates = Vec::new();

  for (previous_style, next_style) in previous.styles.iter().zip(&next.styles) {
    if previous_style.content_hash != next_style.content_hash {
      updates.push(next_style.virtual_id.clone());
    }
  }

  for (previous_block, next_block) in previous.custom_blocks.iter().zip(&next.custom_blocks) {
    if previous_block.content_hash != next_block.content_hash {
      updates.push(next_block.virtual_id.clone());
    }
  }

  if updates.is_empty() {
    None
  } else {
    Some(updates)
  }
}

pub fn content_hash(content: &str) -> u64 {
  let mut hasher = DefaultHasher::new();
  content.hash(&mut hasher);
  hasher.finish()
}
