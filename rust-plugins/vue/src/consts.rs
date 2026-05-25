// Pattern matched against module ids; if a module id has a `?vue` query, it is a
// virtual sub-module (style and custom blocks; see `styles.rs`).
pub const VUE_QUERY_KEY: &str = "vue";
pub const VUE_QUERY_TYPE_KEY: &str = "type";
pub const VUE_QUERY_INDEX_KEY: &str = "idx";
pub const VUE_QUERY_LANG_KEY: &str = "lang";
pub const VUE_QUERY_SCOPED_KEY: &str = "scoped";
pub const VUE_QUERY_SCOPE_ID_KEY: &str = "scopeId";
pub const VUE_QUERY_BLOCK_KEY: &str = "block";

pub const VUE_QUERY_TYPE_STYLE: &str = "style";
pub const VUE_QUERY_TYPE_CUSTOM: &str = "custom";

/// Module type tag used in the load hook for the main `.vue` file so the
/// transform hook can pick it up.
pub const VUE_MODULE_TYPE: &str = "vue";

/// File suffix that auto-enables custom element compilation, matching
/// unplugin-vue's default `features.customElement: /\.ce\.vue$/`.
pub const CE_VUE_SUFFIX: &str = ".ce.vue";

/// Default extensions matched by the include filter when the user does not
/// supply one. Matches unplugin-vue's `include: /\.vue$/` default.
pub const DEFAULT_INCLUDE_PATTERN: &str = r"\.vue$";

/// Default custom element pattern, matching unplugin-vue's
/// `features.customElement: /\.ce\.vue$/`.
pub const DEFAULT_CUSTOM_ELEMENT_PATTERN: &str = r"\.ce\.vue$";
