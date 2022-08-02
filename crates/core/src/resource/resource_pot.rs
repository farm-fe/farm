use std::any::Any;

use rkyv::{Archive, Deserialize, Serialize};

use farm_macro_cache_item::cache_item;
use rkyv_dyn::archive_dyn;

#[cache_item]
pub struct ResourcePot {
  pub name: String,
  pub resource_pot_type: ResourcePotType,
  pub meta: ResourcePotMetaData,
}

#[cache_item]
#[derive(Debug, Clone)]
pub enum ResourcePotType {
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

#[cache_item]
pub struct JsResourcePotMetaData {
  ast: String,
}

#[archive_dyn(deserialize)]
pub trait CustomResourcePotMetaData: Any + Send + Sync {}
