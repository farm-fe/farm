use farmfe_core::{common::PackageJsonInfo, farm_profile_function, serde_json::Value};
use farmfe_utils::relative;

use crate::resolver::utils::get_field_value_from_package_json_info;

pub enum BrowserMapResult {
  Str(String),
  External,
}

#[derive(Clone)]
pub enum BrowserMapType {
  Source(String),
  ResolvedPath(String),
}

impl ToString for BrowserMapType {
  fn to_string(&self) -> String {
    match self {
      BrowserMapType::Source(s) => s.clone(),
      BrowserMapType::ResolvedPath(s) => s.clone(),
    }
  }
}

pub fn try_browser_map(
  package_json_info: &PackageJsonInfo,
  browser_map_type: BrowserMapType,
) -> Option<BrowserMapResult> {
  farm_profile_function!("try_browser_replace".to_string());

  let sub_path = match browser_map_type {
    BrowserMapType::Source(source) => source,
    BrowserMapType::ResolvedPath(resolved_path) => {
      format!("./{}", relative(package_json_info.dir(), &resolved_path))
    }
  };

  let browser_field = get_field_value_from_package_json_info(package_json_info, "browser");
  if let Some(Value::Object(obj)) = browser_field {
    for (key, value) in obj {
      if key == sub_path
        || key == format!("{sub_path}.js")
        || key == format!("{sub_path}/index.js")
      {
        match value {
          Value::String(str) => return Some(BrowserMapResult::Str(str.clone())),
          Value::Bool(false) => return Some(BrowserMapResult::External),
          _ => {}
        }
      }
    }
  }

  None
}
