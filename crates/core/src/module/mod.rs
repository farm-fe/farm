use std::{
  any::Any, cell::RefCell, collections::HashMap, hash::Hash, path::Path, rc::Rc, sync::Arc,
};

use blake2::{
  digest::{Update, VariableOutput},
  Blake2bVar,
};
use custom_meta_data::CustomMetaDataMap;
use downcast_rs::{impl_downcast, Downcast};
use farmfe_macro_cache_item::cache_item;
use farmfe_utils::relative;
use heck::AsLowerCamelCase;
use relative_path::RelativePath;
use rkyv::Deserialize;
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;
use std::collections::HashSet;
use swc_common::{
  comments::{
    Comment, SingleThreadedComments, SingleThreadedCommentsMap, SingleThreadedCommentsMapInner,
  },
  BytePos, DUMMY_SP,
};
use swc_css_ast::Stylesheet;
use swc_ecma_ast::Module as SwcModule;
use swc_html_ast::Document;

use crate::{config::Mode, resource::resource_pot::ResourcePotId, Cacheable};

use self::module_group::ModuleGroupId;

pub mod custom_meta_data;
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
  /// the resource pot this module belongs to
  pub resource_pot: Option<ResourcePotId>,
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
      meta: Box::new(ModuleMetaData::Custom(CustomMetaDataMap::default())),
      module_groups: HashSet::new(),
      resource_pot: None,
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
      custom: CustomMetaDataMap::default(),
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
  Custom(CustomMetaDataMap),
}

impl ToString for ModuleMetaData {
  fn to_string(&self) -> String {
    match self {
      Self::Script(_) => "script".to_string(),
      Self::Css(_) => "css".to_string(),
      Self::Html(_) => "html".to_string(),
      Self::Custom(_) => "custom".to_string(),
    }
  }
}

impl Clone for ModuleMetaData {
  fn clone(&self) -> Self {
    match self {
      Self::Script(script) => Self::Script(script.clone()),
      Self::Css(css) => Self::Css(css.clone()),
      Self::Html(html) => Self::Html(html.clone()),
      Self::Custom(custom) => {
        let mut custom_new = HashMap::new();
        for (k, v) in custom.iter() {
          let cloned_data = v.serialize_bytes().unwrap();
          let cloned_custom = v.deserialize_bytes(cloned_data).unwrap();
          custom_new.insert(k.clone(), cloned_custom);
        }
        Self::Custom(CustomMetaDataMap::from(custom_new))
      }
    }
  }
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
      panic!("ModuleMetaData is not Script but {:?}", self.to_string())
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

  /// get custom meta data by key
  pub fn get_custom_mut<T: Cacheable + Default>(&mut self, key: &str) -> &mut T {
    if let Self::Custom(custom) = self {
      custom.get_mut(key).unwrap()
    } else {
      panic!("ModuleMetaData is not Custom")
    }
  }
}

#[cache_item]
#[derive(Clone)]
pub struct CommentsMetaDataItem {
  pub byte_pos: BytePos,
  pub comment: Vec<Comment>,
}

#[cache_item]
#[derive(Clone, Default)]
pub struct CommentsMetaData {
  pub leading: Vec<CommentsMetaDataItem>,
  pub trailing: Vec<CommentsMetaDataItem>,
}

impl From<SingleThreadedComments> for CommentsMetaData {
  fn from(value: SingleThreadedComments) -> Self {
    let (swc_leading_map, swc_trailing_map) = value.take_all();
    let transform_comment_map = |map: SingleThreadedCommentsMap| {
      map
        .take()
        .into_iter()
        .map(|(byte_pos, comments)| CommentsMetaDataItem {
          byte_pos,
          comment: comments,
        })
        .collect::<Vec<CommentsMetaDataItem>>()
    };

    let leading = transform_comment_map(swc_leading_map);
    let trailing = transform_comment_map(swc_trailing_map);

    Self { leading, trailing }
  }
}

