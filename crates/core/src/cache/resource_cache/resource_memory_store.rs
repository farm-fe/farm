use farmfe_macro_cache_item::cache_item;

use crate::{plugin::PluginGenerateResourcesHookResult, resource::meta_data::ResourcePotMetaData};

#[cache_item]
#[derive(Clone)]
pub struct CachedResourcePot {
  pub resources: PluginGenerateResourcesHookResult,
  pub meta: ResourcePotMetaData,
  /// hash of all modules' content hash
  pub hash: String,
}

pub trait ResourceMemoryStore {
  fn has_cache(&self, name: &str) -> bool;
  fn set_cache(&self, name: &str, resource: CachedResourcePot);
  fn get_cache(&self, name: &str) -> Option<CachedResourcePot>;
  /// Write the cache map to the disk.
  fn write_cache(&self);
}
