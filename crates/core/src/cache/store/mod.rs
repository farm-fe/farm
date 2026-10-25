//! Cache store of the persistent cache, responsible for reading and writing the cache from the disk.

pub mod constant;
mod disk;
mod error;
pub mod memory;
pub mod namespace;

pub use disk::*;
use farmfe_macro_cache_item::cache_item;

/// Cache key of the store, it's a pair of (name, cache_key), a name should only be related to one cache key.
/// Previous cache will be cleared if the related cache key changed for a name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cache_item]
// #[rkyv(derive(Serialize, Deserialize))]
#[rkyv(derive(Hash, Eq, PartialEq))]
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

#[cfg(test)]
pub(crate) mod controlled_write_backend_probe {
  use super::{
    CacheStoreKey,
    constant::{CacheStoreItemRef, CacheStoreTrait},
    error::CacheError,
  };
  use crate::{HashMap, cache::module_cache::CachedModule, deserialize};
  use std::sync::{Arc, Condvar, Mutex, mpsc};
  pub(crate) struct State {
    names: Mutex<Vec<String>>,
    paused: mpsc::Sender<(String, String)>,
    released: (Mutex<bool>, Condvar),
    pause_publish: bool,
    persisted: Mutex<HashMap<CacheStoreKey, Vec<u8>>>,
    history: Mutex<Vec<Vec<u8>>>,
  }
  impl State {
    pub(crate) fn release(&self) {
      *self.released.0.lock().expect("release") = true;
      self.released.1.notify_all();
    }
    fn pause(&self, pair: (String, String)) {
      self.paused.send(pair).expect("backend entered");
      let guard = self.released.0.lock().expect("release");
      drop(
        self
          .released
          .1
          .wait_while(guard, |released| !*released)
          .expect("backend barrier"),
      );
    }
    pub(crate) fn written_payload_count(&self) -> usize {
      let history = self.history.lock().expect("history");
      for bytes in history.iter() {
        let _: CachedModule = deserialize!(bytes, CachedModule);
      }
      history.len()
    }
    pub(crate) fn persisted_content(&self, name: &str) -> Option<String> {
      let bytes = self
        .persisted
        .lock()
        .expect("persisted")
        .iter()
        .find(|(key, _)| key.name == name)
        .map(|(_, value)| value.clone())?;
      let value: CachedModule = deserialize!(&bytes, CachedModule);
      Some(value.module.content.as_ref().clone())
    }
  }
  struct Backend(Arc<State>);
  type BackendFixture = (
    Box<dyn CacheStoreTrait>,
    Arc<State>,
    mpsc::Receiver<(String, String)>,
  );
  fn factory(pause_publish: bool) -> BackendFixture {
    let (paused, receiver) = mpsc::channel();
    let state = Arc::new(State {
      names: Mutex::new(vec![]),
      paused,
      released: (Mutex::new(false), Condvar::new()),
      pause_publish,
      persisted: Mutex::new(HashMap::default()),
      history: Mutex::new(vec![]),
    });
    (Box::new(Backend(state.clone())), state, receiver)
  }
  pub(crate) fn create() -> BackendFixture {
    factory(false)
  }
  pub(crate) fn create_publish() -> BackendFixture {
    factory(true)
  }
  impl CacheStoreTrait for Backend {
    fn has_cache(&self, name: &str) -> bool {
      self
        .0
        .persisted
        .lock()
        .expect("persisted")
        .keys()
        .any(|key| key.name == name)
    }
    fn is_cache_changed(&self, key: &CacheStoreKey) -> bool {
      let pair = {
        let mut names = self.0.names.lock().expect("names");
        names.push(key.name.clone());
        (!self.0.pause_publish && names.len() == 2).then(|| (names[0].clone(), names[1].clone()))
      };
      if let Some(pair) = pair {
        self.0.pause(pair);
      }
      true
    }
    fn write_single_cache(&self, key: CacheStoreKey, bytes: Vec<u8>) -> Result<(), CacheError> {
      self.write_cache(std::iter::once((key, bytes)).collect());
      Ok(())
    }
    fn write_cache(&self, data: HashMap<CacheStoreKey, Vec<u8>>) {
      if self.0.pause_publish {
        let key = data.keys().next().expect("publish payload");
        self.0.pause((key.name.clone(), key.name.clone()));
      }
      self
        .0
        .history
        .lock()
        .expect("history")
        .extend(data.values().cloned());
      self.0.persisted.lock().expect("persisted").extend(data);
    }
    fn read_cache(&self, name: &str) -> Option<Vec<u8>> {
      self
        .0
        .persisted
        .lock()
        .expect("persisted")
        .iter()
        .find(|(key, _)| key.name == name)
        .map(|(_, value)| value.clone())
    }
    fn read_cache_ref(&self, _: &str) -> Option<CacheStoreItemRef<'_>> {
      None
    }
    fn remove_cache(&self, name: &str) -> Option<Vec<u8>> {
      let mut data = self.0.persisted.lock().expect("persisted");
      let key = data.keys().find(|key| key.name == name).cloned()?;
      data.remove(&key)
    }
    fn remove_cache_only(&self, name: &str) {
      self
        .0
        .persisted
        .lock()
        .expect("persisted")
        .retain(|key, _| key.name != name);
    }
  }
}
