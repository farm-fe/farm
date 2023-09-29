use std::{
  collections::HashMap,
  fmt,
  path::{Path, PathBuf},
  str::FromStr,
  sync::Arc,
};

use farmfe_core::{
  common::PackageJsonInfo,
  context::CompilationContext,
  farm_profile_function,
  relative_path::RelativePath,
  serde_json::{from_str, Map, Value, self},
};

pub const SLASH_CODE: char = '/';

pub const DOT_CODE: char = '.';

pub const HASH_CODE: char = '#';

pub const NODE_MODULES: &str = "node_modules";

pub fn is_source_relative(source: &str) -> bool {
  // fix: relative path start with .. or ../
  source.starts_with("./") || source.starts_with("../")
}

pub fn is_source_absolute(source: &str) -> bool {
  if let Ok(sp) = PathBuf::from_str(source) {
    sp.is_absolute()
  } else {
    false
  }
}

pub fn is_source_dot(source: &str) -> bool {
  source == "."
}

pub fn is_double_source_dot(source: &str) -> bool {
  source == ".."
}

pub fn is_module_side_effects(package_json_info: &PackageJsonInfo, resolved_path: &str) -> bool {
  farm_profile_function!("is_module_side_effects".to_string());
  match package_json_info.side_effects() {
    farmfe_core::common::ParsedSideEffects::Bool(b) => *b,
    farmfe_core::common::ParsedSideEffects::Array(arr) => arr.iter().any(|s| s == resolved_path),
  }
}

pub fn is_module_external(package_json_info: &PackageJsonInfo, resolved_path: &str) -> bool {
  farm_profile_function!("is_module_external".to_string());
  let browser_field = get_field_value_from_package_json_info(package_json_info, "browser");
  if let Some(Value::Object(obj)) = browser_field {
    for (key, value) in obj {
      let path = Path::new(resolved_path);

      if matches!(value, Value::Bool(false)) {
        // resolved path
        if path.is_absolute() {
          let key_path = get_key_path(&key, package_json_info.dir());

          if key_path == resolved_path {
            return true;
          }
        } else {
          // source, e.g. 'foo' in require('foo')
          if key == resolved_path {
            return true;
          }
        }
      }
    }
  }

  false
}

/// Try resolve as a file with the configured extensions.
/// If `/root/index` exists, return `/root/index`, otherwise try `/root/index.[configured extension]` in order, once any extension exists (like `/root/index.ts`), return it immediately
pub fn try_file(file: &PathBuf, context: &Arc<CompilationContext>) -> Option<String> {
  // TODO add a test that for directory imports like `import 'comps/button'` where comps/button is a dir
  if file.exists() && file.is_file() {
    Some(file.to_string_lossy().to_string())
  } else {
    let append_extension = |file: &PathBuf, ext: &str| {
      let file_name = file.file_name().unwrap().to_string_lossy().to_string();
      file.with_file_name(format!("{}.{}", file_name, ext))
    };
    let ext = context.config.resolve.extensions.iter().find(|&ext| {
      let new_file = append_extension(file, ext);
      new_file.exists() && new_file.is_file()
    });

    ext.map(|ext| append_extension(file, ext).to_string_lossy().to_string())
  }
}

pub fn get_field_value_from_package_json_info(
  package_json_info: &PackageJsonInfo,
  field: &str,
) -> Option<Value> {
  let raw_package_json_info: Map<String, Value> = from_str(package_json_info.raw()).unwrap();

  if let Some(field_value) = raw_package_json_info.get(field) {
    return Some(field_value.clone());
  }

  None
}

/**
 * check if two paths are equal
 * Prevent path carrying / cause path resolution to fail
 */

pub fn are_values_equal<P1: AsRef<Path>, P2: AsRef<Path>>(path1: P1, path2: P2) -> bool {
  farm_profile_function!("are_values_equal".to_string());
  let path1 = PathBuf::from(path1.as_ref());
  let path2 = PathBuf::from(path2.as_ref());
  let path1_suffix = path1.strip_prefix("/").unwrap_or(&path1);
  let path2_suffix = path2.strip_prefix("/").unwrap_or(&path2);
  path1_suffix == path2_suffix
}

