use std::{any::Any, hash::Hash, path::Path};

use blake2::{
  digest::{Update, VariableOutput},
  Blake2bVar,
};
use downcast_rs::{impl_downcast, Downcast};
use farmfe_macro_cache_item::cache_item;
use farmfe_utils::relative;
use hashbrown::HashSet;
use relative_path::RelativePath;
use rkyv::{Archive, Archived, Deserialize, Serialize};
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;
use swc_css_ast::Stylesheet;
use swc_ecma_ast::Module as SwcModule;
use swc_html_ast::Document;

use crate::{config::Mode, resource::resource_pot::ResourcePotId};

use self::module_group::ModuleGroupId;

pub mod module_graph;
pub mod module_group;

/// A [Module] is a basic compilation unit
/// The [Module] is created by plugins in the parse hook of build stage
#[cache_item]
pub struct Module {
  /// the id of this module, generated from the resolved id.
  pub id: ModuleId,
  /// the type of this module, for example [ModuleType::Js]
  pub module_type: ModuleType,
  /// the module groups this module belongs to, used to construct [crate::module::module_group::ModuleGroupGraph]
  pub module_groups: HashSet<ModuleGroupId>,
  /// the resource pot this module belongs to
  pub resource_pot: Option<ResourcePotId>,
  /// the meta data of this module custom by plugins
  pub meta: ModuleMetaData,
  /// whether this module has side_effects
  pub side_effects: bool,
  /// the transformed source map chain of this module
  pub source_map_chain: Vec<String>,
  /// whether this module marked as external
  pub external: bool,
  /// whether this module is immutable, for example, the module is immutable if it is from node_modules.
  /// This field will be set according to partialBundling.immutable of the user config, default to the module whose resolved_path contains ["/node_modules/"].
  pub immutable: bool,
}

impl Module {
  pub fn new(id: ModuleId) -> Self {
    Self {
      id,
      module_type: ModuleType::Custom("__farm_unknown".to_string()),
      meta: ModuleMetaData::Custom(Box::new(EmptyModuleMetaData) as _),
      module_groups: HashSet::new(),
      resource_pot: None,
      side_effects: false,
      source_map_chain: vec![],
      external: false,
      immutable: false,
    }
  }
}

/// Module meta data shared by core plugins through the compilation
/// Meta data which is not shared by core plugins should be stored in [ModuleMetaData::Custom]
#[cache_item]
pub enum ModuleMetaData {
  Script(ScriptModuleMetaData),
  Css(CssModuleMetaData),
  Html(HtmlModuleMetaData),
  Custom(Box<dyn SerializeCustomModuleMetaData>),
}

impl ModuleMetaData {
  pub fn as_script_mut(&mut self) -> &mut ScriptModuleMetaData {
    if let Self::Script(script) = self {
      script
    } else {
      panic!("ModuleMetaData is not Script")
    }
  }

  pub fn as_script(&self) -> &ScriptModuleMetaData {
    if let Self::Script(script) = self {
      script
    } else {
      panic!("ModuleMetaData is not Script")
    }
  }

  pub fn as_css(&self) -> &CssModuleMetaData {
    if let Self::Css(css) = self {
      css
    } else {
      panic!("ModuleMetaData is not css")
    }
  }

  pub fn as_css_mut(&mut self) -> &mut CssModuleMetaData {
    if let Self::Css(css) = self {
      css
    } else {
      panic!("ModuleMetaData is not css")
    }
  }

  pub fn as_html(&self) -> &HtmlModuleMetaData {
    if let Self::Html(html) = self {
      html
    } else {
      panic!("ModuleMetaData is not html")
    }
  }

  pub fn as_html_mut(&mut self) -> &mut HtmlModuleMetaData {
    if let Self::Html(html) = self {
      html
    } else {
      panic!("ModuleMetaData is not html")
    }
  }

  pub fn as_custom_mut<T: SerializeCustomModuleMetaData + 'static>(&mut self) -> &mut T {
    if let Self::Custom(custom) = self {
      if let Some(c) = custom.downcast_mut::<T>() {
        c
      } else {
        panic!("custom meta type is not serializable");
      }
    } else {
      panic!("ModuleMetaData is not Custom")
    }
  }

  pub fn as_custom<T: SerializeCustomModuleMetaData + 'static>(&self) -> &T {
    if let Self::Custom(custom) = self {
      if let Some(c) = custom.downcast_ref::<T>() {
        c
      } else {
        panic!("custom meta type is not serializable");
      }
    } else {
      panic!("ModuleMetaData is not Custom")
    }
  }
}

