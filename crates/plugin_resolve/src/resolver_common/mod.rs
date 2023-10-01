use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  str::FromStr,
  sync::Arc,
};

use farmfe_core::{
  common::PackageJsonInfo,
  context::CompilationContext,
  farm_profile_function,
  relative_path::RelativePath,
  serde_json::{self, from_str, Map, Value},
};

pub const SLASH_CODE: char = '/';

pub const DOT_CODE: char = '.';

pub const HASH_CODE: char = '#';

pub const NODE_MODULES: &str = "node_modules";

pub struct PathDifference {
  pub origin_request: String,
  pub remaining_request: String,
  pub query_params: HashMap<String, String>,
}

/**
 * Checks if a source string is a relative path.
 *
 * @param {string} source - The source string to check.
 * @returns {boolean} True if the source is a relative path starting with ./ or ../, false otherwise.
 */
pub fn is_source_relative(source: &str) -> bool {
  // fix: relative path start with .. or ../
  source.starts_with("./") || source.starts_with("../")
}

/**
 * Checks if a source string is an absolute path.
 *
 * @param {string} source - The source string to check.
 * @returns {boolean} True if the source can be converted to an absolute PathBuf, false otherwise.
 */
pub fn is_source_absolute(source: &str) -> bool {
  if let Ok(sp) = PathBuf::from_str(source) {
    sp.is_absolute()
  } else {
    false
  }
}

/**
 * Checks if the source string is ".".
 *
 * @param {string} source - The source string to check.
 * @returns {boolean} True if source is ".", false otherwise.
 */

pub fn is_source_dot(source: &str) -> bool {
  source == "."
}

/**
 * Checks if the source string is "..".
 *
 * @param {string} source - The source string to check.
 * @returns {boolean} True if source is "..", false otherwise.
*/

pub fn is_double_source_dot(source: &str) -> bool {
  source == ".."
}

/**
 * Checks if a module has side effects based on package.json#sideEffects.
 *
 * @param {PackageJsonInfo} packageJsonInfo - The package.json information.
 * @param {string} resolvedPath - The resolved path of the module.
 * @returns {boolean} True if module has side effects, false otherwise.
 */

pub fn is_module_side_effects(package_json_info: &PackageJsonInfo, resolved_path: &str) -> bool {
  farm_profile_function!("is_module_side_effects".to_string());
  match package_json_info.side_effects() {
    farmfe_core::common::ParsedSideEffects::Bool(b) => *b,
    farmfe_core::common::ParsedSideEffects::Array(arr) => arr.iter().any(|s| s == resolved_path),
  }
}

/**
 * Checks if a module is marked as external in package.json#browser field.
 *
 * @param {PackageJsonInfo} packageJsonInfo - The package.json information.
 * @param {string} resolvedPath - The resolved path of the module.
 * @returns {boolean} True if module is marked as external, false otherwise.
*/

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

/**
 * Attempts to locate a file based on the provided file path. If the file exists, it returns
 * the string representation of the file path. If the file does not exist, it appends configured
 * file extensions to the original path in an attempt to find a matching file with an extension.
 *
 * @param {PathBuf} file - The file path to try locating.
 * @param {CompilationContext} context - The compilation context object.
 * @returns {string|null} The string representation of the found file path, or null if not found.
 */

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

/**
 * Get the value of a specific field from the given `PackageJsonInfo` object.
 *
 * @param {PackageJsonInfo} packageJsonInfo - The `PackageJsonInfo` object containing the original JSON data.
 * @param {string} field - The name of the field to retrieve its value.
 * @returns {Value|null} The value of the specified field if found, or `null` if the field is not present.
 */

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
 * Compare two paths for equality.
 *
 * @param {string|Path} path1 - The first path to compare.
 * @param {string|Path} path2 - The second path to compare.
 * @returns {boolean} `true` if the paths are equal, `false` otherwise.
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

/**
 * Generate a path based on the given key and directory.
 *
 * @param {string} key - The key used to generate the path.
 * @param {string} dir - The base directory for path generation.
 * @returns {string} The generated path.
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
 * Generate a path based on the given string and PackageJsonInfo.
 *
 * @param {string} str - The string used to generate the path.
 * @param {PackageJsonInfo} package_json_info - Information related to the project, including the 'dir' field.
 * @returns {string|null} The generated path or null if the string does not have an extension.
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

/**
 * Generate a path based on the given Value and PackageJsonInfo.
 *
 * @param {Value} value - The Value representing a configuration item's value.
 * @param {PackageJsonInfo} package_json_info - Information related to the project, including the 'dir' field.
 * @returns {string|null} The generated path or null if unable to generate a path from the Value.
 */

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

/**
 * Extract the file name from a given path.
 *
 * @param {string} path - The path from which to extract the file name.
 * @returns {string|null} The extracted file name or null if unable to extract it.
 */

pub fn get_file_name_form_path(path: &str) -> Option<String> {
  farm_profile_function!("get_file_name_form_path".to_string());
  let path = Path::new(path);
  match path.file_name() {
    Some(file_name) => Some(file_name.to_string_lossy().to_string()),
    None => None,
  }
}

/**
 * Compare two path strings and find the difference between them.
 *
 * @param {string} path_str1 - The first path string to compare.
 * @param {string} path_str2 - The second path string to compare.
 * @returns {PathDifference|null} An object containing the difference between the paths, or null if they are identical.
 */

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

/**
 * Compare two path strings and find the difference between them.
 *
 * @param {string} path_str1 - The first path string to compare.
 * @param {string} path_str2 - The second path string to compare.
 * @returns {string|null} The difference between the paths, or null if they are identical.
 */

pub fn path_difference(path_str1: &str, path_str2: &str) -> Option<String> {
  let path1 = PathBuf::from(path_str1.split('?').next().unwrap_or(""));
  let path2 = PathBuf::from(path_str2.split('?').next().unwrap_or(""));

  let relative_path1 = path1.strip_prefix(&path2).ok()?;

  Some(relative_path1.to_string_lossy().to_string())
}

/**
 * Extract query parameters from a given path string and store them in a HashMap.
 *
 * @param {string} path_str - The path string containing query parameters.
 * @returns {Map<string, string>} A HashMap containing the query parameters as key-value pairs.
 */

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

/**
 * Find a mapping with a specified key in a JSON data.
 *
 * @param {string} key - The key to search for in the JSON data.
 * @param {Object.<string, any>} json_data - The JSON data containing the mappings.
 * @returns {any | null} The value associated with the specified key, or null if the key is not found.
 */

pub fn find_mapping<'a>(
  key: &str,
  json_data: &'a serde_json::Map<String, Value>,
) -> Option<&'a Value> {
  match json_data.get(key) {
    Some(value) => Some(value),
    None => None,
  }
}

/**
 * Get a result path based on the provided value and current resolve base directory.
 *
 * @param {string} value - The value for which to generate a path.
 * @param {string} current_resolve_base_dir - The current base directory for resolution.
 * @returns {string | null} The resulting path or null if it cannot be determined.
 */

pub fn get_result_path(value: &str, current_resolve_base_dir: &String) -> Option<String> {
  Some(get_key_path(value, current_resolve_base_dir))
}
