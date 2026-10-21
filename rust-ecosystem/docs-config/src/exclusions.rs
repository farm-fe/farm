use crate::validate::ConfigError;
use std::{
  collections::HashSet,
  fs,
  path::{Path, PathBuf},
};

pub(crate) fn resolve(
  directory: &Path,
  values: &[String],
  field: &str,
) -> Result<HashSet<PathBuf>, ConfigError> {
  let mut result = HashSet::new();
  let mut names = HashSet::new();
  for (index, value) in values.iter().enumerate() {
    let field = format!("{field}.exclude[{index}]");
    let invalid =
      |message: &str| ConfigError::new("DOCS_CONFIG_COLLECTION_EXCLUDE", &field, message);
    if value.is_empty()
      || value.contains(['\\', ':', '*', '?', '[', ']', '\0'])
      || value.chars().any(char::is_control)
      || value.split('/').any(|part| {
        part.is_empty() || part == "." || part == ".." || crate::is_private_tooling_component(part)
      })
      || !matches!(
        Path::new(value).extension().and_then(|part| part.to_str()),
        Some("md" | "mdx")
      )
    {
      return Err(invalid(
        "require an exact relative Markdown filename without patterns or traversal",
      ));
    }
    let mut path = directory.to_path_buf();
    for part in value.split('/') {
      path.push(part);
      let metadata = fs::symlink_metadata(&path).map_err(|error| invalid(&error.to_string()))?;
      if metadata.file_type().is_symlink() {
        return Err(invalid("excluded inputs cannot contain symbolic links"));
      }
    }
    let path = path
      .canonicalize()
      .map_err(|error| invalid(&error.to_string()))?;
    if !path.starts_with(directory) || !path.is_file() {
      return Err(invalid(
        "excluded input must be a file inside its collection",
      ));
    }
    if !names.insert(value.to_lowercase()) || !result.insert(path) {
      return Err(invalid("duplicate excluded file"));
    }
  }
  Ok(result)
}
