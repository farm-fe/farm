use std::{any::Any, char::ToLowercase};

use downcast_rs::{impl_downcast, Downcast};
use farmfe_macro_cache_item::cache_item;
use hashbrown::HashSet;
use rkyv::{Archive, Archived, Deserialize, Serialize};
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;
use swc_css_ast::Stylesheet;
use swc_ecma_ast::Module as SwcModule;
use swc_html_ast::Document;

use crate::module::{module_group::ModuleGroupId, ModuleId, ModuleType};

#[cache_item]
pub struct ResourcePot {
  pub id: ResourcePotId,
  pub resource_pot_type: ResourcePotType,
  modules: HashSet<ModuleId>,
  pub meta: ResourcePotMetaData,
  /// [None] if this [ResourcePot] does not contain entry module.
  /// [Some(entry_id)] otherwise
  pub entry_module: Option<ModuleId>,
  /// the resources generated in this [ResourcePot]
  resources: HashSet<String>,

  /// This field should be filled in partial_bundling_hooks.
  /// the module groups that this [ResourcePot] belongs to.
  /// A [ResourcePot] can belong to multiple module groups.
  pub module_groups: HashSet<ModuleGroupId>,
  pub immutable: bool,
}

impl ResourcePot {
  pub fn new(id: ResourcePotId, ty: ResourcePotType) -> Self {
    Self {
      id,
      resource_pot_type: ty,
      modules: HashSet::new(),
      meta: ResourcePotMetaData::default(),
      entry_module: None,
      resources: HashSet::new(),
      module_groups: HashSet::new(),
      immutable: false,
    }
  }

  pub fn add_module(&mut self, module_id: ModuleId) {
    self.modules.insert(module_id);
  }

  pub fn modules(&self) -> Vec<&ModuleId> {
    let mut modules = self.modules.iter().collect::<Vec<&ModuleId>>();
    // sort by module id
    modules.sort_by_key(|m| m.to_string());

    modules
  }

  pub fn take_meta(&mut self) -> ResourcePotMetaData {
    std::mem::take(&mut self.meta)
  }

  pub fn remove_module(&mut self, module_id: &ModuleId) {
    self.modules.remove(module_id);
  }

  pub fn add_resource(&mut self, name: String) {
    self.resources.insert(name);
  }

  pub fn resources(&self) -> Vec<&String> {
    self.resources.iter().collect()
  }

  pub fn remove_resource(&mut self, name: &String) {
    self.resources.remove(name);
  }

  pub fn clear_resources(&mut self) {
    self.resources.clear();
  }
}

#[cache_item]
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ResourcePotId {
  name: String,
}

impl ToString for ResourcePotId {
  fn to_string(&self) -> String {
    self.name.clone()
  }
}

impl From<&str> for ResourcePotId {
  fn from(n: &str) -> Self {
    Self {
      name: n.to_string(),
    }
  }
}

impl ResourcePotId {
  pub fn new(name: String) -> Self {
    Self { name }
  }
}

#[cache_item]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourcePotType {
  Runtime,
  Js,
  Css,
  Html,
  Asset,
  Custom(String),
}

impl From<ModuleType> for ResourcePotType {
  fn from(m_ty: ModuleType) -> Self {
    match m_ty {
      ModuleType::Js | ModuleType::Jsx | ModuleType::Ts | ModuleType::Tsx => Self::Js,
      ModuleType::Css => Self::Css,
      ModuleType::Html => Self::Html,
      ModuleType::Asset => Self::Asset,
      ModuleType::Runtime => Self::Runtime,
      ModuleType::Custom(c) => Self::Custom(c),
    }
  }
}

impl ToString for ResourcePotType {
  fn to_string(&self) -> String {
    format!("{:?}", self).to_lowercase()
  }
}

#[cache_item]
pub enum ResourcePotMetaData {
  Js(JsResourcePotMetaData),
  Css(CssResourcePotMetaData),
  Html(HtmlResourcePotMetaData),
  Custom(Box<dyn SerializeCustomResourcePotMetaData>),
}

impl Default for ResourcePotMetaData {
  fn default() -> Self {
    Self::Custom(Box::new(EmptyResourcePotMetaData) as _)
  }
}

impl ResourcePotMetaData {
  pub fn as_js(&self) -> &JsResourcePotMetaData {
    match self {
      ResourcePotMetaData::Js(r) => r,
      _ => panic!("ResourcePotMetaData is not js!"),
    }
  }

  pub fn as_js_mut(&mut self) -> &mut JsResourcePotMetaData {
    match self {
      ResourcePotMetaData::Js(r) => r,
      _ => panic!("ResourcePotMetaData is not js!"),
    }
  }

  pub fn take_js(self) -> JsResourcePotMetaData {
    match self {
      ResourcePotMetaData::Js(r) => r,
      _ => panic!("ResourcePotMetaData is not js!"),
    }
  }

  pub fn as_css(&self) -> &CssResourcePotMetaData {
    match self {
      ResourcePotMetaData::Css(r) => r,
      _ => panic!("ResourcePotMetaData is not css!"),
    }
  }

  pub fn as_css_mut(&mut self) -> &mut CssResourcePotMetaData {
    match self {
      ResourcePotMetaData::Css(r) => r,
      _ => panic!("ResourcePotMetaData is not css!"),
    }
  }
  pub fn as_html(&self) -> &HtmlResourcePotMetaData {
    match self {
      ResourcePotMetaData::Html(r) => r,
      _ => panic!("ResourcePotMetaData is not html!"),
    }
  }

  pub fn as_html_mut(&mut self) -> &mut HtmlResourcePotMetaData {
    match self {
      ResourcePotMetaData::Html(r) => r,
      _ => panic!("ResourcePotMetaData is not html!"),
    }
  }

  pub fn as_custom<T: SerializeCustomResourcePotMetaData>(&self) -> &T {
    match self {
      ResourcePotMetaData::Custom(c) => {
        if let Some(c) = c.downcast_ref::<T>() {
          c
        } else {
          panic!("Custom resource meta data is not serializable!");
        }
      }
      _ => panic!("ResourcePotMetaData is not custom!"),
    }
  }

  pub fn as_custom_mut<T: SerializeCustomResourcePotMetaData>(&mut self) -> &mut T {
    match self {
      ResourcePotMetaData::Custom(c) => {
        if let Some(c) = c.downcast_mut::<T>() {
          c
        } else {
          panic!("Custom resource meta data is not serializable!");
        }
      }
      _ => panic!("ResourcePotMetaData is not custom!"),
    }
  }
}

#[cache_item]
pub struct JsResourcePotMetaData {
  pub ast: SwcModule,
}

#[cache_item]
pub struct CssResourcePotMetaData {
  pub ast: Stylesheet,
}

#[cache_item]
pub struct HtmlResourcePotMetaData {
  pub ast: Document,
}

#[archive_dyn(deserialize)]
pub trait CustomResourcePotMetaData: Any + Send + Sync + Downcast {}

impl_downcast!(SerializeCustomResourcePotMetaData);

#[cache_item(CustomResourcePotMetaData)]
pub struct EmptyResourcePotMetaData;
