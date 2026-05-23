#![deny(clippy::all)]

use farmfe_core::{serde, serde_json};
use farmfe_toolkit::resolve::package_json_loader::{
  Options as PackageJsonLoaderOptions, PackageJsonLoader,
};

use std::path::Path;

use serde_json::Value;
pub fn default_scale() -> Option<f32> {
  Some(1.2)
}

pub fn default_auto_install() -> Option<bool> {
  Some(false)
}

pub fn default_compiler() -> Option<String> {
  Some(String::from("jsx"))
}

#[derive(Debug, serde::Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Options {
  #[serde(default = "default_scale")]
  pub scale: Option<f32>,
  pub default_style: Option<Value>,
  pub default_class: Option<String>,
  #[serde(default = "default_compiler")]
  pub compiler: Option<String>,
  pub jsx: Option<String>,
  // https://example.com/icons/[iconname].svg || { "collections": { "custom": { "dir": "path/to/icons", "recursion": false } } }
  pub custom_collections: Option<Value>,
  #[serde(default = "default_auto_install")]
  pub auto_install: Option<bool>,
  pub collections_node_resolve_path: Option<String>,
}

pub fn guess_jsx(root_path: &str) -> String {
  let loader = PackageJsonLoader::new();
  let package_path = Path::new(root_path);
  let package_json = loader
    .load(
      package_path.to_path_buf(),
      PackageJsonLoaderOptions {
        follow_symlinks: false,
        resolve_ancestor_dir: false,
      },
    )
    .unwrap();
  let preact = package_json
    .raw_map()
    .get("preact")
    .and_then(|v| v.as_str());
  preact.map_or("react".to_string(), |_| "preact".to_string())
}
