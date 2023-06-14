use farmfe_core::error::{CompilationError, Result};

use crate::hash::sha256;

/// read content of the path, return utf8 string.
pub fn read_file_utf8(path: &str) -> Result<String> {
  let raw = read_file_raw(path)?;
  String::from_utf8(raw).map_err(|e| {
    CompilationError::GenericError(format!(
      "File `{}` is not utf8! Detailed Error: {:?}",
      path, e
    ))
  })
}

/// read content of the path, return bytes.
pub fn read_file_raw(path: &str) -> Result<Vec<u8>> {
  std::fs::read(path).map_err(|e| CompilationError::GenericError(format!("{:?}", e)))
}

pub fn transform_output_filename(
  filename_config: String,
  name: &str,
  bytes: &[u8],
  ext: &str,
) -> String {
  let mut res = filename_config;

  if res.contains("[resourceName]") {
    res = res.replace("[resourceName]", name);
  }

  if res.contains("[contentHash]") {
    let content_hash = sha256(bytes, 8);
    res = res.replace("[contentHash]", &content_hash);
  }

  if res.contains("[ext]") {
    res = res.replace("[ext]", ext);
  }

  res
}

pub fn transform_output_entry_filename(
  entry_filename_config: String,
  name: &str,
  entry_filename: &str,
  bytes: &[u8],
  ext: &str,
) -> String {
  let mut res = entry_filename_config;

  if res.contains("[entryName]") {
    res = res.replace("[entryName]", entry_filename);
  }

  transform_output_filename(res, name, bytes, ext)
}
