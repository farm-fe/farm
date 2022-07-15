use std::fmt::Debug;

pub struct CompilationError {}

impl Debug for CompilationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CompilationError").finish()
  }
}

pub type Result<T> = core::result::Result<T, CompilationError>;
