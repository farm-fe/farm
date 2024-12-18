use farmfe_macro_cache_item::cache_item;

use serde::ser::SerializeStruct;

use crate::{
  module::{module_group::ModuleGroupId, ModuleId, ModuleType},
  HashSet,
};

use super::meta_data::ResourcePotMetaData;

#[cache_item]
#[derive(Clone)]
pub struct ResourcePot {
  pub id: ResourcePotId,
  pub name: String,
  pub resource_pot_type: ResourcePotType,
  pub modules: HashSet<ModuleId>,
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

impl serde::Serialize for ResourcePot {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    // serializer specific fields of ResourcePot
    let mut state = serializer.serialize_struct("ResourcePot", 2)?;
    state.serialize_field("id", &self.id)?;
    state.serialize_field("name", &self.name)?;
    state.serialize_field("resource_pot_type", &self.resource_pot_type)?;
    state.serialize_field("modules", &self.modules)?;
    state.serialize_field("entry_module", &self.entry_module)?;
    state.serialize_field("resources", &self.resources)?;
    state.serialize_field("module_groups", &self.module_groups)?;
    state.serialize_field("immutable", &self.immutable)?;
    state.end()
  }
}

impl ResourcePot {
  pub fn new(name: String, ty: ResourcePotType) -> Self {
    Self {
      id: Self::gen_id(&name, ty.clone()),
      name,
      resource_pot_type: ty,
      modules: HashSet::default(),
      meta: ResourcePotMetaData::default(),
      entry_module: None,
      resources: HashSet::default(),
      module_groups: HashSet::default(),
      immutable: false,
    }
  }

  pub fn gen_id(name: &str, ty: ResourcePotType) -> String {
    format!("{}_{}", name, ty.to_string())
  }

  pub fn set_resource_pot_id(&mut self, id: String) {
    self.id.clone_from(&id);
    // self.info.id = id;
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

  pub fn has_module(&self, module_id: &ModuleId) -> bool {
    self.modules.contains(module_id)
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
    format!("{self:?}").to_lowercase()
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
