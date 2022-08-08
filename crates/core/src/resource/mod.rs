use rkyv::{Archive, Deserialize, Serialize};

use farm_macro_cache_item::cache_item;

pub mod resource_pot;
pub mod resource_pot_graph;

#[cache_item]
pub enum ResourceType {
  Js,
  Css,
  Html,
  SourceMap,
  Asset,
  Custom(String),
}

#[cache_item]
pub struct Resource {
  pub name: String,
  pub bytes: Vec<u8>,
  // whether this resource emitted, if true, it won't be emitted again by the default strategy.
  pub emitted: bool,
  pub resource_type: ResourceType,
}
