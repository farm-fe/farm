use crate::{PreparedAsset, StaticDirectoryInput, hash, prepare::farm_path, routes};
use farmfe_docs_config::{PublicSiteConfig, ValidatedConfig};
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
};
const URL_SEGMENT: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
  .remove(b'-')
  .remove(b'.')
  .remove(b'_')
  .remove(b'~');
fn walk(
  root: &Path,
  dir: &Path,
  entries: &mut Vec<String>,
  files: &mut Vec<PathBuf>,
) -> Result<(), String> {
  let mut children = fs::read_dir(dir)
    .map_err(|e| format!("[DOCS_STATIC_READ] {}: {e}", dir.display()))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("[DOCS_STATIC_READ] {}: {e}", dir.display()))?;
  children.sort_by_key(|entry| entry.file_name());
  for child in children {
    let file = child.path();
    let metadata = fs::symlink_metadata(&file)
      .map_err(|e| format!("[DOCS_STATIC_READ] {}: {e}", file.display()))?;
    let reparse = metadata.file_type().is_symlink();
    #[cfg(windows)]
    let reparse = {
      use std::os::windows::fs::MetadataExt;
      reparse || metadata.file_attributes() & 0x400 != 0
    };
    if reparse {
      return Err(format!(
        "[DOCS_STATIC_PATH] {}: symlinks/junctions are not supported",
        file.display()
      ));
    }
    let relative = farm_path(file.strip_prefix(root).map_err(|e| e.to_string())?)?;
    routes::validate_output(&relative, &farm_path(&file)?)?;
    if farmfe_docs_config::is_private_tooling_component(&child.file_name().to_string_lossy()) {
      return Err(format!(
        "[DOCS_STATIC_PATH] {}: private tooling input",
        file.display()
      ));
    }
    if metadata.is_dir() {
      entries.push(format!("{relative}/"));
      walk(root, &file, entries, files)?;
    } else if metadata.is_file() {
      entries.push(relative);
      files.push(file);
    } else {
      return Err(format!(
        "[DOCS_STATIC_PATH] {}: require ordinary files",
        file.display()
      ));
    }
  }
  Ok(())
}
pub fn listing(path: &str) -> Result<(String, Vec<PathBuf>), String> {
  let root = Path::new(path);
  let metadata =
    fs::symlink_metadata(root).map_err(|cause| format!("[DOCS_STATIC_READ] {path}: {cause}"))?;
  let reparse = metadata.file_type().is_symlink();
  #[cfg(windows)]
  let reparse = {
    use std::os::windows::fs::MetadataExt;
    reparse || metadata.file_attributes() & 0x400 != 0
  };
  if reparse || !metadata.is_dir() {
    return Err(format!(
      "[DOCS_STATIC_PATH] {path}: require an ordinary directory"
    ));
  }
  let mut entries = vec![];
  let mut files = vec![];
  walk(root, root, &mut entries, &mut files)?;
  entries.sort();
  Ok((
    hash(&serde_json::to_vec(&entries).map_err(|e| e.to_string())?),
    files,
  ))
}
pub fn scan(
  config: &ValidatedConfig,
) -> Result<(Vec<PreparedAsset>, Vec<StaticDirectoryInput>), String> {
  let mut files = vec![];
  let mut inputs = vec![];
  for source in &config.static_dirs {
    let directory = farm_path(&source.directory)?;
    let (entries_hash, paths) = listing(&directory)?;
    inputs.push(StaticDirectoryInput {
      path: directory.clone(),
      entries_hash,
    });
    for path in paths {
      let source_path = farm_path(&path)?;
      let relative = farm_path(path.strip_prefix(&directory).map_err(|e| e.to_string())?)?;
      let output_path = format!("{}{}", source.route_base.trim_start_matches('/'), relative);
      routes::validate_output(&output_path, &source_path)?;
      let source_hash =
        hash(&fs::read(&path).map_err(|e| format!("[DOCS_STATIC_READ] {source_path}: {e}"))?);
      let normalized_root = PathBuf::from(farm_path(&config.site_root)?);
      let source_relative = farm_path(
        Path::new(&source_path)
          .strip_prefix(&normalized_root)
          .map_err(|e| e.to_string())?,
      )?;
      let key = hash(
        &serde_json::to_vec(&(source_relative, &source_hash, &output_path))
          .map_err(|e| e.to_string())?,
      );
      files.push(PreparedAsset {
        key,
        source_path,
        source_hash,
        output_path,
        is_static: true,
      });
    }
  }
  files.sort_by(|a, b| a.output_path.cmp(&b.output_path));
  let mut names: Vec<_> = files.iter().map(|file| file.output_path.clone()).collect();
  names.extend(reserved(&config.public));
  validate_names(&names)?;
  Ok((files, inputs))
}
pub fn reserved(site: &PublicSiteConfig) -> Vec<String> {
  let mut names: Vec<String> = [
    "docs-plan.json",
    "docs-public-config.json",
    "docs-manifest.json",
    "docs-build-report.json",
  ]
  .into_iter()
  .map(str::to_string)
  .collect();
  names.extend(
    crate::not_found::outputs(site)
      .into_iter()
      .map(|(_, output, _)| output),
  );
  names.extend(crate::syndication::output_paths(site));
  if site.ai.as_ref().is_some_and(|ai| ai.enabled) {
    names.extend(["docs-ai-manifest.json".to_string(), "llms.txt".to_string()]);
  }
  names
}
pub fn validate_names(names: &[String]) -> Result<(), String> {
  let mut sorted: BTreeMap<String, String> = BTreeMap::new();
  for name in names {
    let normalized = routes::collision_key(name);
    if let Some(previous) = sorted.insert(normalized, name.clone()) {
      return Err(format!("[DOCS_STATIC_COLLISION] {previous} and {name}"));
    }
  }
  for (key, name) in &sorted {
    // Lexicographic neighbours are insufficient: `a-other` sorts between
    // conflicting `a` and `a/file`. Test each complete ancestor instead.
    for (separator, _) in key.match_indices('/') {
      if let Some(old) = sorted.get(&key[..separator]) {
        return Err(format!(
          "[DOCS_STATIC_COLLISION] {old} and {name}: file/directory conflict"
        ));
      }
    }
  }
  Ok(())
}
pub fn public_url(config: &farmfe_docs_config::PublicSiteConfig, path: &str) -> String {
  let encoded = path
    .split('/')
    .map(|part| percent_encoding::utf8_percent_encode(part, URL_SEGMENT).to_string())
    .collect::<Vec<_>>()
    .join("/");
  format!("{}{encoded}", config.base_path)
}
