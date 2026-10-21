use crate::{error, hash, valid_hash};
use serde::{Deserialize, Serialize};
use std::{
  fs,
  path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceRecord {
  pub path: String,
  pub canonical_path: String,
  pub directory: bool,
  pub hash: String,
  pub exclusions: Vec<String>,
}

fn path_text(path: &Path) -> Result<String, String> {
  path
    .to_str()
    .map(String::from)
    .ok_or_else(|| error("source path must be UTF-8"))
}

fn public_path(path: &Path) -> Result<(), String> {
  if !path.is_absolute()
    || path.components().any(|part| {
      farmfe_docs_config::is_private_tooling_component(&part.as_os_str().to_string_lossy())
    })
  {
    return Err(error(format!(
      "invalid or private source {}",
      path.display()
    )));
  }
  Ok(())
}

fn within(path: &Path, root: &Path) -> bool {
  if cfg!(windows) {
    let path = path.to_string_lossy().replace('\\', "/").to_lowercase();
    let root = root.to_string_lossy().replace('\\', "/").to_lowercase();
    path == root || path.starts_with(&format!("{}/", root.trim_end_matches('/')))
  } else {
    path.starts_with(root)
  }
}

// Generated directories may not exist on the first observation. Resolve their
// existing parent identities without requiring the generated tree to exist.
fn exclusion_identity(path: &Path) -> Result<PathBuf, String> {
  let mut ancestor = path;
  let mut suffix = vec![];
  loop {
    match fs::canonicalize(ancestor) {
      Ok(mut resolved) => {
        for component in suffix.into_iter().rev() {
          resolved.push(component);
        }
        return Ok(resolved);
      }
      Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => {
        let component = ancestor.file_name().ok_or_else(|| error(&failure))?;
        suffix.push(component.to_os_string());
        ancestor = ancestor.parent().ok_or_else(|| error(&failure))?;
      }
      Err(failure) => return Err(error(failure)),
    }
  }
}

fn listing(
  root: &Path,
  path: &Path,
  exclusions: &[PathBuf],
  rows: &mut Vec<(String, String)>,
) -> Result<(), String> {
  for entry in fs::read_dir(path).map_err(|e| error(format!("{}: {e}", path.display())))? {
    let entry = entry.map_err(error)?;
    let path = entry.path();
    if exclusions.iter().any(|excluded| within(&path, excluded))
      || farmfe_docs_config::is_private_tooling_component(&entry.file_name().to_string_lossy())
    {
      continue;
    }
    let metadata = fs::symlink_metadata(&path).map_err(error)?;
    if metadata.file_type().is_symlink() {
      return Err(error(format!(
        "watch directory contains symlink: {}",
        path.display()
      )));
    }
    if metadata.is_dir() {
      let relative = path.strip_prefix(root).map_err(error)?;
      rows.push((path_text(relative)?.replace('\\', "/"), "directory".into()));
      listing(root, &path, exclusions, rows)?;
    } else if metadata.is_file() {
      let relative = path.strip_prefix(root).map_err(error)?;
      rows.push((
        path_text(relative)?.replace('\\', "/"),
        hash(&fs::read(&path).map_err(error)?),
      ));
    } else {
      return Err(error(format!("unsupported watch input {}", path.display())));
    }
  }
  Ok(())
}

pub fn observe(path: &Path, exclusions: &[PathBuf]) -> Result<SourceRecord, String> {
  public_path(path)?;
  let metadata =
    fs::symlink_metadata(path).map_err(|e| error(format!("{}: {e}", path.display())))?;
  if metadata.file_type().is_symlink() || !(metadata.is_file() || metadata.is_dir()) {
    return Err(error(format!(
      "source is not a regular file or directory: {}",
      path.display()
    )));
  }
  let canonical = fs::canonicalize(path).map_err(error)?;
  public_path(&canonical)?;
  let directory = metadata.is_dir();
  // A pruning rule applies to descendants of the observed root. An explicit
  // watch below that rule must still observe its own members.
  let (exclusions, identities) = if directory {
    let mut retained = vec![];
    let mut identities = vec![];
    for excluded in exclusions {
      let identity = exclusion_identity(excluded)?;
      if !within(&canonical, &identity) {
        retained.push(excluded.clone());
        identities.push(identity);
      }
    }
    (retained, identities)
  } else {
    (exclusions.to_vec(), vec![])
  };
  let digest = if directory {
    let mut rows = vec![];
    listing(&canonical, &canonical, &identities, &mut rows)?;
    rows.sort();
    hash(&serde_json::to_vec(&rows).map_err(error)?)
  } else {
    hash(&fs::read(path).map_err(error)?)
  };
  Ok(SourceRecord {
    path: path_text(path)?,
    canonical_path: path_text(&canonical)?,
    directory,
    hash: digest,
    exclusions: exclusions
      .iter()
      .map(|path| path_text(path))
      .collect::<Result<_, _>>()?,
  })
}

pub fn verify_source(source: &SourceRecord) -> Result<(), String> {
  if !valid_hash(&source.hash)
    || source
      .exclusions
      .iter()
      .any(|path| !Path::new(path).is_absolute())
  {
    return Err(error(format!("invalid source receipt {}", source.path)));
  }
  let current = observe(
    Path::new(&source.path),
    &source
      .exclusions
      .iter()
      .map(PathBuf::from)
      .collect::<Vec<_>>(),
  )?;
  if current != *source {
    return Err(error(format!("source snapshot changed: {}", source.path)));
  }
  Ok(())
}
