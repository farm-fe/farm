use std::{collections::HashMap, path::PathBuf};

use farmfe_core::{
  common::PackageJsonInfo,
  dashmap::DashMap,
  error::{CompilationError, Result},
  serde_json::{from_str, Value},
};

use crate::fs::read_file_utf8;

use super::follow_symlinks;

const PACKAGE_JSON_FILE: &str = "package.json";

/// Load closest package.json, return [farmfe_core::error::Result] if not found.
/// With cache supported, if the giving path is loaded then the cache will be used.
///
/// ```ignore
/// let package_json_loader = PackageJsonLoader::new();
/// let info = package_json.load("/root/packages/app/src")?;
/// ```
pub struct PackageJsonLoader {
  /// path -> package_json_info cache
  cache: DashMap<String, PackageJsonInfo>,
}

pub struct Options {
  // whether follow symlinks when resolving package.json
  pub follow_symlinks: bool,
  // whether resolve package.json in ancestor directories
  pub resolve_ancestor_dir: bool,
}

impl Default for Options {
  fn default() -> Self {
    Self {
      follow_symlinks: true,
      resolve_ancestor_dir: true,
    }
  }
}

impl PackageJsonLoader {
  pub fn new() -> Self {
    Self {
      cache: DashMap::new(),
    }
  }

  pub fn get_cache_key(&self, path: &PathBuf, options: &Options) -> String {
    format!(
      "{}{}{}",
      path.to_string_lossy(),
      options.follow_symlinks,
      options.resolve_ancestor_dir
    )
  }

  /// resolve package.json start from path to all its ancestor
  pub fn load(&self, path: PathBuf, options: Options) -> Result<PackageJsonInfo> {
    let mut current = path.clone();
    let mut visited_stack = vec![];

    while current.parent().is_some() {
      if self
        .cache
        .contains_key(&self.get_cache_key(&current, &options))
      {
        return Ok(
          self
            .cache
            .get(&self.get_cache_key(&current, &options))
            .unwrap()
            .clone(),
        );
      }

      // TODO cover it with tests
      if !options.resolve_ancestor_dir && visited_stack.len() == 1 {
        return Err(CompilationError::LoadPackageJsonError {
          package_json_path: current.to_string_lossy().to_string(),
          err_message: "can not load node_modules when resolve_ancestor_dir set to false"
            .to_string(),
        });
      }

      visited_stack.push(current.clone());

      let package_json_path = if options.follow_symlinks {
        follow_symlinks(current.join(PACKAGE_JSON_FILE))
      } else {
        current.join(PACKAGE_JSON_FILE)
      };

      if package_json_path.exists() && package_json_path.is_file() {
        let content = read_file_utf8(package_json_path.to_str().unwrap())?;

        let map: HashMap<String, Value> =
          from_str(&content).map_err(|e| CompilationError::LoadPackageJsonError {
            package_json_path: package_json_path.to_string_lossy().to_string(),
            err_message: format!("{e:?}"),
          })?;
        let map_string = |v: &Value| {
          if let Value::String(v) = v {
            v.to_string()
          } else {
            "".to_string()
          }
        };
        let name = map.get("name").map(map_string);
        let version = map.get("version").map(map_string);
        let mut result = PackageJsonInfo::new(name, version);
        result.set_raw_map(from_str(&content).map_err(|e| {
          CompilationError::LoadPackageJsonError {
            package_json_path: package_json_path.to_string_lossy().to_string(),
            err_message: format!("{e:?}"),
          }
        })?);
        result.set_raw(content);
        result.set_dir(
          package_json_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        );
        result.parse();

        for visited in visited_stack {
          self
            .cache
            .insert(self.get_cache_key(&visited, &options), result.clone());
        }

        return Ok(result);
      }

      current = current.parent().unwrap().to_path_buf();
    }

    Err(CompilationError::LoadPackageJsonError {
      package_json_path: path.to_string_lossy().to_string(),
      err_message: String::from("Can not find package.json in all ancestor directories"),
    })
  }

  pub fn cache(&self) -> &DashMap<String, PackageJsonInfo> {
    &self.cache
  }
}
