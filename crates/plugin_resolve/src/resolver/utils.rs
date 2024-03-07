use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

use farmfe_core::{
  common::PackageJsonInfo,
  farm_profile_function, regex,
  relative_path::RelativePath,
  serde_json::{from_str, Map, Value},
};

const PACKAGE_REGEX: &str = r"^(?P<group1>[^@][^/]*)/|^(?P<group2>@[^/]+/[^/]+)/";

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

/**
 * check if two paths are equal
 * Prevent path carrying / cause path resolution to fail
 */

pub fn are_paths_equal<P1: AsRef<Path>, P2: AsRef<Path>>(path1: P1, path2: P2) -> bool {
  farm_profile_function!("are_paths_equal".to_string());
  let path1 = PathBuf::from(path1.as_ref());
  let path2 = PathBuf::from(path2.as_ref());
  let path1_suffix = path1.strip_prefix("/").unwrap_or(&path1);
  let path2_suffix = path2.strip_prefix("/").unwrap_or(&path2);
  path1_suffix == path2_suffix
}

/**
 * get key path with other different key
 */
pub fn get_key_path(key: &str, dir: &String) -> String {
  farm_profile_function!("get_key_path".to_string());
  let key_path = match Path::new(&key).is_relative() {
    true => {
      let resolve_key = &key.trim_matches('\"');
      RelativePath::new(resolve_key).to_logical_path(dir)
    }
    false => RelativePath::new("").to_logical_path(key),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePackageSourceResult {
  pub package_name: String,
  pub sub_path: Option<String>,
}

pub fn parse_package_source(source: &str) -> ParsePackageSourceResult {
  farm_profile_function!("get_sub_path_of_source".to_string());

  // clean query of source
  let source = source.split('?').collect::<Vec<&str>>()[0];

  let regex = regex::Regex::new(PACKAGE_REGEX).unwrap();
  let captures = regex.captures(source).unwrap();

  let package_name = if let Some(group1) = captures.name("group1") {
    group1.as_str()
  } else if let Some(group2) = captures.name("group2") {
    group2.as_str()
  } else {
    source
  };

  let sub_path = if package_name == source {
    None
  } else {
    Some(format!(".{}", source.replace(package_name, "")))
  };

  ParsePackageSourceResult {
    package_name: package_name.to_string(),
    sub_path,
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn get_sub_path_of_source() {
    let source = "lodash/clone";
    let result = super::parse_package_source(source);
    assert_eq!(
      result,
      super::ParsePackageSourceResult {
        package_name: "lodash".to_string(),
        sub_path: Some("./clone".to_string())
      }
    );

    let source = "@babel/core/clone";
    let result = super::parse_package_source(source);
    assert_eq!(
      result,
      super::ParsePackageSourceResult {
        package_name: "@babel/core".to_string(),
        sub_path: Some("./clone".to_string())
      }
    );

    let source = "clone";
    let result = super::parse_package_source(source);
    assert_eq!(
      result,
      super::ParsePackageSourceResult {
        package_name: "clone".to_string(),
        sub_path: None
      }
    );
  }
}
