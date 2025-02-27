use std::path::PathBuf;

use farmfe_core::{dashmap::DashMap, relative_path::RelativePath};
use farmfe_utils::diff_paths;

/// Analyze symlinks and get the real path.
///
/// It will traverse all the ancestor of the specified path, if
/// any ancestor is symlinks if will be followed to the real path.
/// If it is not symlinked, just return the original path.
#[derive(Default)]
pub struct SymlinksAnalyzer {
  cache: DashMap<PathBuf, PathBuf>,
}

impl SymlinksAnalyzer {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn follow_symlinks(&self, path: PathBuf) -> PathBuf {
    if self.cache.contains_key(&path) {
      return self.cache.get(&path).unwrap().clone();
    }

    let mut current = path.clone();
    let mut real_path = path.clone();

    while current.parent().is_some() {
      if current.is_symlink() {
        if cfg!(windows) {
          real_path = current.read_link().unwrap();
        } else {
          real_path = RelativePath::new(current.read_link().unwrap().to_str().unwrap())
            .to_logical_path(current.parent().unwrap());
        }

        break;
      }

      current = current.parent().unwrap().to_path_buf();
    }

    // there is symlink existed
    let real_path = if real_path != path {
      let relative_path = diff_paths(path.clone(), current).unwrap();
      real_path.join(relative_path)
    } else {
      path.clone()
    };

    self.cache.insert(path, real_path.clone());

    real_path
  }
}
