//! Private source and output receipts for native Farm Docs builds.
mod source;
mod verify;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub use source::{SourceRecord, observe, verify_source};
use std::{collections::BTreeSet, fs, path::Path};
pub use verify::{Verification, verify_build};

pub fn hash(bytes: &[u8]) -> String {
  format!("{:x}", Sha256::digest(bytes))
}

pub fn error(message: impl std::fmt::Display) -> String {
  format!("[DOCS_INPUT_SEAL] {message}")
}

pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
  let bytes = fs::read(path).map_err(|e| error(format!("{}: {e}", path.display())))?;
  serde_json::from_slice(&bytes).map_err(|e| error(format!("{}: {e}", path.display())))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
  Browser,
  Server,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceRecord {
  pub name: String,
  pub resource_type: String,
  pub length: usize,
  pub hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Receipt {
  pub schema_version: u32,
  pub plan_snapshot_id: String,
  pub target: Target,
  pub root: String,
  pub sources: Vec<SourceRecord>,
  pub resources: Vec<ResourceRecord>,
}

impl Receipt {
  pub fn validate(&self, snapshot: &str, target: Target, root: &str) -> Result<(), String> {
    if self.schema_version != 1
      || self.plan_snapshot_id != snapshot
      || self.target != target
      || self.root != root
    {
      return Err(error("receipt schema, snapshot, target or root mismatch"));
    }
    if self.sources.is_empty() || self.resources.is_empty() {
      return Err(error("receipt is missing sources or resources"));
    }
    let mut paths = BTreeSet::new();
    for source in &self.sources {
      if !paths.insert(&source.path) {
        return Err(error(format!("duplicate source {}", source.path)));
      }
      verify_source(source)?;
    }
    let names = self
      .resources
      .iter()
      .map(|r| r.name.clone())
      .collect::<Vec<_>>();
    farmfe_docs_plan::validate_resource_names(&names).map_err(error)?;
    let mut names = BTreeSet::new();
    for resource in &self.resources {
      if !valid_resource_name(&resource.name)
        || !names.insert(&resource.name)
        || !valid_hash(&resource.hash)
      {
        return Err(error(format!(
          "duplicate or invalid resource {}",
          resource.name
        )));
      }
    }
    Ok(())
  }
}

pub fn valid_hash(value: &str) -> bool {
  value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn valid_resource_name(name: &str) -> bool {
  !name.is_empty()
    && !name.contains(['\\', ':'])
    && !Path::new(name).is_absolute()
    && !name
      .split('/')
      .any(|part| part.is_empty() || part == "." || part == "..")
}

pub fn verify_resource(record: &ResourceRecord, path: &Path) -> Result<(), String> {
  let metadata =
    fs::symlink_metadata(path).map_err(|e| error(format!("{}: {e}", path.display())))?;
  if !metadata.is_file() || metadata.file_type().is_symlink() {
    return Err(error(format!(
      "resource is not a regular file: {}",
      path.display()
    )));
  }
  let bytes = fs::read(path).map_err(|e| error(format!("{}: {e}", path.display())))?;
  if bytes.len() != record.length || hash(&bytes) != record.hash {
    return Err(error(format!("resource changed: {}", path.display())));
  }
  Ok(())
}
