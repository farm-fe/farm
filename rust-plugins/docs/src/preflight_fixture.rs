use std::{
  fs, io,
  path::{Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};

/// Own only a directory reserved by an exclusive filesystem operation.
pub(crate) struct FixtureRoot(PathBuf);

impl FixtureRoot {
  pub(crate) fn new() -> io::Result<Self> {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_err(io::Error::other)?
      .as_nanos();
    Self::with_nonce(nonce)
  }

  fn with_nonce(nonce: u128) -> io::Result<Self> {
    for attempt in 0..1024 {
      let root = std::env::temp_dir().join(format!(
        "farm-docs-preflight-{}-{nonce}-{attempt}",
        std::process::id()
      ));
      match fs::create_dir(&root) {
        Ok(()) => return Ok(Self(root)),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
        Err(error) => return Err(error),
      }
    }
    Err(io::Error::new(
      io::ErrorKind::AlreadyExists,
      "could not reserve an independent preflight fixture",
    ))
  }

  pub(crate) fn path(&self) -> &Path {
    &self.0
  }
}

impl Drop for FixtureRoot {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{
    collections::HashSet,
    sync::{Arc, Barrier},
  };

  #[test]
  fn normal_fixture_is_removed_only_after_owner_drops() {
    let owner = FixtureRoot::new().unwrap();
    let root = owner.path().to_path_buf();
    assert!(root.is_dir());
    drop(owner);
    assert!(!root.exists());
  }

  #[test]
  fn repeated_clock_tick_does_not_share_or_remove_another_fixture() {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let first = FixtureRoot::with_nonce(nonce).unwrap();
    let second = FixtureRoot::with_nonce(nonce).unwrap();
    assert_ne!(first.path(), second.path());
    fs::write(first.path().join("index.md"), "# First\n").unwrap();
    fs::write(second.path().join("index.md"), "# Second\n").unwrap();
    drop(first);
    assert_eq!(
      fs::read_to_string(second.path().join("index.md")).unwrap(),
      "# Second\n"
    );
  }

  #[test]
  fn parallel_same_tick_fixtures_keep_independent_content_and_cleanup() {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let barrier = Arc::new(Barrier::new(8));
    let paths = std::thread::scope(|scope| {
      let workers: Vec<_> = (0..8)
        .map(|index| {
          let barrier = barrier.clone();
          scope.spawn(move || {
            let owner = FixtureRoot::with_nonce(nonce).unwrap();
            let content = format!("# Owner {index}\n");
            fs::write(owner.path().join("index.md"), &content).unwrap();
            barrier.wait();
            assert_eq!(
              fs::read_to_string(owner.path().join("index.md")).unwrap(),
              content
            );
            owner.path().to_path_buf()
          })
        })
        .collect();
      workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect::<Vec<_>>()
    });
    assert_eq!(paths.iter().collect::<HashSet<_>>().len(), 8);
    assert!(paths.iter().all(|root| !root.exists()));
  }
}
