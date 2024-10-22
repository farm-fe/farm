use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use farmfe_core::module::ModuleId;
use farmfe_toolkit::hash::sha256;

pub fn try_get_filename(path: PathBuf) -> String {
  path
    .file_stem()
    .map(|name| name.to_string_lossy().to_string())
    .unwrap_or(path.to_string_lossy().to_string())
}

pub fn get_sorted_module_ids_str(module_ids: &HashSet<ModuleId>) -> String {
  let mut sorted_module_ids = module_ids.iter().collect::<Vec<_>>();
  sorted_module_ids.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
  sorted_module_ids
    .into_iter()
    .map(|id| id.to_string())
    .collect::<Vec<_>>()
    .join("_")
}

pub fn hash_module_ids(module_ids: &HashSet<ModuleId>) -> String {
  let str = get_sorted_module_ids_str(module_ids);

  sha256(&str.into_bytes(), 4)
}

pub fn group_is_enforce(group_name: &str, groups_enforce_map: &HashMap<String, bool>) -> bool {
  groups_enforce_map.get(group_name).cloned().unwrap_or(false)
}
