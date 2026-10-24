//! Host-owned immutable resources shared by explicitly related compiler instances.
//!
//! Providers are passed through a compilation context, never recovered from a
//! process-global registry or an untyped pointer. Dynamic plugins must match
//! the exact Core ABI before they can access this field.
use std::sync::Arc;

use crate::error::Result;

/// A namespace-scoped read-only resource provider owned by the compiler host.
///
/// Returning an `Arc` leases these bytes independently of the host's JS handle.
/// A consumer must not mutate them or interpret an absent key as valid data.
pub trait SharedResourceProvider: Send + Sync {
  /// Obtain a resource from the explicit namespace. Errors must fail closed.
  fn get(&self, namespace: &str, key: &str) -> Result<Option<Arc<str>>>;
}
