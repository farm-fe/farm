use std::collections::HashSet;
use std::{collections::HashMap, sync::Arc};

use farmfe_macro_cache_item::cache_item;

use crate::module::{module_group::ModuleGroupId, ModuleId, ModuleType};

#[cache_item]
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
}

impl ResourcePot {
  pub fn new(name: String, ty: ResourcePotType) -> Self {
    Self {
      id: Self::gen_id(&name, ty.clone()),
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

pub type ResourcePotId = String;

#[cache_item]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RenderedModule {
  pub id: ModuleId,
  pub rendered_content: Arc<String>,
  pub rendered_map: Option<Arc<String>>,
  pub rendered_length: usize,
  pub original_length: usize,
}

#[cache_item]
#[derive(Clone)]
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
