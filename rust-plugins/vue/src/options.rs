//! User-facing options surface, deserialised from the JSON string Farm
//! forwards to the plugin constructor.
//!
//! Only the subset of `unplugin-vue`'s `Options` that the underlying
//! `fervid` compiler can honour today is exposed here (see the README /
//! docs page). Unsupported options are intentionally absent rather than
//! silently ignored.

use serde::Deserialize;

/// One pattern in an `include` / `exclude` list. Accepts either a plain
/// string (interpreted as a regex source) or a regex-like object with a
/// `source` field, matching `vite`'s `createFilter` behaviour.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PatternSource {
  String(String),
  /// Mirrors how a serialised JS `RegExp` arrives over JSON (e.g.
  /// `{ "source": "\\.vue$", "flags": "i" }`). `flags` is accepted for
  /// forward-compat but currently ignored — patterns are interpreted as
  /// Rust regex sources directly.
  Object {
    source: String,
    #[serde(default)]
    #[allow(dead_code)]
    flags: Option<String>,
  },
}

impl PatternSource {
  pub fn into_regex_source(self) -> String {
    match self {
      PatternSource::String(s) => s,
      PatternSource::Object { source, .. } => source,
    }
  }
}

/// Accepts a single pattern, an array of patterns, or omitted.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PatternList {
  Single(PatternSource),
  Many(Vec<PatternSource>),
}

impl PatternList {
  pub fn into_sources(self) -> Vec<String> {
    match self {
      PatternList::Single(p) => vec![p.into_regex_source()],
      PatternList::Many(ps) => ps.into_iter().map(|p| p.into_regex_source()).collect(),
    }
  }
}

/// Custom-element matcher. Can be either a boolean (all or nothing) or a
/// pattern list (matching files).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CustomElementMatcher {
  Boolean(bool),
  Patterns(PatternList),
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct VueFeaturesOptions {
  /// Defaults to `true`. Drives the `__VUE_OPTIONS_API__` define flag.
  pub options_api: Option<bool>,
  /// Defaults to `false`. Drives `__VUE_PROD_DEVTOOLS__`.
  pub prod_devtools: Option<bool>,
  /// Defaults to `false`. Drives `__VUE_PROD_HYDRATION_MISMATCH_DETAILS__`.
  pub prod_hydration_mismatch_details: Option<bool>,
  /// Enable reactive destructure for `defineProps`. Forwarded to fervid as
  /// `props_destructure`.
  pub props_destructure: Option<bool>,
  /// Files matching this pattern are compiled as custom elements. Defaults
  /// to `/\.ce\.vue$/`.
  pub custom_element: Option<CustomElementMatcher>,
}

/// Top-level options for `@farmfe/plugin-vue`.
///
/// Field names are camelCase to match the TypeScript surface defined in
/// `index.d.ts`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct VuePluginOptions {
  pub include: Option<PatternList>,
  pub exclude: Option<PatternList>,
  /// Force production mode regardless of Farm's compilation mode.
  pub is_production: Option<bool>,
  pub ssr: Option<bool>,
  pub source_map: Option<bool>,
  /// Deprecated: prefer `features.customElement`.
  pub custom_element: Option<CustomElementMatcher>,
  /// Whether to emit asset URL imports for tags like `<img src=…>`.
  /// `true` (default) enables fervid's defaults; `false` disables.
  pub transform_asset_urls: Option<bool>,
  pub features: Option<VueFeaturesOptions>,
}
