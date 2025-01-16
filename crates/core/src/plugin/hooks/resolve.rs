use farmfe_macro_cache_item::cache_item;

use crate::module::ModuleId;
use crate::HashMap;

#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cache_item]
pub enum ResolveKind {
  /// entry input in the config
  Entry(String),
  /// dynamic entry input, which will always be bundled as a separate resource
  /// all deep dependencies of a dynamic entry will be merged into a special module group
  DynamicEntry {
    name: String,
    /// the same as config.output.filename, default to config.output.filename
    output_filename: Option<String>,
    /// disable importer of the dynamic entry. for example if b.ts is a dynamic entry return by a.ts, then a.ts is the importer of b.ts by default,
    /// if you want a isolate dynamic entry, you can set this field to true
    no_importer: bool,
  },
  /// static import, e.g. `import a from './a'`
  #[default]
  Import,
  /// static export, e.g. `export * from './a'`
  ExportFrom,
  /// dynamic import, e.g. `import('./a').then(module => console.log(module))`
  DynamicImport,
  /// cjs require, e.g. `require('./a')`
  Require,
  /// @import of css, e.g. @import './a.css'
  CssAtImport,
  /// url() of css, e.g. url('./a.png')
  CssUrl,
  /// `<script src="./index.html" />` of html
  ScriptSrc,
  /// `<link href="index.css" />` of html
  LinkHref,
  /// Hmr update
  HmrUpdate,
  /// Custom ResolveKind, e.g. `const worker = new Worker(new Url("worker.js"))` of a web worker
  Custom(String),
}

impl ResolveKind {
  /// dynamic if self is [ResolveKind::DynamicImport] or [ResolveKind::Custom("dynamic:xxx")] (dynamic means the module is loaded dynamically, for example, fetch from network)
  /// used when analyzing module groups
  pub fn is_dynamic_import(&self) -> bool {
    matches!(self, ResolveKind::DynamicImport)
      || matches!(self, ResolveKind::Custom(c) if c.starts_with("dynamic:"))
  }

  pub fn is_dynamic_entry(&self) -> bool {
    matches!(self, ResolveKind::DynamicEntry { .. })
  }

  pub fn is_export_from(&self) -> bool {
    matches!(self, ResolveKind::ExportFrom)
  }

  pub fn is_require(&self) -> bool {
    matches!(self, ResolveKind::Require)
  }
}

impl From<&str> for ResolveKind {
  fn from(value: &str) -> Self {
    serde_json::from_str(value).unwrap()
  }
}

impl From<ResolveKind> for String {
  fn from(value: ResolveKind) -> Self {
    serde_json::to_string(&value).unwrap()
  }
}

/// Parameter of the resolve hook
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginResolveHookParam {
  /// the source would like to resolve, for example, './index'
  pub source: String,
  /// the start location to resolve `specifier`, being [None] if resolving a entry or resolving a hmr update.
  pub importer: Option<ModuleId>,
  /// for example, [ResolveKind::Import] for static import (`import a from './a'`)
  pub kind: ResolveKind,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginResolveHookResult {
  /// resolved path, normally a absolute file path.
  pub resolved_path: String,
  /// whether this module should be external, if true, the module won't present in the final result
  pub external: bool,
  /// whether this module has side effects, affects tree shaking. By default, it's true, means all modules may has side effects.
  /// use sideEffects field in package.json to mark it as side effects free
  pub side_effects: bool,
  /// the query parsed from specifier, for example, query should be `{ inline: "" }` if specifier is `./a.png?inline`
  /// if you custom plugins, your plugin should be responsible for parsing query
  /// if you just want a normal query parsing like the example above, [farmfe_toolkit::resolve::parse_query] should be helpful
  pub query: Vec<(String, String)>,
  /// the meta data passed between plugins and hooks
  pub meta: HashMap<String, String>,
}

impl Default for PluginResolveHookResult {
  fn default() -> Self {
    Self {
      side_effects: true,
      resolved_path: "unknown".to_string(),
      external: false,
      query: vec![],
      meta: Default::default(),
    }
  }
}
