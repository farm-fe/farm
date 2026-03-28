use std::{collections::HashSet, fs, path::Path};

use crate::find_local_components::{ComponentInfo, ExportType};
use farmfe_core::{config::config_regex::ConfigRegex, regex::Regex};
use farmfe_toolkit::resolve::package_json_loader::{Options, PackageJsonLoader};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, fmt::Formatter};

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub enum ImportStyle {
  Bool(bool),
  String(String),
}

impl Serialize for ImportStyle {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match *self {
      ImportStyle::Bool(ref b) => serializer.serialize_bool(*b),
      ImportStyle::String(ref s) => serializer.serialize_str(s),
    }
  }
}

impl<'de> Deserialize<'de> for ImportStyle {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct StringOrBoolVisitor;
    impl<'de> Visitor<'de> for StringOrBoolVisitor {
      type Value = ImportStyle;
      fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a boolean or a string")
      }
      fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(ImportStyle::Bool(value))
      }
      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(ImportStyle::String(value.to_owned()))
      }
    }
    deserializer.deserialize_any(StringOrBoolVisitor)
  }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ResolverOption {
  pub module: String,
  pub prefix: Option<String>,
  pub export_type: Option<ExportType>,
  pub import_style: Option<ImportStyle>,
  pub exclude: Option<Vec<ConfigRegex>>,
  pub include: Option<Vec<ConfigRegex>>,
}

pub fn get_resolvers_result(
  root_path: &str,
  resolvers: Vec<ResolverOption>,
) -> HashSet<ComponentInfo> {
  let mut resolver_set = HashSet::new();
  for item in resolvers {
    let components = get_resolvers(root_path, item);
    for ele in components {
      resolver_set.insert(ele);
    }
  }
  resolver_set
}

pub fn get_resolvers(root_path: &str, component_lib: ResolverOption) -> Vec<ComponentInfo> {
  let mut components = vec![];
  let prefix = &component_lib.prefix.unwrap_or("".to_string());
  let loader = PackageJsonLoader::new();
  let package_path = Path::new(root_path).join(format!("node_modules/{}", &component_lib.module));
  let package_json = loader
    .load(
      package_path.clone(),
      Options {
        follow_symlinks: false,
        resolve_ancestor_dir: false,
      },
    )
    .unwrap();
  let types = package_json.raw_map().get("types");
  let typings = package_json.raw_map().get("typings");
  let relative_type_file = {
    if let Some(typings) = typings {
      typings.as_str().unwrap()
    } else if let Some(types) = types {
      types.as_str().unwrap()
    } else {
      "index.d.ts"
    }
  };
  let type_file = package_path.join(Path::new(relative_type_file));
  let content = fs::read_to_string(type_file).expect("Failed to read file");
  let import_style = component_lib
    .import_style
    .unwrap_or(ImportStyle::Bool(false));
  let re = Regex::new(r"export\s+\{\s*default\s+as\s+(\w+)\s*\}\s+from\s+'\.\/(\w+)';").unwrap();
  for cap in re.captures_iter(&content) {
    components.push(ComponentInfo {
      name: format!("{}{}", prefix, cap[1].to_string()),
      path: component_lib.module.clone(),
      export_type: ExportType::Named,
      original_name: cap[1].to_string(),
      import_style: import_style.clone(),
      is_local: false,
    })
  }
  components
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env;
  #[test]
  fn test_generate_dts() {
    let current_dir = env::current_dir().unwrap();
    let binding = current_dir.join("playground");
    let root_path = binding.to_str().unwrap();
    let resolver_option = ResolverOption {
      module: "antd".to_string(),
      export_type: Some(ExportType::Named),
      import_style: Some(ImportStyle::Bool(false)),
      exclude: None,
      include: None,
      prefix: Some("Ant".to_string()),
    };

    let components = get_resolvers(root_path, resolver_option);
    println!("components:{:#?}", components);
  }
}
