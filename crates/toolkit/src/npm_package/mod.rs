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

use farmfe_core::serde_json::{from_str, Value};

/// Load closest package.json start from the specified path, return [Result<Value, std::io::Error>].
/// Return [std::io::Error] if no package.json found
pub fn load_package_json(path: &str) -> Result<Value, std::io::Error> {
  Ok(Value::Null)
}

/// Load closest package.json start from the specified path, return default package.json info if no package.json found.
/// The default package.json info is:
/// ```json
/// {
///   "name": "farm-default-package-info",
///   "version": "0.0.0"
/// }
/// ```
pub fn load_package_json_or_default(path: &str) -> Value {
  load_package_json(path).unwrap_or(
    from_str(
      r#"{
    "name": "farm-default-package-info",
    "version": "0.0.0"
  }"#,
    )
    .unwrap(),
  )
}

pub fn parse_query(path: &str) {}
