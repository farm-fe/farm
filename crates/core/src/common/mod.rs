use std::path::PathBuf;

use farmfe_macro_cache_item::cache_item;
use rkyv::{Archive, Deserialize, Serialize};

/// package json info that farm used.
/// **Note**: if you want to use the field that not defined here, you can deserialize raw and get the raw package.json [serde_json::Value]
#[cache_item]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageJsonInfo {
  pub name: String,
  pub version: String,
  // pub browser: Option<String>,
  // pub exports: Option<String>,
  // pub side_effects: Option<bool>,
  raw: Option<String>,
  /// the directory this package.json belongs to
  dir: Option<String>,
}

impl PackageJsonInfo {
  pub fn set_raw(&mut self, raw: String) {
    self.raw = Some(raw);
  }

  pub fn raw(&self) -> &String {
    self.raw.as_ref().unwrap()
  }

  pub fn set_dir(&mut self, dir: String) {
    self.dir = Some(dir);
  }

  pub fn dir(&self) -> &String {
    self.dir.as_ref().unwrap()
  }
}
