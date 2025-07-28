use std::{hash::Hash, path::Path, sync::Arc};

use crate::HashSet;
use blake2::{
  digest::{Update, VariableOutput},
  Blake2bVar,
};
use farmfe_macro_cache_item::cache_item;
use farmfe_utils::{bytes_str::FarmBytesStr, relative};
use heck::AsLowerCamelCase;
pub use meta_data::{custom::CustomMetaDataMap, script::ModuleSystem, ModuleMetaData};
use relative_path::RelativePath;

use crate::{config::Mode, resource::resource_pot::ResourcePotId};

use self::module_group::ModuleGroupId;

pub mod meta_data;
pub mod module_graph;
pub mod module_group;
pub mod watch_graph;

pub const VIRTUAL_MODULE_PREFIX: &str = "virtual:";

/// A [Module] is a basic compilation unit
/// The [Module] is created by plugins in the parse hook of build stage
#[cache_item]
#[derive(Clone)]
pub struct Module {
  /// the id of this module, generated from the resolved id.
  pub id: ModuleId,
  /// the type of this module, for example [ModuleType::Js]
  pub module_type: ModuleType,
  /// the module groups this module belongs to, used to construct [crate::module::module_group::ModuleGroupGraph]
  pub module_groups: HashSet<ModuleGroupId>,
  /// the resource pot this module belongs to. A module may belongs multiple resource pots.
  pub resource_pots: HashSet<ResourcePotId>,
  /// the meta data of this module custom by plugins
  pub meta: Box<ModuleMetaData>,
  /// whether this module has side_effects
  pub side_effects: bool,
  /// the transformed source map chain of this module
  pub source_map_chain: Vec<Arc<String>>,
  /// whether this module marked as external
  pub external: bool,
  /// whether this module is immutable, for example, the module is immutable if it is from node_modules.
  /// This field will be set according to partialBundling.immutable of the user config, default to the module whose resolved_path contains ["/node_modules/"].
  pub immutable: bool,
  /// Execution order of this module in the module graph
  /// updated after the module graph is built
  pub execution_order: usize,
  /// Source size of this module
  pub size: usize,
  /// Source content after load and transform
  pub content: Arc<String>,
  /// Used exports of this module. Set by the tree-shake plugin
  pub used_exports: Vec<String>,
  /// last update timestamp
  pub last_update_timestamp: u128,
  /// content(after load and transform) hash
  pub content_hash: String,
  /// package name of this module
  pub package_name: String,
  /// package version of this module
  pub package_version: String,
  /// whether this module is a entry module
  pub is_entry: bool,
  /// whether this module is a dynamic entry module
  pub is_dynamic_entry: bool,

  // custom meta map
  pub custom: CustomMetaDataMap,
}

impl Default for Module {
  fn default() -> Self {
    Self::new(ModuleId::from(""))
  }
}

impl Module {
  pub fn new(id: ModuleId) -> Self {
    Self {
      id,
      module_type: ModuleType::Custom("__farm_unknown".to_string()),
      meta: Box::new(ModuleMetaData::default()),
      module_groups: HashSet::default(),
      resource_pots: Default::default(),
      side_effects: true,
      source_map_chain: vec![],
      external: false,
      immutable: false,
      // default to the last
      execution_order: usize::MAX,
      size: 0,
      content: Arc::new("".to_string()),
      used_exports: vec![],
      last_update_timestamp: 0,
      content_hash: "".to_string(),
      package_name: "".to_string(),
      package_version: "".to_string(),
      is_entry: false,
      is_dynamic_entry: false,
      custom: CustomMetaDataMap::default(),
    }
  }
}

/// Internal support module types by the core plugins,
/// other [ModuleType] will be set after the load hook, but can be change in transform hook too.
#[cache_item]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleType {
  // native supported module type by the core plugins
  Js,
  Jsx,
  Ts,
  Tsx,
  Css,
  Html,
  Asset,
  // Runtime,
  // custom module type from using by custom plugins
  Custom(String),
}

impl serde::Serialize for ModuleType {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(self.to_string().as_str())
  }
}

impl<'de> serde::Deserialize<'de> for ModuleType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = <std::string::String as serde::Deserialize>::deserialize(deserializer)?;
    Ok(s.into())
  }
}

impl ModuleType {
  pub fn is_typescript(&self) -> bool {
    matches!(self, ModuleType::Ts) || matches!(self, ModuleType::Tsx)
  }

  pub fn to_custom_css(&self, scope: &str) -> Option<ModuleType> {
    if self.is_css() && !matches!(self, Self::Custom(_)) {
      return Some(ModuleType::Custom(format!("farm_css:{scope}")));
    }

    None
  }

