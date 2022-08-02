use rkyv::{Archive, Deserialize, Serialize};

use farm_macro_cache_item::cache_item;

pub mod resource_pot;
pub mod resource_pot_graph;

#[cache_item]
pub struct Resource {
  pub bytes: Vec<u8>,
}