/**
 * get key path with other different key
 * TODO need add a argument (default | node) to determine the key
 */

pub fn get_key_path(key: &str, dir: &String) -> String {
  farm_profile_function!("get_key_path".to_string());
  let key_path = match Path::new(&key).is_relative() {
    true => {
      let resolve_key = &key.trim_matches('\"');
      RelativePath::new(resolve_key).to_logical_path(dir)
    }
    false => RelativePath::new("").to_logical_path(dir),
  };
  key_path.to_string_lossy().to_string()
}

/**
 * get normal path_value
 */
pub fn get_string_value_path(str: &str, package_json_info: &PackageJsonInfo) -> Option<String> {
  farm_profile_function!("get_string_value_path".to_string());
  let path = Path::new(&str);
  if path.extension().is_none() {
    // resolve imports field import other deps. import can only use relative paths
    return Some(path.to_string_lossy().to_string());
  } else {
    let value_path = get_key_path(str, package_json_info.dir());
    Some(value_path)
  }
}

pub fn get_path_from_value(value: &Value, package_json_info: &PackageJsonInfo) -> Option<String> {
  match value {
    Value::String(key_value_string) => {
      Some(get_key_path(key_value_string, package_json_info.dir()))
    }
    Value::Object(key_value_object) => key_value_object
      .get("default")
      .and_then(|default_str| default_str.as_str())
      .map(|default_str| get_key_path(default_str, package_json_info.dir())),
    _ => None,
  }
}

pub fn get_file_name_form_path(path: &str) -> Option<String> {
  farm_profile_function!("get_file_name_form_path".to_string());
  let path = Path::new(path);
  match path.file_name() {
    Some(file_name) => Some(file_name.to_string_lossy().to_string()),
    None => None,
  }
}

pub struct PathDifference {
  pub origin_request: String,
  pub remaining_request: String,
  pub query_params: HashMap<String, String>,
}

pub fn find_request_diff_entry_path(path_str1: &str, path_str2: &str) -> Option<PathDifference> {
  let origin_request = path_difference(path_str1, path_str2)?;

  let query_params = extract_query_params(path_str1);
  let remaining_request = if query_params.is_empty() {
    origin_request.to_string()
  } else {
    let result = if origin_request == "." {
      format!("./{}", origin_request.clone())
    } else {
      if let Some(query_start) = path_str1.find('?') {
        let query_string = &path_str1[(query_start)..];
        origin_request.to_string() + query_string
      } else {
        origin_request.to_string()
      }
    };
    result
  };

  Some(PathDifference {
    origin_request,
    remaining_request,
    query_params,
  })
}

pub fn path_difference(path_str1: &str, path_str2: &str) -> Option<String> {
  let path1 = PathBuf::from(path_str1.split('?').next().unwrap_or(""));
  let path2 = PathBuf::from(path_str2.split('?').next().unwrap_or(""));

  let relative_path1 = path1.strip_prefix(&path2).ok()?;

  Some(relative_path1.to_string_lossy().to_string())
}

pub fn extract_query_params(path_str: &str) -> HashMap<String, String> {
  let mut query_params_map = HashMap::new();

  if let Some(query_start) = path_str.find('?') {
    let query_string = &path_str[(query_start + 1)..];

    for param in query_string.split('&') {
      let parts: Vec<&str> = param.split('=').collect();
      if parts.len() == 2 {
        query_params_map.insert(parts[0].to_string(), parts[1].to_string());
      }
    }
  }

  query_params_map
}

pub fn find_mapping<'a>(key: &str, json_data: &'a serde_json::Map<String, Value>) -> Option<&'a Value> {
  match json_data.get(key) {
    Some(value) => Some(value),
    None => None,
  }
}