  pub fn to_custom_script(&self, scope: &str) -> Option<ModuleType> {
    if self.is_script() && !matches!(self, Self::Custom(_)) {
      return Some(ModuleType::Custom(format!("farm_script:{scope}")));
    }

    None
  }

  pub fn to_custom(&self, scope: &str) -> ModuleType {
    if let Some(css) = self.to_custom_css(scope) {
      return css;
    }

    if let Some(script) = self.to_custom_script(scope) {
      return script;
    }

    panic!("Unsupported module type: {self:?} when calling ModuleType::to_custom");
  }

  pub fn is_css(&self) -> bool {
    let mut m = matches!(self, ModuleType::Css);

    if !m && let ModuleType::Custom(s) = self {
      m = s.starts_with("farm_css:");
    }

    m
  }

  pub fn is_script(&self) -> bool {
    let mut m = matches!(
      self,
      ModuleType::Js | ModuleType::Jsx | ModuleType::Ts | ModuleType::Tsx
    );

    if !m && let ModuleType::Custom(s) = self {
      m = s.starts_with("farm_script:");
    }

    m
  }
}

impl ModuleType {
  /// transform native supported file type to [ModuleType]
  pub fn from_ext(ext: &str) -> Self {
    ext.into()
  }
}

impl<T: AsRef<str>> From<T> for ModuleType {
  fn from(s: T) -> Self {
    match s.as_ref() {
      "js" => Self::Js,
      "mjs" => Self::Js,
      "cjs" => Self::Js,
      "jsx" => Self::Jsx,
      "ts" => Self::Ts,
      "cts" => Self::Ts,
      "mts" => Self::Ts,
      "tsx" => Self::Tsx,
      "css" => Self::Css,
      "html" => Self::Html,
      "asset" => Self::Asset,
      // "runtime" => Self::Runtime,
      custom => Self::Custom(custom.to_string()),
    }
  }
}

impl ToString for ModuleType {
  fn to_string(&self) -> String {
    match *self {
      Self::Custom(ref s) => s.to_string(),
      _ => AsLowerCamelCase(format!("{self:?}")).to_string(),
    }
  }
}

/// Abstract ModuleId from the module's resolved id
#[cache_item]
#[derive(PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord)]
#[rkyv(derive(Hash, Eq, PartialEq))]
pub struct ModuleId {
  relative_path: FarmBytesStr,
  query_string: FarmBytesStr,
}

const LEN: usize = 4;

impl ModuleId {
  /// the resolved_path and query determine a module
  pub fn new(resolved_path: &str, query_string: &str, cwd: &str) -> Self {
    let rp = Path::new(resolved_path);
    let relative_path = if rp.is_absolute() {
      relative(cwd, resolved_path)
    } else {
      resolved_path.to_string()
    };

    Self {
      relative_path: FarmBytesStr::from(relative_path),
      query_string: FarmBytesStr::from(query_string.to_string()),
    }
  }

  pub fn from_resolved_path_with_query(resolved_path_with_query: &str, root: &str) -> Self {
    let rp = Path::new(resolved_path_with_query);
    let relative_path = if rp.is_absolute() {
      relative(root, resolved_path_with_query)
    } else {
      resolved_path_with_query.to_string()
    };
    relative_path.into()
  }

  /// return self.relative_path and self.query_string in dev,
  /// return hash(self.relative_path) in prod
  pub fn id(&self, mode: Mode) -> String {
    if let Ok(val) = std::env::var("FARM_DEBUG_ID")
      && !val.is_empty()
    {
      return self.to_string();
    }

    match mode {
      Mode::Development => self.to_string(),
      Mode::Production => self.hash(),
    }
  }

  /// transform the id back to relative path
  pub fn relative_path(&self) -> &str {
    self.relative_path.as_str()
  }

  pub fn query_string(&self) -> &str {
    self.query_string.as_str()
  }

  /// transform the id back to resolved path
  pub fn resolved_path(&self, root: &str) -> String {
    // if self.relative_path is absolute path, return it directly
    if Path::new(self.relative_path()).is_absolute()
      || self.relative_path().starts_with(VIRTUAL_MODULE_PREFIX)
    {
      return self.relative_path().to_string();
    }

    RelativePath::new(self.relative_path())
      .to_logical_path(root)
      .to_string_lossy()
      .to_string()
  }

  /// transform the id back to resolved path, with additional query
  pub fn resolved_path_with_query(&self, root: &str) -> String {
    format!("{}{}", self.resolved_path(root), self.query_string)
  }

