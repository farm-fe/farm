//! OS-owned workflow output locks. The stable lock file is never removed here:
//! unlinking it would let concurrent callers lock different file identities.
use std::{
  fs::{self, File, OpenOptions, TryLockError},
  path::Path,
};

use napi::{Error, Result};

#[napi]
pub struct OutputDirectoryLock {
  file: Option<File>,
}

#[napi]
impl OutputDirectoryLock {
  #[napi(constructor)]
  pub fn new(path: String) -> Result<Self> {
    let path = Path::new(&path);
    if !path.is_absolute() {
      return Err(Error::from_reason(
        "[DOCS_OUTPUT_LOCK] expected an absolute lock path",
      ));
    }
    match fs::symlink_metadata(path) {
      Ok(metadata) if !metadata.file_type().is_file() => {
        return Err(Error::from_reason(
          "[DOCS_OUTPUT_LOCK] expected a regular lock file",
        ));
      }
      Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
        return Err(Error::from_reason(format!("[DOCS_OUTPUT_LOCK] {error}")));
      }
      _ => {}
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(windows)]
    {
      use std::os::windows::fs::OpenOptionsExt;
      // FILE_SHARE_READ | FILE_SHARE_WRITE: permit competing lock attempts,
      // but prevent deleting/replacing the lock identity while a handle lives.
      options.share_mode(0x0000_0001 | 0x0000_0002);
    }
    let file = options
      .open(path)
      .map_err(|error| Error::from_reason(format!("[DOCS_OUTPUT_LOCK] {error}")))?;
    match file.try_lock() {
      Ok(()) => Ok(Self { file: Some(file) }),
      Err(TryLockError::WouldBlock) => Err(Error::from_reason(
        "[DOCS_OUTPUT_LOCKED] another build owns this output",
      )),
      Err(TryLockError::Error(error)) => {
        Err(Error::from_reason(format!("[DOCS_OUTPUT_LOCK] {error}")))
      }
    }
  }

  #[napi]
  pub fn release(&mut self) -> Result<()> {
    if let Some(file) = self.file.take() {
      file
        .unlock()
        .map_err(|error| Error::from_reason(format!("[DOCS_OUTPUT_LOCK] release: {error}")))?;
    }
    Ok(())
  }
}
