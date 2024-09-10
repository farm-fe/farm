use farmfe_core::{common::PackageJsonInfo, farm_profile_function, regex, serde_json::Value};
use once_cell::sync::Lazy;
use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

use super::NODE_MODULES;

static PACKAGE_REGEX: Lazy<regex::Regex> =
  Lazy::new(|| regex::Regex::new(r"^(?P<group1>[^@][^/]*)|^(?P<group2>@[^/]+/[^/]+)").unwrap());

pub fn get_field_value_from_package_json_info(
  package_json_info: &PackageJsonInfo,
  field: &str,
) -> Option<Value> {
  package_json_info.raw_map().get(field).cloned()
}

pub fn is_source_relative(source: &str) -> bool {
  // fix: relative path start with .. or ../
  // source.starts_with("./") || source.starts_with("../") || source == "." || source == ".."
  source.starts_with('.')
    && (source.len() == 1 || source.starts_with("./") || source.starts_with(".."))
}

pub fn is_source_absolute(source: &str) -> bool {
  PathBuf::from_str(source).map_or(false, |p| p.is_absolute())
}

pub fn is_source_dot(source: &str) -> bool {
  source == "."
}

pub fn is_double_source_dot(source: &str) -> bool {
  source == ".."
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePackageSourceResult {
  pub package_name: String,
  pub sub_path: Option<String>,
}

pub fn parse_package_source(source: &str) -> Option<ParsePackageSourceResult> {
  farm_profile_function!("get_sub_path_of_source".to_string());
  let source = source.split('?').next()?;
  let captures = PACKAGE_REGEX.captures(source)?;
  let package_name = captures
    .name("group1")
    .or_else(|| captures.name("group2"))
    .map_or(source, |m| m.as_str());

  let sub_path = source
    .strip_prefix(package_name)
    .map(|s| format!(".{}", s))
    .filter(|_| package_name != source);

  Some(ParsePackageSourceResult {
    package_name: package_name.to_string(),
    sub_path,
  })
}

#[cfg(test)]
mod tests {
  #[test]
  fn get_sub_path_of_source() {
    let source = "lodash/clone";
    let result = super::parse_package_source(source).unwrap();
    assert_eq!(
      result,
      super::ParsePackageSourceResult {
        package_name: "lodash".to_string(),
        sub_path: Some("./clone".to_string())
      }
    );

    let source = "@babel/core/clone";
    let result = super::parse_package_source(source).unwrap();
    assert_eq!(
      result,
      super::ParsePackageSourceResult {
        package_name: "@babel/core".to_string(),
        sub_path: Some("./clone".to_string())
      }
    );

    let source = "clone";
    let result = super::parse_package_source(source).unwrap();
    assert_eq!(
      result,
      super::ParsePackageSourceResult {
        package_name: "clone".to_string(),
        sub_path: None
      }
    );

    let source = "http-proxy/lib/http-proxy/common";
    let result = super::parse_package_source(source).unwrap();
    assert_eq!(
      result,
      super::ParsePackageSourceResult {
        package_name: "http-proxy".to_string(),
        sub_path: Some("./lib/http-proxy/common".to_string())
      }
    );

    let source = "@/styles/index.css";
    let result = super::parse_package_source(source);
    assert_eq!(result, None);
  }
}