impl From<CommentsMetaData> for SingleThreadedComments {
  fn from(value: CommentsMetaData) -> Self {
    let transform_comment_map = |comments: Vec<CommentsMetaDataItem>| {
      Rc::new(RefCell::new(
        comments
          .into_iter()
          .map(|item| (item.byte_pos, item.comment))
          .collect::<SingleThreadedCommentsMapInner>(),
      ))
    };

    let leading = transform_comment_map(value.leading);
    let trailing = transform_comment_map(value.trailing);

    SingleThreadedComments::from_leading_and_trailing(leading, trailing)
  }
}

/// Script specific meta data, for example, [swc_ecma_ast::Module]
#[cache_item]
pub struct ScriptModuleMetaData {
  pub ast: SwcModule,
  pub top_level_mark: u32,
  pub unresolved_mark: u32,
  pub module_system: ModuleSystem,
  /// true if this module calls `import.meta.hot.accept()` or `import.meta.hot.accept(mod => {})`
  pub hmr_self_accepted: bool,
  pub hmr_accepted_deps: HashSet<ModuleId>,
  pub comments: CommentsMetaData,
  pub custom: CustomMetaDataMap,
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
      hmr_self_accepted: false,
      hmr_accepted_deps: Default::default(),
      comments: Default::default(),
      custom: Default::default(),
    }
  }
}

impl Clone for ScriptModuleMetaData {
  fn clone(&self) -> Self {
    let custom = if self.custom.is_empty() {
      HashMap::new()
    } else {
      let mut custom = HashMap::new();
      for (k, v) in self.custom.iter() {
        let cloned_data = v.serialize_bytes().unwrap();
        let cloned_custom = v.deserialize_bytes(cloned_data).unwrap();
        custom.insert(k.clone(), cloned_custom);
      }
      custom
    };

    Self {
      ast: self.ast.clone(),
      top_level_mark: self.top_level_mark,
      unresolved_mark: self.unresolved_mark,
      module_system: self.module_system.clone(),
      hmr_self_accepted: self.hmr_self_accepted,
      hmr_accepted_deps: self.hmr_accepted_deps.clone(),
      comments: self.comments.clone(),
      custom: CustomMetaDataMap::from(custom),
    }
  }
}

impl ScriptModuleMetaData {
  pub fn take_ast(&mut self) -> SwcModule {
    std::mem::replace(
      &mut self.ast,
      SwcModule {
        span: Default::default(),
        body: Default::default(),
        shebang: None,
      },
    )
  }

  pub fn set_ast(&mut self, ast: SwcModule) {
    self.ast = ast;
  }

  pub fn take_comments(&mut self) -> CommentsMetaData {
    std::mem::take(&mut self.comments)
  }

  pub fn set_comments(&mut self, comments: CommentsMetaData) {
    self.comments = comments;
  }

  pub fn is_cjs(&self) -> bool {
    matches!(self.module_system, ModuleSystem::CommonJs)
  }

  pub fn is_esm(&self) -> bool {
    matches!(self.module_system, ModuleSystem::EsModule)
  }

  pub fn is_hybrid(&self) -> bool {
    matches!(self.module_system, ModuleSystem::Hybrid)
  }
}

#[cache_item]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleSystem {
  UnInitial,
  EsModule,
  CommonJs,
  // Hybrid of commonjs and es-module
  Hybrid,
  Custom(String),
}

impl ModuleSystem {
  pub fn merge(&self, module_system: ModuleSystem) -> ModuleSystem {
    if matches!(module_system, ModuleSystem::UnInitial) {
      return self.clone();
    }

    match self {
      ModuleSystem::UnInitial => module_system,
      ModuleSystem::EsModule => {
        if matches!(module_system, ModuleSystem::CommonJs) {
          ModuleSystem::Hybrid
        } else {
          module_system
        }
      }

      ModuleSystem::CommonJs => {
        if matches!(module_system, ModuleSystem::EsModule) {
          ModuleSystem::Hybrid
        } else {
          module_system
        }
      }

      ModuleSystem::Hybrid => ModuleSystem::Hybrid,

      ModuleSystem::Custom(_) => module_system,
    }
  }
}

#[cache_item]
pub struct CssModuleMetaData {
  pub ast: Stylesheet,
  pub comments: CommentsMetaData,
  pub custom: CustomMetaDataMap,
}

