use std::path::{Path, PathBuf};

use farmfe_core::{dashmap::DashMap, relative_path::RelativePath};
use farmfe_utils::diff_paths;

/// Analyze symlinks and get the real path.
/// It will traverse all the ancestor of the specified path, if any ancestor is symlinks if will be followed to the real path.
/// If it is not symlinked, just return the original path.
pub struct SymlinksAnalyzer {
  cache: DashMap<PathBuf, PathBuf>,
}

impl Default for SymlinksAnalyzer {
  fn default() -> Self {
    Self::new()
  }
}

impl SymlinksAnalyzer {
  pub fn new() -> Self {
    Self {
      cache: DashMap::new(),
    }
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
          real_path = resolve_link_target(current.parent().unwrap(), current.read_link().unwrap());
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

// Absolute targets must not be rebased onto the link parent. Canonicalizing
// existing targets also resolves ancestor aliases such as macOS /var -> /private/var.
fn resolve_link_target(parent: &Path, target: PathBuf) -> PathBuf {
  let resolved = if target.is_absolute() {
    target
  } else {
    RelativePath::new(target.to_str().unwrap()).to_logical_path(parent)
  };
  std::fs::canonicalize(&resolved).unwrap_or(resolved)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn absolute_link_target_is_not_rebased() {
    let parent = std::env::temp_dir().join("farm-link-target-nonexistent-parent");
    let target = parent.join("physical");
    assert_eq!(resolve_link_target(&parent, target.clone()), target);
  }

  #[test]
  fn relative_link_target_uses_link_parent() {
    let parent = std::env::temp_dir().join("farm-link-target-nonexistent-parent");
    assert_eq!(
      resolve_link_target(&parent, PathBuf::from("../physical")),
      parent.parent().unwrap().join("physical")
    );
  }

  #[cfg(unix)]
  #[test]
  fn directory_alias_targets_preserve_child_identity() {
    use std::{
      fs,
      os::unix::fs::symlink,
      time::{SystemTime, UNIX_EPOCH},
    };
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let owner =
      std::env::temp_dir().join(format!("farm-link-target-{}-{nonce}", std::process::id()));
    fs::create_dir(&owner).unwrap();
    struct Owner(PathBuf);
    impl Drop for Owner {
      fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
      }
    }
    let _owner = Owner(owner.clone());
    let physical = owner.join("physical");
    fs::create_dir(&physical).unwrap();
    fs::write(physical.join("entry.js"), "export const value = 1;").unwrap();
    let absolute = owner.join("absolute");
    let relative = owner.join("relative");
    let chained = owner.join("chained");
    symlink(&physical, &absolute).unwrap();
    symlink("physical", &relative).unwrap();
    symlink(&absolute, &chained).unwrap();
    let expected = fs::canonicalize(physical.join("entry.js")).unwrap();
    let analyzer = SymlinksAnalyzer::new();
    for alias in [absolute, relative, chained] {
      assert_eq!(analyzer.follow_symlinks(alias.join("entry.js")), expected);
    }
  }
}
