#[cfg(feature = "file_watcher")]
use std::path::{Path, PathBuf};

#[cfg(feature = "file_watcher")]
use napi::{
  bindgen_prelude::FunctionRef,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env, Status,
};

#[cfg(feature = "file_watcher")]
use notify::{
  event::{AccessKind, ModifyKind},
  EventKind, RecommendedWatcher, Watcher,
};

#[cfg(feature = "file_watcher")]
pub struct FsWatcher {
  watcher: notify::RecommendedWatcher,
  watched_paths: Vec<PathBuf>,
}
#[cfg(feature = "file_watcher")]
impl FsWatcher {
  pub fn new<F>(mut callback: F) -> notify::Result<Self>
  where
    F: FnMut(Vec<String>) + Send + Sync + 'static,
  {
    // TODO support other kind of events
    let watcher = RecommendedWatcher::new(
      move |result: std::result::Result<notify::Event, notify::Error>| {
        let event = result.unwrap();
        let get_paths = || {
          event
            .paths
            .iter()
            .map(|p| p.to_str().unwrap().to_string())
            .collect::<Vec<_>>()
        };
        // println!("{:?} {:?}", event.kind, event);
        if cfg!(target_os = "macos") {
          if matches!(event.kind, EventKind::Modify(ModifyKind::Data(_))) {
            callback(get_paths());
          }
        } else if cfg!(target_os = "linux") {
          // a close event is always followed by a modify event
          if matches!(event.kind, EventKind::Access(AccessKind::Close(_))) {
            callback(get_paths());
          }
        } else if event.kind.is_modify() {
          callback(get_paths());
        }
      },
      Default::default(),
    )?;

    Ok(Self {
      watcher,
      watched_paths: vec![],
    })
  }

  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub fn watch(&mut self, paths: Vec<&Path>) -> notify::Result<()> {
    if paths.is_empty() {
      return Ok(());
    }
    // find the longest common prefix
    let mut prefix_comps = vec![];
    let first_item = &paths[0];
    let rest = &paths[1..];

    for (index, comp) in first_item.components().enumerate() {
      if rest.iter().all(|item| {
        let mut item_comps = item.components();

        if index >= item.components().count() {
          return false;
        }

        item_comps.nth(index).unwrap() == comp
      }) {
        prefix_comps.push(comp);
      }
    }

    let watch_path = PathBuf::from_iter(prefix_comps.iter());

    if self
      .watched_paths
      .iter()
      .any(|item| watch_path.starts_with(item))
    {
      return Ok(());
    } else {
      self.watched_paths.push(watch_path.clone());
    }

    // println!("watch path {:?}", watch_path);

    self
      .watcher
      .watch(watch_path.as_path(), notify::RecursiveMode::Recursive)
  }

  #[cfg(target_os = "linux")]
  pub fn watch(&mut self, paths: Vec<&Path>) -> notify::Result<()> {
    for path in paths {
      if self.watched_paths.contains(&path.to_path_buf()) {
        continue;
      }

      self
        .watcher
        .watch(path, notify::RecursiveMode::NonRecursive)
        .ok();

      self.watched_paths.push(path.to_path_buf());
    }

    Ok(())
  }

  pub fn unwatch(&mut self, path: &str) -> notify::Result<()> {
    self.watcher.unwatch(Path::new(path))
  }
}

#[cfg(feature = "file_watcher")]
#[napi(js_name = "JsFileWatcher")]
pub struct FileWatcher {
  watcher: FsWatcher,
}

#[cfg(feature = "file_watcher")]
#[napi]
impl FileWatcher {
  #[napi(constructor)]
  pub fn new(env: Env, callback: FunctionRef<Vec<String>, ()>) -> napi::Result<Self> {
    let callback = callback.borrow_back(&env)?;
    let thread_safe_callback = callback
      .build_threadsafe_function::<Vec<String>>()
      .build()?;

    let watcher = FsWatcher::new(move |paths| {
      thread_safe_callback.call(paths, ThreadsafeFunctionCallMode::Blocking);
    })
    .map_err(|e| napi::Error::new(Status::GenericFailure, format!("{e}")))?;

    Ok(Self { watcher })
  }

  #[napi]
  pub fn watch(&mut self, paths: Vec<String>) -> napi::Result<()> {
    self
      .watcher
      .watch(paths.iter().map(Path::new).collect())
      .ok();

    Ok(())
  }

  #[napi]
  pub fn unwatch(&mut self, paths: Vec<String>) -> napi::Result<()> {
    for path in paths {
      self.watcher.unwatch(&path).ok();
    }

    Ok(())
  }
}
