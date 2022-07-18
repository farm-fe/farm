use std::any::Any;

use rkyv::{Archive, Deserialize, Serialize};

use farm_macro_cache_item::cache_item;
use rkyv_dyn::archive_dyn;

use self::file::ResourceFile;

pub mod file;
pub mod resource_graph;

#[cache_item]
pub struct Resource {
  name: String,
  resource_type: ResourceType,
  meta: ResourceMetaData,
  file: ResourceFile,
}

#[cache_item]
pub enum ResourceType {
  Js,
  Css,
  Html,
  Asset,
  Custom(String),
}

#[cache_item]
pub enum ResourceMetaData {
  Js(JsResourceMetaData),
  Custom(Box<dyn SerializeCustomResourceMetaData>),
}

#[cache_item]
pub struct JsResourceMetaData {
  ast: String,
}

#[archive_dyn(deserialize)]
pub trait CustomResourceMetaData: Any + Send + Sync {}
