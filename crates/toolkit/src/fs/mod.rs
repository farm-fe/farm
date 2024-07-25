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

fn is_valid_char(ch: char) -> bool {
  ch.is_ascii_digit() || is_valid_first_char(ch)
}

fn is_valid_first_char(ch: char) -> bool {
  ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == '_'
}

/// normalize file name as variable name.
pub fn normalize_file_name_as_variable(str: String) -> String {
  let mut res = String::with_capacity(str.len());

  let mut first = true;

  let mut prev_is_invalid = false;
  for ch in str.chars() {
    if first {
      if !is_valid_first_char(ch) {
        res.push('_');

        if is_valid_char(ch) {
          res.push(ch);
        } else {
          prev_is_invalid = true;
        }
      } else {
        res.push(ch)
      }
      first = false;
    } else if is_valid_char(ch) || ch.is_ascii_digit() {
      res.push(ch);
      prev_is_invalid = false;
    } else {
      if prev_is_invalid {
        continue;
      }

      res.push('_');
      prev_is_invalid = true;
    }
  }

  res
}

#[cfg(test)]
mod tests {
    use crate::fs::normalize_file_name_as_variable;

  #[test]
  fn test_normalize_name() {
    let normalized_str = normalize_file_name_as_variable(String::from("F:\\path\\to\\file.ts"));
    assert_eq!(normalized_str, "F_path_to_file_ts");

    let normalized_str = normalize_file_name_as_variable(String::from("/path/to/file.ts"));
    assert_eq!(normalized_str, "_path_to_file_ts");

    let normalized_str = normalize_file_name_as_variable(String::from("$_#$()axq"));
    assert_eq!(normalized_str, "___axq");

    let normalized_str = normalize_file_name_as_variable(String::from("_a_b_C_D"));
    assert_eq!(normalized_str, "_a_b_C_D");

    let normalized_str = normalize_file_name_as_variable(String::from("123456789"));
    assert_eq!(normalized_str, "_123456789");

    let normalized_str = normalize_file_name_as_variable(String::from("1_2_3_4"));
    assert_eq!(normalized_str, "_1_2_3_4");

    let normalized_str = normalize_file_name_as_variable(String::from("1text.ts"));
    assert_eq!(normalized_str, "_1text_ts");
  }
}