  pub fn hash(&self) -> String {
    let mut hasher = Blake2bVar::new(LEN).unwrap();
    hasher.update(self.to_string().as_bytes());
    let mut buf = [0u8; LEN];
    hasher.finalize_variable(&mut buf).unwrap();
    hex::encode(buf)
  }

  fn split_query(p: &str) -> (String, String) {
    let (resolved_path, query) = p.split_once('?').unwrap_or((p, ""));

    if !query.is_empty() {
      return (resolved_path.to_string(), format!("?{query}"));
    }

    (p.to_string(), query.to_string())
  }
}

impl From<&str> for ModuleId {
  fn from(rp: &str) -> Self {
    let (rp, qs) = Self::split_query(rp);

    Self {
      relative_path: FarmBytesStr::from(rp),
      query_string: FarmBytesStr::from(qs),
    }
  }
}

impl From<String> for ModuleId {
  fn from(rp: String) -> Self {
    ModuleId::from(rp.as_str())
  }
}

impl ToString for ModuleId {
  fn to_string(&self) -> String {
    self.relative_path.to_string() + self.query_string.as_str()
  }
}

impl<'de> serde::Deserialize<'de> for ModuleId {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = <std::string::String as serde::Deserialize>::deserialize(deserializer)?;

    Ok(ModuleId::from(s))
  }
}

