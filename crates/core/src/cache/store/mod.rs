//! Cache store of the persistent cache, responsible for reading and writing the cache from the disk.

pub mod constant;
mod disk;
mod error;
mod memory;

pub use disk::*;

/// Cache key of the store, it's a pair of (name, cache_key), a name should only be related to one cache key.
/// Previous cache will be cleared if the related cache key changed for a name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheStoreKey {
  pub name: String,
  pub key: String,
}
