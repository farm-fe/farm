use super::scan_exports::{scan_exports, Import};
use farmfe_core::config::config_regex::ConfigRegex;
use farmfe_toolkit::plugin_utils::{normalize_path::normalize_path, path_filter::PathFilter};
use glob::Pattern;
use walkdir::{DirEntry, WalkDir};

pub fn scan_dir_exports(dir: &str) -> Vec<Import> {
  let walker = WalkDir::new(dir).into_iter();
  let file_exts = vec!["js", "ts", "jsx", "tsx"];
  let filtered_entries = walker.filter_map(Result::ok).filter(|e| {
    e.file_type().is_file()
      && e.path().extension().is_some()
      && file_exts.contains(&e.path().extension().unwrap().to_str().unwrap())
  });

  let mut exports = Vec::new();
  for entry in filtered_entries {
    let file_path = entry.path();
    let exports_names = scan_exports(&normalize_path(file_path.to_str().unwrap()), None);
    exports.extend(exports_names);
  }
  exports
}

pub fn is_exclude_dir(entry: &DirEntry, exclude_patterns: &[Pattern]) -> bool {
  let path = entry.path();
  exclude_patterns.iter().any(|p| p.matches_path(path))
}

pub fn scan_dirs_exports(root_path: &str, dirs: &Vec<ConfigRegex>) -> Vec<Import> {
  let exclude_patterns = vec![Pattern::new("**/node_modules/**").expect("Invalid pattern")];
  let exclude = vec![];
  let filter = PathFilter::new(&dirs, &exclude);
  let walker = WalkDir::new(root_path).into_iter();
  let file_exts = vec!["js", "ts", "jsx", "tsx"];
  let filtered_entries = walker
    .filter_entry(|e| !is_exclude_dir(e, &exclude_patterns))
    .filter_map(Result::ok)
    .filter_map(|e| {
      if e.file_type().is_file() {
        let normalized_path = normalize_path(e.path().to_str().unwrap());
        if filter.execute(&normalized_path)
          && e.path().extension().is_some()
          && file_exts.contains(&e.path().extension().unwrap().to_str().unwrap())
        {
          return Some(normalized_path);
        } else {
          return None;
        }
      } else {
        None
      }
    });
  let mut exports = Vec::new();
  for file_path in filtered_entries {
    let exports_names = scan_exports(&file_path, None);
    exports.extend(exports_names);
  }
  exports
}
