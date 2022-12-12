use hashbrown::HashMap;
use parking_lot::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolveResultCacheKey {
  pub source: String,
  pub importer: Option<String>,
}

/// All cache related operation are charged by [CacheManager]
pub struct CacheManager {
  pub resolve_result_cache: Mutex<HashMap<ResolveResultCacheKey, String>>,
}

impl CacheManager {
  pub fn new() -> Self {
    Self {
      resolve_result_cache: Mutex::new(HashMap::new()),
    }
  }
}

impl Default for CacheManager {
  fn default() -> Self {
    Self::new()
  }
}
