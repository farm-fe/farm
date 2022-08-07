use std::any::Any;

use downcast_rs::{impl_downcast, Downcast};
use farm_macro_cache_item::cache_item;
use hashbrown::HashSet;
use rkyv::{Archive, Archived, Deserialize, Serialize};
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;
use swc_ecma_ast::Module as SwcModule;

use crate::module::ModuleId;

#[cache_item]
pub struct ResourcePot {
  pub id: ResourcePotId,
  pub resource_pot_type: ResourcePotType,
  modules: HashSet<ModuleId>,
  pub meta: ResourcePotMetaData,
  pub entry_module: Option<ModuleId>,
}

impl ResourcePot {
  pub fn new(id: ResourcePotId) -> Self {
    Self {
      id,
      // TODO, set resource type
      resource_pot_type: ResourcePotType::Js,
      modules: HashSet::new(),
      meta: ResourcePotMetaData::Custom(Box::new(EmptyResourcePotMetaData) as _),
      entry_module: None,
    }
  }

  pub fn add_module(&mut self, module_id: ModuleId) {
    self.modules.insert(module_id);
  }

  pub fn modules(&self) -> Vec<&ModuleId> {
    self.modules.iter().collect()
  }

  pub fn remove_module(&mut self, module_id: &ModuleId) {
    self.modules.remove(module_id);
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

impl ResourcePotId {
  pub fn new(name: String) -> Self {
    Self { name }
  }
}

#[cache_item]
#[derive(Debug, Clone)]
pub enum ResourcePotType {
  Runtime,
  Js,
  Css,
  Html,
  Asset,
  Custom(String),
}

#[cache_item]
pub enum ResourcePotMetaData {
  Js(JsResourcePotMetaData),
  Custom(Box<dyn SerializeCustomResourcePotMetaData>),
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

#[archive_dyn(deserialize)]
pub trait CustomResourcePotMetaData: Any + Send + Sync + Downcast {}

impl_downcast!(SerializeCustomResourcePotMetaData);

#[cache_item(CustomResourcePotMetaData)]
pub struct EmptyResourcePotMetaData;
