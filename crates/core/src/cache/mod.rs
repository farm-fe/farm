pub struct CacheManager {}

impl CacheManager {
  pub fn new() -> Self {
    Self {}
  }
}

impl Default for CacheManager {
  fn default() -> Self {
    Self::new()
  }
}