impl Clone for CssModuleMetaData {
  fn clone(&self) -> Self {
    let custom = if self.custom.is_empty() {
      HashMap::new()
    } else {
      let mut custom = HashMap::new();
      for (k, v) in self.custom.iter() {
        let cloned_data = v.serialize_bytes().unwrap();
        let cloned_custom = v.deserialize_bytes(cloned_data).unwrap();
        custom.insert(k.clone(), cloned_custom);
      }
      custom
    };

    Self {
      ast: self.ast.clone(),
      comments: self.comments.clone(),
      custom: CustomMetaDataMap::from(custom),
    }
  }
}

impl CssModuleMetaData {
  pub fn take_ast(&mut self) -> Stylesheet {
    std::mem::replace(
      &mut self.ast,
      Stylesheet {
        span: DUMMY_SP,
        rules: vec![],
      },
    )
  }

  pub fn set_ast(&mut self, ast: Stylesheet) {
    self.ast = ast;
  }
}

#[cache_item]
pub struct HtmlModuleMetaData {
  pub ast: Document,
  pub custom: CustomMetaDataMap,
}

impl Clone for HtmlModuleMetaData {
  fn clone(&self) -> Self {
    let custom = if self.custom.is_empty() {
      HashMap::new()
    } else {
      let mut custom = HashMap::new();
      for (k, v) in self.custom.iter() {
        let cloned_data = v.serialize_bytes().unwrap();
        let cloned_custom = v.deserialize_bytes(cloned_data).unwrap();
        custom.insert(k.clone(), cloned_custom);
      }
      custom
    };

    Self {
      ast: self.ast.clone(),
      custom: CustomMetaDataMap::from(custom),
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
  Runtime,
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

  pub fn is_script(&self) -> bool {
    let mut m = matches!(
      self,
      ModuleType::Js | ModuleType::Jsx | ModuleType::Ts | ModuleType::Tsx
    );

    if !m {
      if let ModuleType::Custom(s) = self {
        m = s.starts_with("farm_script:");
      }
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
      "runtime" => Self::Runtime,
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
#[archive_attr(derive(Hash, Eq, PartialEq))]
pub struct ModuleId {
  relative_path: String,
  query_string: String,
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
      relative_path,
      query_string: query_string.to_string(),
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
    if let Ok(val) = std::env::var("FARM_DEBUG_ID") {
      if !val.is_empty() {
        return self.to_string();
      }
    }

    match mode {
      Mode::Development => self.to_string(),
      Mode::Production => self.hash(),
    }
  }

  /// transform the id back to relative path
  pub fn relative_path(&self) -> &str {
    &self.relative_path
  }

  pub fn query_string(&self) -> &str {
    &self.query_string
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
  use crate::{config::Mode, module::custom_meta_data::CustomMetaDataMap};
  use downcast_rs::Downcast;
  use farmfe_macro_cache_item::cache_item;
  use rkyv_dyn::archive_dyn;
  use rkyv_typename::TypeName;
  use std::collections::{HashMap, HashSet};

  use super::{Cacheable, Module, ModuleId, ModuleMetaData, ModuleSystem, ModuleType};

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
    let mut module = Module::new(ModuleId::new("/root/index.ts", "", "/root"));

    #[cache_item]
    #[derive(Default)]
    pub struct StructModuleData {
      ast: String,
      imports: Vec<String>,
    }

    module.module_groups = HashSet::from([ModuleId::new("1", "", ""), ModuleId::new("2", "", "")]);

    module.meta = Box::new(ModuleMetaData::Custom(CustomMetaDataMap::from(
      HashMap::from([(
        "custom".to_string(),
        Box::new(StructModuleData {
          ast: "ast".to_string(),
          imports: vec!["./index".to_string()],
        }) as Box<dyn Cacheable>,
      )]),
    )));

    // let mut v = Box::new(StructModuleData {
    //   ast: String::from("ast"),
    //   imports: vec![String::from("./index")],
    // }) as Box<dyn Cacheable>;

    // let value = v.as_any_mut().downcast_mut::<StructModuleData>().unwrap();
    // let module = std::mem::take(value);

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
      .contains(&ModuleId::new("1", "", "")));
    assert!(deserialized_module
      .module_groups
      .contains(&ModuleId::new("2", "", "")));
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
