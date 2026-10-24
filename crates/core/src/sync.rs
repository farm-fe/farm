//! Locks for state shared with dynamically loaded Rust plugins.
//!
//! parking_lot keeps its wait queues in library-local static state. A plugin
//! can therefore release a lock without waking a waiter parked by the host.
//! The standard library's platform locks wake by the shared lock address.
//! Host and plugin still need matching Farm ABI and Rust toolchain versions.

use std::sync::{LockResult, PoisonError, TryLockError, TryLockResult};
pub use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

fn unpoison<T>(result: LockResult<T>) -> T {
  // Preserve the previous poison-free API; compilation errors are handled by
  // the compiler rather than turning subsequent lock access into a new panic.
  result.unwrap_or_else(PoisonError::into_inner)
}

fn try_unpoison<T>(result: TryLockResult<T>) -> Option<T> {
  match result {
    Ok(guard) => Some(guard),
    Err(TryLockError::Poisoned(error)) => Some(error.into_inner()),
    Err(TryLockError::WouldBlock) => None,
  }
}

#[derive(Debug, Default)]
pub struct Mutex<T: ?Sized>(std::sync::Mutex<T>);

impl<T> Mutex<T> {
  pub fn new(value: T) -> Self {
    Self(std::sync::Mutex::new(value))
  }
}

impl<T: ?Sized> Mutex<T> {
  pub fn lock(&self) -> MutexGuard<'_, T> {
    unpoison(self.0.lock())
  }

  pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
    try_unpoison(self.0.try_lock())
  }
}

#[derive(Debug, Default)]
pub struct RwLock<T: ?Sized>(std::sync::RwLock<T>);

impl<T> RwLock<T> {
  pub fn new(value: T) -> Self {
    Self(std::sync::RwLock::new(value))
  }
}

impl<T: ?Sized> RwLock<T> {
  pub fn read(&self) -> RwLockReadGuard<'_, T> {
    unpoison(self.0.read())
  }

  pub fn write(&self) -> RwLockWriteGuard<'_, T> {
    unpoison(self.0.write())
  }

  pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
    try_unpoison(self.0.try_read())
  }

  pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
    try_unpoison(self.0.try_write())
  }
}
