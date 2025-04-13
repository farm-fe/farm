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

pub struct TransformOutputFileNameParams<'a> {
  pub filename_config: String,
  pub name: &'a str,
  pub name_hash: &'a str,
  pub bytes: &'a [u8],
  pub ext: &'a str,
}

pub fn transform_output_filename(
  TransformOutputFileNameParams {
    filename_config,
    name,
    name_hash,
    bytes,
    ext,
  }: TransformOutputFileNameParams,
) -> String {
  let mut res = filename_config;
  let mut name = name.to_string();

  if res.contains(CONTENT_HASH) {
    let content_hash = sha256(bytes, 8);
    res = res.replace(CONTENT_HASH, &content_hash);
  } else if res.contains(CONTENT_HASH_NEW) {
    let content_hash = sha256(bytes, 8);
    res = res.replace(CONTENT_HASH_NEW, &content_hash);
  } else if !name_hash.is_empty() {
    name = format!(
      "{}-{}",
      name,
      if name_hash.len() > 8 {
        &name_hash[0..8]
      } else {
        name_hash
      }
    );
  }

  if res.contains(RESOURCE_NAME) {
    res = res.replace(RESOURCE_NAME, &name);
  } else if res.contains(RESOURCE_NAME_NEW) {
    res = res.replace(RESOURCE_NAME_NEW, &name);
  }

  if res.contains(EXT) {
    res = res.replace(EXT, ext);
  }

  res
}

pub fn transform_output_entry_filename(
  entry_name: &str,
  mut params: TransformOutputFileNameParams,
) -> String {
  if params.filename_config.contains(ENTRY_NAME) {
    params.filename_config = params.filename_config.replace(ENTRY_NAME, entry_name);
  }

  transform_output_filename(params)
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
