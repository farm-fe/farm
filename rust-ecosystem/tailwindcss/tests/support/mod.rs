use std::path::{Path, PathBuf};

pub fn fixture_path(relative: &str) -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("fixtures")
    .join(relative)
}

pub fn manifest_path(path: &Path) -> String {
  path.strip_prefix(env!("CARGO_MANIFEST_DIR"))
    .unwrap_or(path)
    .to_string_lossy()
    .replace('\\', "/")
}

pub fn sorted_manifest_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<String> {
  let mut values = paths
    .into_iter()
    .map(|path| manifest_path(&path))
    .collect::<Vec<_>>();
  values.sort();
  values
}
