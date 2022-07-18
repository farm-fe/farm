use rkyv::{Archive, Deserialize, Serialize};

use farm_macro_cache_item::cache_item;

#[cache_item]
pub struct ResourceFile {}
