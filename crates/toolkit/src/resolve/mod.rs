//! Shared utilities for plugins.
//! ```ignore
//! use farmfe_core::load_package_json;
//!
//! struct MyPlugin {}
//!
//! impl Plugin for MyPlugin {
//!   fn resolve(...) -> ... {
//!     let pkg_json = load_package_json(dir);
//!   }
//! }
//! ```

use std::path::PathBuf;

use farmfe_core::{
  common::PackageJsonInfo,
  serde_json::{from_str, Value},
};
use lazy_static::lazy_static;

pub mod package_json_loader;

use package_json_loader::PackageJsonLoader;

lazy_static! {
  pub static ref PACKAGE_JSON_LOADER: PackageJsonLoader = PackageJsonLoader::new();
}

/// Load closest package.json start from the specified path, return [farmfe_core::error::Result<Value>].
pub fn load_package_json(path: PathBuf) -> farmfe_core::error::Result<PackageJsonInfo> {
  // using global static package.json loader
  PACKAGE_JSON_LOADER.load(path)
}

/// The default package.json info is:
/// ```json
/// {
///   "name": "farm-default-package-info",
///   "version": "0.0.0"
/// }
/// ```
/// And this function won't trigger any file/io operation
pub fn default_package_json() -> PackageJsonInfo {
  from_str(
    r#"{
    "name": "farm-default-package-info",
    "version": "0.0.0"
  }"#,
  )
  .unwrap()
}

/// see [querystring::querify]
pub fn parse_query(path: &str) -> Vec<(&str, &str)> {
  let query_str = path.split('?').last().unwrap();
  querystring::querify(query_str)
}
