//! Cache store of the persistent cache, responsible for reading and writing the cache from the disk.

pub mod constant;
mod disk;
mod error;
pub mod memory;

pub use disk::*;

/// Cache key of the store, it's a pair of (name, cache_key), a name should only be related to one cache key.
/// Previous cache will be cleared if the related cache key changed for a name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheStoreKey {
  pub name: String,
  pub key: String,
}

impl<A1: ToString, A2: ToString> From<(A1, A2)> for CacheStoreKey {
  fn from((name, key): (A1, A2)) -> Self {
    Self {
      name: name.to_string(),
      key: key.to_string(),
    }
  }
}
