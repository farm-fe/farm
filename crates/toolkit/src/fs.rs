use farmfe_core::error::{CompilationError, Result};

/// read content of the path, return utf8 string.
pub fn read_file_utf8(path: &str) -> Result<String> {
  let raw = std::fs::read(path).map_err(|e| CompilationError::GenericError(format!("{:?}", e)))?;
  String::from_utf8(raw).map_err(|e| {
    CompilationError::GenericError(format!(
      "File `{}` is not utf8! Detailed Error: {:?}",
      path, e
    ))
  })
}
