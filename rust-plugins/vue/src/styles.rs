//! Style sub-block virtual module registry.
//!
//! For every `<style>` block returned by fervid we synthesise an `id` of
//! the form `<module_id>?vue&type=style&idx=<N>&lang=<lang>&scoped=<bool>`
//! and remember its compiled (or raw) content. The `load` hook serves these
//! ids directly so downstream CSS plugins (`@farmfe/plugin-sass`,
//! `postcss`, …) can pick them up by `module_type`.

use farmfe_core::module::ModuleType;
use farmfe_core::parking_lot::Mutex;
use fxhash::FxHashMap;

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
pub fn style_virtual_id(module_id: &str, idx: usize, lang: &str, scoped: bool) -> String {
  let normalized_lang = if lang.trim().is_empty() {
    "css".to_string()
  } else {
    lang.trim().to_ascii_lowercase()
  };
  format!("{module_id}?vue&type=style&idx={idx}&lang={normalized_lang}&scoped={scoped}")
}
