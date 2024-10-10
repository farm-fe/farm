use serde::{Deserialize, Serialize};

use std::collections::HashSet;
use std::{collections::HashMap, sync::Arc};

use farmfe_macro_cache_item::cache_item;

use crate::module::{module_group::ModuleGroupId, ModuleId, ModuleType};

const DEFER_BUNDLE_MINIFY: &str = "DEFER_BUNDLE_MINIFY";

#[cache_item]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePot {
  pub id: ResourcePotId,
  pub name: String,
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
  pub info: Box<ResourcePotInfo>,
}

impl ResourcePot {
  pub fn new(name: String, ty: ResourcePotType) -> Self {
    Self {
      id: Self::gen_id(&name, ty.clone()),
      info: Box::new(ResourcePotInfo {
        id: Self::gen_id(&name, ty.clone()),
        name: name.clone(),
        resource_pot_type: ty.clone(),
        module_ids: vec![],
        map: None,
        modules: HashMap::new(),
        data: ResourcePotInfoData::Custom("{}".to_string()),
        custom: HashMap::new(),
      }),
      name,
      resource_pot_type: ty,
      modules: HashSet::new(),
      meta: ResourcePotMetaData::default(),
      entry_module: None,
      resources: HashSet::new(),
      module_groups: HashSet::new(),
      immutable: false,
    }
  }

  pub fn gen_id(name: &str, ty: ResourcePotType) -> String {
    format!("{}_{}", name, ty.to_string())
  }

  pub fn set_resource_pot_id(&mut self, id: String) {
    self.id.clone_from(&id);
    self.info.id = id;
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

  pub fn defer_minify_as_resource_pot(&mut self) {
    self
      .meta
      .custom_data
      .insert(DEFER_BUNDLE_MINIFY.to_string(), "true".to_string());
  }

  pub fn is_defer_minify_as_resource_pot(&self) -> bool {
    self
      .meta
      .custom_data
      .get(DEFER_BUNDLE_MINIFY)
      .is_some_and(|v| v == "true")
  }
}

pub type ResourcePotId = String;

#[cache_item]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourcePotType {
  Runtime,
  Js,
  Css,
  Html,
  Asset,
  Custom(String),
}

impl serde::Serialize for ResourcePotType {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(self.to_string().as_str())
  }
}

impl<'de> serde::Deserialize<'de> for ResourcePotType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = <std::string::String as serde::Deserialize>::deserialize(deserializer)?;
    Ok(s.into())
  }
}

#[cache_item]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePotInfo {
  pub id: ResourcePotId,
  pub name: String,
  pub resource_pot_type: ResourcePotType,
  pub module_ids: Vec<ModuleId>,
  pub map: Option<Arc<String>>,
  pub modules: HashMap<ModuleId, RenderedModule>,
  pub data: ResourcePotInfoData,
  pub custom: HashMap<String, String>,
}

impl ResourcePotInfo {
  pub fn new(resource_pot: &ResourcePot) -> Self {
    let data = match &resource_pot.resource_pot_type {
      ResourcePotType::Js => ResourcePotInfoData::Script(JsResourcePotInfo::new(resource_pot)),
      ResourcePotType::Css => ResourcePotInfoData::Css(CssResourcePotInfo {}),
      ResourcePotType::Html => ResourcePotInfoData::Html(HtmlResourcePotInfo {}),
      ResourcePotType::Runtime => ResourcePotInfoData::Custom("{}".to_string()),
      ResourcePotType::Asset => ResourcePotInfoData::Custom("{}".to_string()),
      ResourcePotType::Custom(_) => ResourcePotInfoData::Custom("{}".to_string()),
    };

    Self {
      id: resource_pot.id.clone(),
      resource_pot_type: resource_pot.resource_pot_type.clone(),
      name: resource_pot.name.clone(),
      module_ids: resource_pot.modules().into_iter().cloned().collect(),
      map: None,
      modules: resource_pot.meta.rendered_modules.clone(),
      data,
      custom: HashMap::new(),
    }
  }
}

/// Info data which is not shared by core plugins should be stored in [ResourcePotInfo::Custom]
#[cache_item]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ResourcePotInfoData {
  Script(JsResourcePotInfo),
  Css(CssResourcePotInfo),
  Html(HtmlResourcePotInfo),
  Custom(String),
}

impl ResourcePotInfoData {
  pub fn as_js(&self) -> &JsResourcePotInfo {
    match self {
      Self::Script(info) => info,
      _ => panic!("ResourcePotInfo is not ResourcePotInfo::Script"),
    }
  }

  pub fn as_css(&self) -> &CssResourcePotInfo {
    match self {
      Self::Css(info) => info,
      _ => panic!("ResourcePotInfo is not ResourcePotInfo::Css"),
    }
  }

  pub fn as_html(&self) -> &HtmlResourcePotInfo {
    match self {
      Self::Html(info) => info,
      _ => panic!("ResourcePotInfo is not ResourcePotInfo::Html"),
    }
  }

  pub fn as_custom(&self) -> &String {
    match self {
      Self::Custom(info) => info,
      _ => panic!("ResourcePotInfo is not ResourcePotInfo::Custom"),
    }
  }
}

#[cache_item]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsResourcePotInfo {
  pub dynamic_imports: Vec<String>,
  pub exports: Vec<String>,
  pub imports: Vec<String>,
  pub imported_bindings: HashMap<String, Vec<String>>,
  pub is_dynamic_entry: bool,
  pub is_entry: bool,
  pub is_implicit_entry: bool,
}

impl JsResourcePotInfo {
  pub fn new(resource_pot: &ResourcePot) -> Self {
    Self {
      dynamic_imports: vec![],           // TODO
      exports: vec![],                   // TODO
      imports: vec![],                   // TODO
      imported_bindings: HashMap::new(), // TODO
      is_dynamic_entry: false,
      is_entry: resource_pot.entry_module.is_some(),
      is_implicit_entry: false, // TODO
    }
  }
}

#[cache_item]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssResourcePotInfo {}

#[cache_item]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtmlResourcePotInfo {}

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

impl From<String> for ResourcePotType {
  fn from(s: String) -> Self {
    match s.as_str() {
      "runtime" => Self::Runtime,
      "js" => Self::Js,
      "css" => Self::Css,
      "html" => Self::Html,
      "asset" => Self::Asset,
      _ => Self::Custom(s),
    }
  }
}

#[cache_item]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderedModule {
  pub id: ModuleId,
  pub rendered_content: Arc<String>,
  pub rendered_map: Option<Arc<String>>,
  pub rendered_length: usize,
  pub original_length: usize,
}

#[cache_item]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePotMetaData {
  pub rendered_modules: HashMap<ModuleId, RenderedModule>,
  pub rendered_content: Arc<String>,
  pub rendered_map_chain: Vec<Arc<String>>,
  pub custom_data: HashMap<String, String>,
}

impl Default for ResourcePotMetaData {
  fn default() -> Self {
    Self {
      rendered_modules: HashMap::new(),
      rendered_content: Arc::new(String::new()),
      rendered_map_chain: vec![],
      custom_data: HashMap::new(),
    }
  }
}
