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

pub fn load_package_json(path: &str) {}

pub fn parse_query(path: &str) {}
