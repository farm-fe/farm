use std::path::PathBuf;

pub fn try_get_filename(path: PathBuf) -> String {
  path
    .file_stem()
    .map(|name| name.to_string_lossy().to_string())
    .unwrap_or(path.to_string_lossy().to_string())
}