impl serde::Serialize for ModuleId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(self.to_string().as_str())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    config::Mode,
    module::{
      meta_data::{custom::CustomMetaDataMap, script::ModuleSystem},
      module_group::{ModuleGroupId, ModuleGroupType},
    },
    Cacheable,
  };
  use farmfe_macro_cache_item::cache_item;

  use super::{Module, ModuleId, ModuleMetaData, ModuleType};
  use crate::{HashMap, HashSet};

  #[test]
  fn module_type() {
    let suffix_vec = vec![
      ("js", ModuleType::Js),
      ("jsx", ModuleType::Jsx),
      ("ts", ModuleType::Ts),
      ("tsx", ModuleType::Tsx),
      ("css", ModuleType::Css),
      ("html", ModuleType::Html),
      ("scss", ModuleType::Custom("scss".to_string())),
      ("some", ModuleType::Custom("some".to_string())),
      ("Some", ModuleType::Custom("Some".to_string())),
      ("None", ModuleType::Custom("None".to_string())),
      ("SomeNone", ModuleType::Custom("SomeNone".to_string())),
    ];

    suffix_vec
      .into_iter()
      .for_each(|(suffix, t)| assert_eq!(suffix, t.to_string()));

    let suffix_vec = vec![
      ("cjs", ModuleType::Js),
      ("mjs", ModuleType::Js),
      ("cts", ModuleType::Ts),
      ("mts", ModuleType::Ts),
    ];

    suffix_vec.into_iter().for_each(|(suffix, t)| {
      assert!(ModuleType::from(suffix) == t);
    });
  }

  #[test]
  fn module_id() {
    #[cfg(not(target_os = "windows"))]
    let resolved_path = "/root/module.html";
    #[cfg(not(target_os = "windows"))]
    let module_id = ModuleId::new(resolved_path, "", "/root");
    #[cfg(not(target_os = "windows"))]
    let root = "/root";

    #[cfg(target_os = "windows")]
    let resolved_path = "C:\\root\\module.html";
    #[cfg(target_os = "windows")]
    let module_id = ModuleId::new(resolved_path, "", "C:\\root");
    #[cfg(target_os = "windows")]
    let root = "C:\\root";

    assert_eq!(module_id.id(Mode::Development), "module.html");
    assert_eq!(module_id.id(Mode::Production), "5de94ab0");
    assert_eq!(module_id.relative_path(), "module.html");
    assert_eq!(module_id.resolved_path(root), resolved_path);
    assert_eq!(module_id.hash(), "5de94ab0");

    #[cfg(not(target_os = "windows"))]
    let resolved_path = "/root/packages/test/module.html";
    #[cfg(not(target_os = "windows"))]
    let module_id = ModuleId::new(resolved_path, "", "/root/packages/app");

    #[cfg(target_os = "windows")]
    let resolved_path = "C:\\root\\packages\\test\\module.html";
    #[cfg(target_os = "windows")]
    let module_id = ModuleId::new(resolved_path, "", "C:\\root\\packages\\app");

    assert_eq!(module_id.id(Mode::Development), "../test/module.html");
  }

  #[test]
  fn module_id_with_query() {
    #[cfg(not(target_os = "windows"))]
    let resolved_path = "/root/logo.png";
    #[cfg(not(target_os = "windows"))]
    let module_id = ModuleId::new(resolved_path, "?inline", "/root");
    #[cfg(not(target_os = "windows"))]
    let root = "/root";

    #[cfg(target_os = "windows")]
    let resolved_path = "C:\\root\\logo.png";
    #[cfg(target_os = "windows")]
    let module_id = ModuleId::new(resolved_path, "?inline", "C:\\root");
    #[cfg(target_os = "windows")]
    let root = "C:\\root";

    assert_eq!(module_id.id(Mode::Development), "logo.png?inline");
    assert_eq!(module_id.id(Mode::Production), "f75a7043");
    assert_eq!(module_id.relative_path(), "logo.png");
    assert_eq!(module_id.resolved_path(root), resolved_path);
    assert_eq!(module_id.hash(), "f75a7043");
  }

  #[test]
  fn module_serialization() {
    println!("module serialization");
    let mut module = Module::new(ModuleId::new("/root/index.ts", "", "/root"));

    #[cache_item]
    #[derive(Default)]
    pub struct StructModuleData {
      ast: String,
      imports: Vec<String>,
    }

    module.module_groups = HashSet::from_iter([
      ModuleGroupId::new(&"1".into(), &ModuleGroupType::Entry),
      ModuleGroupId::new(&"2".into(), &ModuleGroupType::Entry),
    ]);

    module.meta = Box::new(ModuleMetaData::Custom(CustomMetaDataMap::from(
      HashMap::from_iter([(
        "custom".to_string(),
        Box::new(StructModuleData {
          ast: "ast".to_string(),
          imports: vec!["./index".to_string()],
        }) as Box<dyn Cacheable>,
      )]),
    )));

    let bytes = module.serialize_bytes().unwrap();
    let mut deserialized_module = module.deserialize_bytes(bytes).unwrap();
    let deserialized_module = deserialized_module.downcast_mut::<Module>().unwrap();

    assert_eq!(
      deserialized_module.id.relative_path(),
      module.id.relative_path()
    );

    assert_eq!(
      deserialized_module
        .meta
        .get_custom_mut::<StructModuleData>("custom")
        .ast,
      "ast"
    );

    assert_eq!(
      deserialized_module
        .meta
        .get_custom_mut::<StructModuleData>("custom")
        .imports,
      vec![String::from("./index")]
    );

    assert!(deserialized_module
      .module_groups
      .contains(&ModuleGroupId::new(&"1".into(), &ModuleGroupType::Entry)));
    assert!(deserialized_module
      .module_groups
      .contains(&ModuleGroupId::new(&"2".into(), &ModuleGroupType::Entry)));
  }

  #[test]
  fn module_system_merge() {
    assert_eq!(
      ModuleSystem::UnInitial.merge(ModuleSystem::EsModule),
      ModuleSystem::EsModule
    );

    assert_eq!(
      ModuleSystem::EsModule.merge(ModuleSystem::UnInitial),
      ModuleSystem::EsModule
    );

    assert_eq!(
      ModuleSystem::UnInitial.merge(ModuleSystem::CommonJs),
      ModuleSystem::CommonJs
    );

    assert_eq!(
      ModuleSystem::CommonJs.merge(ModuleSystem::UnInitial),
      ModuleSystem::CommonJs
    );

    let module_system = ModuleSystem::EsModule;

    assert_eq!(
      module_system.merge(ModuleSystem::CommonJs),
      ModuleSystem::Hybrid
    );

    let module_system = ModuleSystem::CommonJs;

    assert_eq!(
      module_system.merge(ModuleSystem::EsModule),
      ModuleSystem::Hybrid
    );

    let module_system = ModuleSystem::Hybrid;

    assert_eq!(
      module_system.merge(ModuleSystem::EsModule),
      ModuleSystem::Hybrid
    );

    assert_eq!(
      module_system.merge(ModuleSystem::CommonJs),
      ModuleSystem::Hybrid
    );

    let module_system = ModuleSystem::Custom("unknown".to_string());

    assert_eq!(
      module_system.merge(ModuleSystem::CommonJs),
      ModuleSystem::CommonJs
    );

    assert_eq!(
      module_system.merge(ModuleSystem::EsModule),
      ModuleSystem::EsModule
    );

    let module_system = ModuleSystem::CommonJs;

    assert_eq!(
      module_system.merge(ModuleSystem::Custom("unknown".to_string())),
      ModuleSystem::Custom("unknown".to_string())
    );
  }
}