/// Trait that makes sure the trait object implements [rkyv::Serialize] and [rkyv::Deserialize]
#[archive_dyn(deserialize)]
pub trait CustomModuleMetaData: Any + Send + Sync + Downcast {}

impl_downcast!(SerializeCustomModuleMetaData);

/// initial empty custom data, plugins may replace this
#[cache_item(CustomModuleMetaData)]
pub struct EmptyModuleMetaData;

/// Script specific meta data, for example, [swc_ecma_ast::Module]
#[cache_item]
pub struct ScriptModuleMetaData {
  pub ast: SwcModule,
  pub top_level_mark: u32,
  pub unresolved_mark: u32,
  pub module_system: ModuleSystem,
  /// true if this module calls `import.meta.hot.accept`
  pub hmr_accepted: bool,
}

impl Default for ScriptModuleMetaData {
  fn default() -> Self {
    Self {
      ast: SwcModule {
        span: Default::default(),
        body: Default::default(),
        shebang: None,
      },
      top_level_mark: 0,
      unresolved_mark: 0,
      module_system: ModuleSystem::EsModule,
      hmr_accepted: false,
    }
  }
}

#[cache_item]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleSystem {
  EsModule,
  CommonJs,
  // Hybrid of commonjs and es-module
  Hybrid,
  Custom(String),
}

#[cache_item]
pub struct CssModuleMetaData {
  pub ast: Stylesheet,
}

#[cache_item]
pub struct HtmlModuleMetaData {
  pub ast: Document,
}

/// Internal support module types by the core plugins,
/// other [ModuleType] will be set after the load hook, but can be change in transform hook too.
#[cache_item]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModuleType {
  // native supported module type by the core plugins
  Js,
  Jsx,
  Ts,
  Tsx,
  Css,
  Html,
  Asset,
  Runtime,
  // custom module type from using by custom plugins
  Custom(String),
}

impl ModuleType {
  pub fn is_typescript(&self) -> bool {
    matches!(self, ModuleType::Ts) || matches!(self, ModuleType::Tsx)
  }

  pub fn is_script(&self) -> bool {
    matches!(
      self,
      ModuleType::Js | ModuleType::Jsx | ModuleType::Ts | ModuleType::Tsx
    )
  }
}

impl ModuleType {
  /// transform native supported file type to [ModuleType]
  pub fn from_ext(ext: &str) -> Self {
    match ext {
      "js" => Self::Js,
      "jsx" => Self::Jsx,
      "ts" => Self::Ts,
      "tsx" => Self::Tsx,
      "css" => Self::Css,
      "html" => Self::Html,
      custom => Self::Custom(custom.to_string()),
    }
  }
}

impl ToString for ModuleType {
  fn to_string(&self) -> String {
    format!("{:?}", self)
  }
}

/// Abstract ModuleId from the module's resolved id
#[cache_item]
#[derive(
  PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[archive_attr(derive(Hash, Eq, PartialEq))]
#[serde(rename_all = "camelCase")]
pub struct ModuleId {
  relative_path: String,
  query_string: Option<String>,
}

const LEN: usize = 4;

impl ModuleId {
  /// the resolved_path and query determine a module
  pub fn new(resolved_path: &str, cwd: &str) -> Self {
    let (resolved_path, qs) = Self::split_query(resolved_path);

    let rp = Path::new(resolved_path.as_str());
    let relative_path = if rp.is_absolute() {
      relative(cwd, resolved_path.as_str())
    } else {
      resolved_path.to_string()
    };

    Self {
      relative_path,
      query_string: qs,
    }
  }

  /// return self.relative_path and self.query_string in dev,
  /// return hash(self.relative_path) in prod
  pub fn id(&self, mode: Mode) -> String {
    match mode {
      Mode::Development => self.to_string(),
      Mode::Production => self.hash(),
    }
  }

  /// transform the id back to relative path
  pub fn relative_path(&self) -> &str {
    &self.relative_path
  }

  /// transform the id back to resolved path
  pub fn resolved_path(&self, root: &str) -> String {
    RelativePath::new(self.relative_path())
      .to_logical_path(root)
      .to_string_lossy()
      .to_string()
  }

  pub fn hash(&self) -> String {
    let mut hasher = Blake2bVar::new(LEN).unwrap();
    hasher.update(self.to_string().as_bytes());
    let mut buf = [0u8; LEN];
    hasher.finalize_variable(&mut buf).unwrap();
    hex::encode(buf)
  }

