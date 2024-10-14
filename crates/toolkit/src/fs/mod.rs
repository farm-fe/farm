use farmfe_core::error::{CompilationError, Result};

use crate::hash::sha256;

pub const RESOURCE_NAME: &str = "[resourceName]";
pub const RESOURCE_NAME_NEW: &str = "[name]";
pub const CONTENT_HASH: &str = "[contentHash]";
pub const CONTENT_HASH_NEW: &str = "[hash]";
pub const EXT: &str = "[ext]";
pub const ENTRY_NAME: &str = "[entryName]";

/// read content of the path, return utf8 string.
pub fn read_file_utf8(path: &str) -> Result<String> {
  let raw = read_file_raw(path)?;
  Ok(String::from_utf8_lossy(&raw).into_owned())
}

/// read content of the path, return bytes.
pub fn read_file_raw(path: &str) -> Result<Vec<u8>> {
  std::fs::read(path).map_err(|e| CompilationError::GenericError(format!("{e:?}")))
}

pub fn transform_output_filename(
  filename_config: String,
  name: &str,
  bytes: &[u8],
  ext: &str,
) -> String {
  let mut res = filename_config;

  if res.contains(RESOURCE_NAME) {
    res = res.replace(RESOURCE_NAME, name);
  } else if res.contains(RESOURCE_NAME_NEW) {
    res = res.replace(RESOURCE_NAME_NEW, name);
  }

  if res.contains(CONTENT_HASH) {
    let content_hash = sha256(bytes, 8);
    res = res.replace(CONTENT_HASH, &content_hash);
  } else if res.contains(CONTENT_HASH_NEW) {
    let content_hash = sha256(bytes, 8);
    res = res.replace(CONTENT_HASH_NEW, &content_hash);
  }

  if res.contains(EXT) {
    res = res.replace(EXT, ext);
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

  if res.contains(ENTRY_NAME) {
    res = res.replace(ENTRY_NAME, entry_filename);
  }

  transform_output_filename(res, name, bytes, ext)
}
