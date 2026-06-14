//! SFC block virtual module registry.
//!
//! For every `<style>` block returned by fervid we synthesise an `id` of
//! the form `<module_id>?vue&type=style&idx=<N>&lang=<lang>&scoped=<bool>`
//! and remember its compiled (or raw) content. Custom blocks are exposed as
//! `<module_id>?vue&type=custom&idx=<N>&block=<tag>&lang=<tag>`. The `load`
//! hook serves these ids directly so downstream plugins can pick them up by
//! `module_type`.

use farmfe_core::module::ModuleType;
use farmfe_core::parking_lot::Mutex;
use fxhash::FxHashMap;

use crate::consts::{
  VUE_QUERY_BLOCK_KEY, VUE_QUERY_INDEX_KEY, VUE_QUERY_LANG_KEY, VUE_QUERY_SCOPED_KEY,
  VUE_QUERY_SCOPE_ID_KEY, VUE_QUERY_TYPE_CUSTOM, VUE_QUERY_TYPE_KEY, VUE_QUERY_TYPE_STYLE,
};

/// Lowercase comparison helper: a style block whose `lang` resolves to
/// vanilla CSS.
pub const CSS_LANGS: [&str; 1] = ["css"];

#[derive(Debug, Clone)]
pub struct StyleEntry {
  pub content: String,
  pub module_type: ModuleType,
}

#[derive(Default)]
pub struct StyleRegistry {
  inner: Mutex<FxHashMap<String, StyleEntry>>,
}

impl StyleRegistry {
  pub fn insert(&self, id: String, entry: StyleEntry) {
    self.inner.lock().insert(id, entry);
  }

  pub fn get(&self, id: &str) -> Option<StyleEntry> {
    self.inner.lock().get(id).cloned()
  }
}

/// Map a style block `lang` string to a Farm [`ModuleType`]. Native CSS is
/// emitted as [`ModuleType::Css`] so Farm's built-in CSS pipeline owns it;
/// everything else (scss, less, stylus, sass) is emitted as
/// [`ModuleType::Custom(lang)`] so a downstream Farm CSS preprocessor
/// plugin can claim it.
pub fn lang_to_module_type(lang: &str) -> ModuleType {
  let normalized = lang.trim().to_ascii_lowercase();
  if normalized.is_empty() || CSS_LANGS.contains(&normalized.as_str()) {
    ModuleType::Css
  } else {
    ModuleType::Custom(normalized)
  }
}

/// Build the virtual module id for a style block.
pub fn style_virtual_id(
  module_id: &str,
  idx: usize,
  lang: &str,
  scoped: bool,
  scope_id: Option<&str>,
) -> String {
  let normalized_lang = if lang.trim().is_empty() {
    "css".to_string()
  } else {
    lang.trim().to_ascii_lowercase()
  };
  let mut id = format!(
    "{module_id}?vue&{type_key}={type_value}&{idx_key}={idx}&{lang_key}={normalized_lang}&{scoped_key}={scoped}",
    type_key = VUE_QUERY_TYPE_KEY,
    type_value = VUE_QUERY_TYPE_STYLE,
    idx_key = VUE_QUERY_INDEX_KEY,
    lang_key = VUE_QUERY_LANG_KEY,
    scoped_key = VUE_QUERY_SCOPED_KEY,
  );

  if scoped
    && let Some(scope_id) = scope_id
  {
    id.push_str(&format!("&{VUE_QUERY_SCOPE_ID_KEY}={scope_id}"));
  }

  id
}

pub fn custom_block_virtual_id(module_id: &str, idx: usize, tag_name: &str) -> String {
  let normalized_tag = tag_name.trim().to_ascii_lowercase();
  format!(
    "{module_id}?vue&{type_key}={type_value}&{idx_key}={idx}&{block_key}={normalized_tag}&{lang_key}={normalized_tag}",
    type_key = VUE_QUERY_TYPE_KEY,
    type_value = VUE_QUERY_TYPE_CUSTOM,
    idx_key = VUE_QUERY_INDEX_KEY,
    block_key = VUE_QUERY_BLOCK_KEY,
    lang_key = VUE_QUERY_LANG_KEY,
  )
}