  fn split_query(p: &str) -> (String, Option<String>) {
    let comps = p.split('?').collect::<Vec<&str>>();

    if comps.len() == 2 {
      return (comps[0].to_string(), Some(format!("?{}", comps[1])));
    }

    (p.to_string(), None)
  }
}

impl From<&str> for ModuleId {
  fn from(rp: &str) -> Self {
    let (rp, qs) = Self::split_query(rp);

    Self {
      relative_path: rp,
      query_string: qs,
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
    self.relative_path.to_string() + self.query_string.as_deref().unwrap_or("")
  }
}

#[cfg(test)]
mod tests {

  use crate::config::Mode;
  use farmfe_macro_cache_item::cache_item;
  use hashbrown::HashSet;
  use rkyv::{Archive, Archived, Deserialize, Serialize};
  use rkyv_dyn::archive_dyn;
  use rkyv_typename::TypeName;

  use super::{
    CustomModuleMetaData, DeserializeCustomModuleMetaData, Module, ModuleId, ModuleMetaData,
    SerializeCustomModuleMetaData,
  };

  #[test]
  fn module_id() {
    #[cfg(not(target_os = "windows"))]
    let resolved_path = "/root/module.html";
    #[cfg(not(target_os = "windows"))]
    let module_id = ModuleId::new(resolved_path, "/root");
    #[cfg(not(target_os = "windows"))]
    let root = "/root";

    #[cfg(target_os = "windows")]
    let resolved_path = "C:\\root\\module.html";
    #[cfg(target_os = "windows")]
    let module_id = ModuleId::new(resolved_path, "C:\\root");
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
    let module_id = ModuleId::new(resolved_path, "/root/packages/app");

    #[cfg(target_os = "windows")]
    let resolved_path = "C:\\root\\packages\\test\\module.html";
    #[cfg(target_os = "windows")]
    let module_id = ModuleId::new(resolved_path, "C:\\root\\packages\\app");

    assert_eq!(module_id.id(Mode::Development), "../test/module.html");
  }

  #[test]
  fn module_id_with_query() {
    #[cfg(not(target_os = "windows"))]
    let resolved_path = "/root/logo.png?inline";
    #[cfg(not(target_os = "windows"))]
    let module_id = ModuleId::new(resolved_path, "/root");
    #[cfg(not(target_os = "windows"))]
    let root = "/root";

    #[cfg(target_os = "windows")]
    let resolved_path = "C:\\root\\logo.png?inline";
    #[cfg(target_os = "windows")]
    let module_id = ModuleId::new(resolved_path, "C:\\root");
    #[cfg(target_os = "windows")]
    let root = "C:\\root";

    assert_eq!(module_id.id(Mode::Development), "logo.png?inline");
    assert_eq!(module_id.id(Mode::Production), "f75a7043");
    assert_eq!(module_id.relative_path(), "logo.png");
    assert_eq!(
      module_id.resolved_path(root),
      resolved_path.split("?").next().unwrap()
    );
    assert_eq!(module_id.hash(), "f75a7043");
  }

  #[test]
  fn module_serialization() {
    let mut module = Module::new(ModuleId::new("/root/index.ts", "/root"));

    #[cache_item(CustomModuleMetaData)]
    struct StructModuleData {
      ast: String,
      imports: Vec<String>,
    }

    module.module_groups = HashSet::from([ModuleId::new("1", ""), ModuleId::new("2", "")]);

    module.meta = ModuleMetaData::Custom(Box::new(StructModuleData {
      ast: String::from("ast"),
      imports: vec![String::from("./index")],
    }) as _);

    let bytes = rkyv::to_bytes::<_, 256>(&module).unwrap();

    let archived = unsafe { rkyv::archived_root::<Module>(&bytes[..]) };
    let mut deserialized_module: Module = archived
      .deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new())
      .unwrap();

    assert_eq!(
      deserialized_module.id.relative_path(),
      module.id.relative_path()
    );

    assert_eq!(
      deserialized_module
        .meta
        .as_custom_mut::<StructModuleData>()
        .ast,
      "ast"
    );

    assert_eq!(
      deserialized_module
        .meta
        .as_custom::<StructModuleData>()
        .imports,
      vec![String::from("./index")]
    );

    assert!(deserialized_module
      .module_groups
      .contains(&ModuleId::new("1", "")));
    assert!(deserialized_module
      .module_groups
      .contains(&ModuleId::new("2", "")));
  }
}
