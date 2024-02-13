use dashmap::mapref::one::{Ref, RefMut};

use crate::module::{Module, ModuleId};

use super::CachedModule;

pub trait ModuleMemoryStore {
  fn is_cache_changed(&self, module: &Module) -> bool;
  fn has_cache(&self, key: &ModuleId) -> bool;
  fn set_cache(&self, key: ModuleId, module: CachedModule);
  fn get_cache(&self, key: &ModuleId) -> Option<CachedModule>;
  fn get_cache_ref(&self, key: &ModuleId) -> Option<Ref<'_, ModuleId, CachedModule>>;
  fn get_cache_mut_ref(&self, key: &ModuleId) -> Option<RefMut<'_, ModuleId, CachedModule>>;
  fn invalidate_cache(&self, key: &ModuleId);
  fn cache_outdated(&self, key: &ModuleId) -> bool;
  /// Write the cache map to the disk.
  fn write_cache(&self);
}
