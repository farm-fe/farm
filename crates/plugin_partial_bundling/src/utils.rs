use std::path::PathBuf;

use farmfe_core::HashSet;
use farmfe_core::module::ModuleId;
use farmfe_toolkit::hash::sha256;

/// Convert module identity into a portable output path without changing the ID
/// used by the module graph or runtime. The Vite adapter encodes NUL as `\\0`.
pub fn normalize_resource_path(path: &str) -> String {
  path
    .strip_prefix("\\0")
    .unwrap_or(path)
    .replace('\\', "/")
    .chars()
    .map(|c| {
      if c.is_control() || "<>:\"|?*'#".contains(c) {
        '_'
      } else {
        c
      }
    })
    .collect()
}

pub fn try_get_filename(path: PathBuf) -> String {
  path
    .file_stem()
    .map(|name| name.to_string_lossy().to_string())
    .unwrap_or(path.to_string_lossy().to_string())
}

pub fn get_sorted_module_ids_str(module_ids: &HashSet<ModuleId>) -> String {
  let mut sorted_module_ids = module_ids.iter().collect::<Vec<_>>();
  sorted_module_ids.sort_by_key(|a| a.to_string());
  sorted_module_ids
    .into_iter()
    .map(|id| id.to_string())
    .collect::<Vec<_>>()
    .join("_")
}

pub fn hash_module_ids(module_ids: &HashSet<ModuleId>, len: usize) -> String {
  let str = get_sorted_module_ids_str(module_ids);

  sha256(&str.into_bytes(), len)
}

#[cfg(test)]
mod tests {
  #[test]
  fn windows_directories_starting_with_zero_are_preserved() {
    assert_eq!(
      super::normalize_resource_path(r"src\0assets\file.ts"),
      "src/0assets/file.ts"
    );
    assert_eq!(
      super::normalize_resource_path(r"C:\0assets\file.ts"),
      "C_/0assets/file.ts"
    );
  }
}
