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

use farmfe_core::common::PackageJsonInfo;
use lazy_static::lazy_static;

pub mod package_json_loader;
pub mod path_start_with_alias;
pub mod symlinks_analyzer;

use package_json_loader::PackageJsonLoader;

use crate::resolve::symlinks_analyzer::SymlinksAnalyzer;

use self::package_json_loader::Options;

lazy_static! {
  pub static ref PACKAGE_JSON_LOADER: PackageJsonLoader = PackageJsonLoader::new();
  pub static ref SYMLINKS_ANALYZER: SymlinksAnalyzer = SymlinksAnalyzer::new();
}

pub const DYNAMIC_EXTENSION_PRIORITY: &str = "DYNAMIC_EXTENSION_PRIORITY";

/// Load closest package.json start from the specified path, return [farmfe_core::error::Result<Value>].
pub fn load_package_json(
  path: PathBuf,
  options: Options,
) -> farmfe_core::error::Result<PackageJsonInfo> {
  // using global static package.json loader
  PACKAGE_JSON_LOADER.load(path, options)
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
  PackageJsonInfo::new(
    Some("farm-default-package-info".to_string()),
    Some("0.0.0".to_string()),
  )
}

/// Try follow symlinks from the specified path, if any ancestor of the path is symlinked, it will be redirected to the real path.
/// For example, the path is `/root/react/index.js` while `/root/react` is symlinked to `/root/store/react`, then the result should be `/root/store/react/index.js`.
pub fn follow_symlinks(path: PathBuf) -> PathBuf {
  SYMLINKS_ANALYZER.follow_symlinks(path)
}
