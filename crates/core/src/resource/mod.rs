use rkyv::{Archive, Deserialize, Serialize};

use farmfe_macro_cache_item::cache_item;

use self::resource_pot::ResourcePotId;

pub mod resource_pot;
pub mod resource_pot_graph;

#[cache_item]
#[derive(Debug, Clone)]
pub enum ResourceType {
  Runtime,
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
  /// whether this resource emitted, true means this resource will not present in the final production
  pub emitted: bool,
  pub resource_type: ResourceType,
  /// the resource pot this [Resource] generated from
  pub resource_pot: ResourcePotId,
}
