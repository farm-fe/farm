use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  str::FromStr,
  sync::Arc,
};

use farmfe_core::{
  common::PackageJsonInfo,
  context::CompilationContext,
  farm_profile_function,
  hashbrown::{HashMap, HashSet},
  regex,
  relative_path::RelativePath,
  serde_json::{self, from_str, Map, Value},
};
use farmfe_toolkit::lazy_static::lazy_static;

pub const SLASH_CODE: char = '/';

pub const DOT_CODE: char = '.';

pub const HASH_CODE: char = '#';

pub const NODE_MODULES: &str = "node_modules";

lazy_static! {
  pub static ref EXTERNAL_RE: regex::Regex = regex::Regex::new(r#"^(https?:)?//"#).unwrap();
}

lazy_static! {
  pub static ref DATA_URL_RE: regex::Regex = regex::Regex::new(r#"^[\s]*data:"#).unwrap();
}

// lazy_static! {
//   pub static ref DEEP_IMPORT_RE: regex::Regex =
//     regex::Regex::new(r"^([^@][^/]*)/|^(@[^/]+/[^/]+)").unwrap();
// }

lazy_static! {
  pub static ref DEEP_IMPORT_RE: regex::Regex =
    regex::Regex::new(r"([^@][^/]*)/|(@[^/]+/[^/]+)").unwrap();
}

lazy_static! {
  static ref POSTFIX_RE: regex::Regex = regex::Regex::new(r"[?#].*").unwrap();
}

pub fn is_external_url(url: &str) -> bool {
  EXTERNAL_RE.is_match(url)
}

pub fn is_data_url(url: &str) -> bool {
  DATA_URL_RE.is_match(url)
}

pub fn is_bare_import_path(id: &str) -> bool {
  if id.starts_with('@') {
    // Check if it's an "@"-based path
    if let Some(rest) = id.strip_prefix('@') {
      let parts: Vec<&str> = rest.split('/').collect();
      return !parts[1].contains("://");
    }
  } else {
    // Check if it's a non-Windows absolute path
    return !id.contains(':') && !id.contains("://");
  }
  false
}

pub struct PathDifference {
  pub origin_request: String,
  pub remaining_request: String,
  pub query_params: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Condition {
  Default,
  Require,
  Import,
  Browser,
  Node,
  Development,
  Module,
  Production,
}

#[derive(Debug)]
pub struct ConditionOptions {
  pub unsafe_flag: bool,
  pub require: bool,
  pub browser: bool,
  pub conditions: Vec<String>,
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

fn clean_url(url: &str) -> String {
  POSTFIX_RE.replace_all(url, "").to_string()
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
  // if file.to_string_lossy().contains("/esm/is-string") {
  //   println!("开始搞 file: {:?}", file);
  // }
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
    // if file.to_string_lossy().contains("/esm/is-string") {
    //   println!("ext: {:?}", ext.map(|ext| append_extension(file, ext).to_string_lossy().to_string()));
    // }
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

pub fn is_in_node_modules(id: &str) -> bool {
  id.contains("node_modules")
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

pub fn conditions(options: &ConditionOptions) -> HashSet<Condition> {
  let mut out: HashSet<Condition> = HashSet::new();
  out.insert(Condition::Default);
  // TODO resolver other conditions
  // for condition in options.conditions.iter() {
  //   out.insert(condition.parse().unwrap());
  // }
  for condition_str in &options.conditions {
    match Condition::from_str(condition_str) {
      Ok(condition_enum) => {
        out.insert(condition_enum);
      }
      Err(error) => {
        // TODO resolve error
        eprintln!("Error: {}", error);
      }
    }
  }
  if !options.unsafe_flag {
    if options.require {
      out.insert(Condition::Require);
    } else {
      out.insert(Condition::Import);
    }

    if options.browser {
      out.insert(Condition::Browser);
    } else {
      out.insert(Condition::Node);
    }
  }
  out
}

pub fn injects(items: &mut Vec<String>, value: &str) -> Option<Vec<String>> {
  let rgx1: regex::Regex = regex::Regex::new(r#"\*"#).unwrap();
  let rgx2: regex::Regex = regex::Regex::new(r#"/$"#).unwrap();

  for item in items.iter_mut() {
    let tmp = item.clone();
    if rgx1.is_match(&tmp) {
      *item = rgx1.replace(&tmp, value).to_string();
    } else if rgx2.is_match(&tmp) {
      *item += value;
    }
  }

  return items.clone().into_iter().map(|s| Some(s)).collect();
}

pub fn loop_value(
  m: Value,
  keys: &HashSet<Condition>,
  mut result: &mut Option<HashSet<String>>,
) -> Option<Vec<String>> {
  match m {
    Value::String(s) => {
      if let Some(result_set) = result {
        result_set.insert(s.clone());
      }
      Some(vec![s])
    }
    Value::Array(values) => {
      let arr_result = result.clone().unwrap_or_else(|| HashSet::new());
      for item in values {
        if let Some(item_result) = loop_value(item, keys, &mut Some(arr_result.clone())) {
          return Some(item_result);
        }
      }

      // 如果使用了初始化的结果集，返回结果
      if result.is_none() && !arr_result.is_empty() {
        return Some(arr_result.into_iter().collect());
      } else {
        None
      }
    }
    Value::Object(map) => {
      let property_order: Vec<String> = vec![
        String::from("browser"),
        String::from("development"),
        String::from("module"),
        String::from("node"),
        String::from("import"),
        String::from("require"),
        String::from("default"),
      ];

      for key in &property_order {
        if let Some(value) = map.get(key) {
          if let Ok(condition) = Condition::from_str(&key) {
            if keys.contains(&condition) {
              return loop_value(value.clone(), keys, result);
            }
          }
        }
      }
      None
    }
    Value::Null => None,
    _ => None,
  }
}

pub fn to_entry(name: &str, ident: &str, externals: Option<bool>) -> Result<String, String> {
  if name == ident || ident == "." {
    return Ok(".".to_string());
  }

  let root = format!("{}/", name);
  let len = root.len();
  let bool = ident.starts_with(&root);
  let output = if bool {
    ident[len..].to_string()
  } else {
    ident.to_string()
  };

  if output.starts_with('#') {
    return Ok(output);
  }

  if bool || externals.unwrap_or(false) {
    if output.starts_with("./") {
      Ok(output)
    } else {
      Ok(format!("./{}", output))
    }
  } else {
    Err(output)
  }
}

pub enum Entry {
  Exports(String),
  Imports(String),
}
pub fn throws(name: &str, entry: &str, condition: Option<i32>) {
  let message = if let Some(cond) = condition {
    if cond != 0 {
      format!(
        "No known conditions for \"{}\" specifier in \"{}\" package",
        entry, name
      )
    } else {
      format!("Missing \"{}\" specifier in \"{}\" package", entry, name)
    }
  } else {
    format!("Missing \"{}\" specifier in \"{}\" package", entry, name)
  };
  eprintln!("{}", message);
}

#[derive(Hash)]
pub enum EntryKey {
  Normal(String),
  Dot(String),
  Hash(String),
}

impl EntryKey {
  fn from_string(input: &str) -> Self {
    let re = regex::Regex::new(r"^(?P<type>[.#])(?P<value>.+)$").unwrap();
    if let Some(caps) = re.captures(input) {
      match &caps["type"] {
        "." => EntryKey::Dot(caps["value"].to_string()),
        "#" => EntryKey::Hash(caps["value"].to_string()),
        _ => EntryKey::Normal(input.to_string()),
      }
    } else {
      EntryKey::Normal(input.to_string())
    }
  }

  fn as_str(&self) -> &str {
    match self {
      EntryKey::Normal(s) | EntryKey::Dot(s) | EntryKey::Hash(s) => s.as_str(),
    }
  }
}

// 实现 Eq 和 PartialEq trait
impl Eq for EntryKey {}

impl PartialEq for EntryKey {
  fn eq(&self, other: &Self) -> bool {
    self.as_str() == other.as_str()
  }
}

pub fn walk(
  name: &str,
  mapping: &HashMap<String, Value>,
  input: &str,
  options: &ConditionOptions,
) -> Vec<String> {
  let entry_result: Result<String, String> = to_entry(name, input, Some(true));
  let entry: String = match entry_result {
    Ok(entry) => entry.to_string(),
    Err(error) => {
      eprintln!("Error resolve {} package error: {}", name, error);
      String::from(name)
    }
  };
  let c: HashSet<Condition> = conditions(options);
  let mut m: Option<&Value> = mapping.get(&entry);
  let mut result: Option<Vec<String>> = None;
  let mut replace: Option<String> = None;
  if m.is_none() {
    let mut longest: Option<&str> = None;

    for (key, value) in mapping.iter() {
      if let Some(cur_longest) = &longest {
        if key.len() < cur_longest.len() {
          // do not allow "./" to match if already matched "./foo*" key
          continue;
        }
      }

      if key.ends_with('/') && entry.starts_with(key) {
        replace = Some(entry[key.len()..].to_string());
        longest = Some(key.as_str().clone());
      } else if key.len() > 1 {
        if let Some(tmp) = key.find('*') {
          let pattern = format!("^{}(.*){}", &key[..tmp], &key[tmp + 1..]);
          let regex = regex::Regex::new(&pattern).unwrap();

          if let Some(captures) = regex.captures(&entry) {
            if let Some(match_group) = captures.get(1) {
              replace = Some(match_group.as_str().to_string());
              longest = Some(key.as_str().clone());
            }
          }
        }
      }
    }

    if let Some(longest_key) = longest {
      m = mapping.get(&longest_key.to_string());
    }
  }
  if m.is_none() {
    throws(name, &entry, None);
    return Vec::new(); // 返回一个空 Vec 作为错误处理的默认值
  }
  let v = loop_value(m.unwrap().clone(), &c, &mut None);
  if v.is_none() {
    throws(name, &entry, Some(1));
    return Vec::new(); // 返回一个空 Vec 作为错误处理的默认值
  }
  let mut cloned_v = v.clone();
  if let Some(replace) = replace {
    if let Some(v1) = injects(&mut cloned_v.unwrap(), &replace) {
      return v1;
    }
  }
  v.unwrap()
}

impl FromStr for Condition {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "default" => Ok(Condition::Default),
      "require" => Ok(Condition::Require),
      "import" => Ok(Condition::Import),
      "browser" => Ok(Condition::Browser),
      "node" => Ok(Condition::Node),
      "development" => Ok(Condition::Development),
      "production" => Ok(Condition::Production),
      "module" => Ok(Condition::Module),
      _ => Err(format!("Invalid Condition: {}", s)),
    }
  }
}

pub fn map_with_browser_field<'a>(relative_path: &'a str, map: &'a Value) -> Option<String> {
  // 使用 Path::new 来创建一个路径对象
  let path = Path::new(relative_path);

  // 使用 file_name() 方法来获取路径中的文件名部分
  let normalized_path = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

  if let Some(map_object) = map.as_object() {
    for (key, value) in map_object {
      // TODO 这快不对 规范化路径这快不对
      let normalized_key = Path::new(key)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");
      if normalized_path == normalized_key
        || equal_without_suffix(normalized_path, normalized_key, ".js")
        || equal_without_suffix(normalized_path, normalized_key, "/index.js")
      {
        if let Some(value_str) = value.as_str() {
          return Some(value_str.to_string());
        }
      }
    }
  }

  None
}

pub fn equal_without_suffix(path: &str, key: &str, suffix: &str) -> bool {
  key.ends_with(suffix) && &key[..key.len() - suffix.len()] == path
}

pub fn split_file_and_postfix(path: &str) -> (String, String) {
  let file = clean_url(path);
  let postfix = path[file.len()..].to_string();
  (file, postfix)
}

pub fn get_real_path(resolved: &str, preserve_symlinks: bool) -> String {
  let mut resolved_path = resolved.to_string();

  if !preserve_symlinks {
    if let Ok(real_path) = safe_real_path_sync(&resolved) {
      resolved_path = real_path.to_string_lossy().to_string();
    }
  }

  normalize_path(&resolved_path)
}

pub fn safe_real_path_sync(resolved: &str) -> Result<PathBuf, std::io::Error> {
  fs::canonicalize(resolved)
}

pub fn normalize_path(resolved: &str) -> String {
  let path = Path::new(resolved);
  let normalized_path = path.to_string_lossy().to_string();
  normalized_path
}

pub fn get_directory_path(file: &str) -> String {
  let file_path = Path::new(file);
  if let Some(parent) = file_path.parent() {
    // Use .to_str().unwrap() to convert the path to a string
    return parent.to_str().unwrap().to_string();
  }
  "".to_string()
}
