use std::{
  collections::{BTreeMap, HashSet},
  str::FromStr,
  sync::Arc,
};

use farmfe_core::{
  common::PackageJsonInfo,
  config::Mode,
  context::CompilationContext,
  farm_profile_function,
  plugin::ResolveKind,
  rayon::iter::{IntoParallelRefIterator, ParallelIterator},
  regex,
  serde_json::Value,
};

use super::utils::get_field_value_from_package_json_info;

#[derive(Debug, Eq, PartialEq, Hash)]
enum Condition {
  Default,
  Require,
  Import,
  Browser,
  Node,
  Development,
  Module,
  Production,
  Custom(String),
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
      c => Ok(Condition::Custom(c.to_string())),
    }
  }
}

impl ToString for &Condition {
  fn to_string(&self) -> String {
    match self {
      Condition::Default => "default".to_string(),
      Condition::Require => "require".to_string(),
      Condition::Import => "import".to_string(),
      Condition::Browser => "browser".to_string(),
      Condition::Node => "node".to_string(),
      Condition::Development => "development".to_string(),
      Condition::Production => "production".to_string(),
      Condition::Module => "module".to_string(),
      Condition::Custom(c) => c.to_string(),
    }
  }
}

#[derive(Debug)]
struct ConditionOptions {
  pub unsafe_flag: bool,
  pub require: bool,
  pub browser: bool,
  pub conditions: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct ResolveExportsOrImportsResult {
  pub resolved: Option<String>,
  pub warnings: Vec<String>,
}

impl ResolveExportsOrImportsResult {
  pub fn to_resolved(self, strict_exports: bool) -> Option<String> {
    if strict_exports && self.warnings.len() > 0 {
      panic!(
        "Panic cause `resolve.strict_exports` is set to true:\n{}",
        self.warnings.join("\n")
      );
    }

    self.resolved
  }
}

pub fn resolve_exports_or_imports(
  package_json_info: &PackageJsonInfo,
  key: &str,
  field_type: &str,
  kind: &ResolveKind,
  context: &Arc<CompilationContext>,
) -> ResolveExportsOrImportsResult {
  farm_profile_function!("resolve_exports_or_imports".to_string());
  let mut additional_conditions: HashSet<String> =
    context.config.resolve.conditions.iter().cloned().collect();

  if !additional_conditions.contains(&String::from("production"))
    && !additional_conditions.contains(&String::from("development"))
  {
    additional_conditions.insert(match context.config.mode {
      Mode::Production => String::from("production"),
      Mode::Development => String::from("development"),
    });
  }

  // resolve exports field
  let is_browser = context.config.output.target_env.is_browser();
  let is_require = match kind {
    ResolveKind::Require => true,
    _ => false,
  };
  let condition_config = ConditionOptions {
    browser: is_browser && !additional_conditions.contains(&String::from("node")),
    require: is_require && !additional_conditions.contains(&String::from("import")),
    conditions: additional_conditions,
    // set default unsafe_flag to insert require & import field
    unsafe_flag: false,
  };

  if field_type == "imports" {
    imports(package_json_info, key, &condition_config)
  } else {
    exports(package_json_info, key, &condition_config)
  }
}

fn exports(
  package_json_info: &PackageJsonInfo,
  source: &str,
  config: &ConditionOptions,
) -> ResolveExportsOrImportsResult {
  if let Some(exports_field) = get_field_value_from_package_json_info(package_json_info, "exports")
  {
    // TODO If the current package does not have a name, then look up for the name of the folder
    let name = match get_field_value_from_package_json_info(package_json_info, "name") {
      Some(n) => n,
      None => {
        let warning = format!("Missing \"name\" field in package.json {package_json_info:?}");
        return ResolveExportsOrImportsResult {
          resolved: None,
          warnings: vec![warning],
        };
      }
    };

    let mut map: BTreeMap<String, Value> = BTreeMap::new();

    match exports_field {
      Value::String(string_value) => {
        map.insert(".".to_string(), Value::String(string_value));
      }
      Value::Object(object_value) => {
        for (k, v) in &object_value {
          if !k.starts_with('.') {
            map.insert(".".to_string(), Value::Object(object_value.clone()));
            break;
          } else {
            map.insert(k.to_string(), v.clone());
          }
        }
      }
      _ => {}
    }

    if !map.is_empty() {
      return walk(name.as_str().unwrap(), &map, source, config);
    }
  }

  ResolveExportsOrImportsResult::default()
}

fn imports(
  package_json_info: &PackageJsonInfo,
  source: &str,
  config: &ConditionOptions,
) -> ResolveExportsOrImportsResult {
  if let Some(imports_field) = get_field_value_from_package_json_info(package_json_info, "imports")
  {
    // TODO If the current package does not have a name, then look up for the name of the folder
    let name = match get_field_value_from_package_json_info(package_json_info, "name") {
      Some(n) => n,
      None => {
        let warning = format!("Missing \"name\" field in package.json {package_json_info:?}");
        return ResolveExportsOrImportsResult {
          resolved: None,
          warnings: vec![warning],
        };
      }
    };
    let mut imports_map: BTreeMap<String, Value> = BTreeMap::new();

    match imports_field {
      Value::Object(object_value) => {
        imports_map.extend(object_value.clone());
      }
      _ => {
        let warning = format!("Unexpected imports field format");
        return ResolveExportsOrImportsResult {
          resolved: None,
          warnings: vec![warning],
        };
      }
    }

    return walk(name.as_str().unwrap(), &imports_map, source, config);
  }

  ResolveExportsOrImportsResult::default()
}

/// [condition order](https://nodejs.org/api/packages.html#conditional-exports)
fn conditions(options: &ConditionOptions) -> HashSet<Condition> {
  // custom conditions should be first
  let mut conditions = options
    .conditions
    .iter()
    .map(|condition| Condition::from_str(condition).unwrap())
    .collect::<HashSet<_>>();

  conditions.insert(Condition::Default);

  if !options.unsafe_flag {
    if options.require {
      conditions.insert(Condition::Require);
    } else {
      conditions.insert(Condition::Import);
    }

    if options.browser {
      conditions.insert(Condition::Browser);
    } else {
      conditions.insert(Condition::Node);
    }
  }

  conditions
}

fn injects(item: Option<String>, value: &str) -> Option<String> {
  item.map(|mut item| {
    let rgx1: regex::Regex = regex::Regex::new(r#"\*"#).unwrap();
    let rgx2: regex::Regex = regex::Regex::new(r#"/$"#).unwrap();

    let tmp = item.clone();
    if rgx1.is_match(&tmp) {
      item = rgx1.replace(&tmp, value).to_string();
    } else if rgx2.is_match(&tmp) {
      item += value;
    }

    item
  })
}

fn loop_value(m: &Value, conditions: &HashSet<Condition>) -> Option<String> {
  match m {
    Value::String(s) => Some(s.to_string()),
    Value::Array(values) => values
      .par_iter()
      .find_map_first(|item| loop_value(item, conditions)),
    Value::Object(map) => {
      for (condition, val) in map.iter() {
        if conditions.contains(&Condition::from_str(condition.as_str()).unwrap()) {
          return loop_value(val, conditions);
        };
      }
      None
    }
    Value::Null => None,
    _ => None,
  }
}

fn throws(name: &str, entry: &str, condition: Option<i32>) -> String {
  match condition {
    Some(cond) if cond != 0 => {
      format!("No known conditions for \"{entry}\" specifier in \"{name}\" package")
    }
    _ => {
      format!("Missing \"{entry}\" specifier in \"{name}\" package")
    }
  }
}

fn walk(
  name: &str,
  mapping: &BTreeMap<String, Value>,
  input: &str,
  options: &ConditionOptions,
) -> ResolveExportsOrImportsResult {
  let entry: String = if input.starts_with(".") || input.starts_with("#") {
    input.to_string()
  } else {
    let warning = format!(
      "input must start with \".\" or \"#\" when walk `exports` or `imports` field of package.json"
    );
    return ResolveExportsOrImportsResult {
      resolved: None,
      warnings: vec![warning],
    };
  };

  let c = conditions(options);
  let mut m: Option<&Value> = mapping.get(&entry);
  let mut replace: Option<String> = None;

  if m.is_none() {
    let mut longest: Option<&str> = None;

    for (key, _) in mapping {
      if let Some(cur_longest) = &longest {
        if key.len() < cur_longest.len() {
          // do not allow "./" to match if already matched "./foo*" key
          continue;
        }
      }

      if key.ends_with('/') && entry.starts_with(key) {
        replace = Some(entry[key.len()..].to_string());
        longest = Some(key.as_str());
      } else if key.len() > 1 {
        if let Some(tmp) = key.find('*') {
          let pattern = format!("^{}(.*){}", &key[..tmp], &key[tmp + 1..]);
          let regex = regex::Regex::new(&pattern).unwrap();

          if let Some(captures) = regex.captures(&entry) {
            if let Some(match_group) = captures.get(1) {
              replace = Some(match_group.as_str().to_string());
              longest = Some(key.as_str());
            }
          }
        }
      }
    }

    if let Some(longest_key) = longest {
      m = mapping.get(longest_key);
    }
  }

  if m.is_none() {
    return ResolveExportsOrImportsResult {
      resolved: None,
      warnings: vec![throws(name, &entry, None)],
    };
  }

  let v = loop_value(m.as_ref().unwrap(), &c);

  if v.is_none() {
    return ResolveExportsOrImportsResult {
      resolved: None,
      warnings: vec![throws(name, &entry, Some(1))],
    };
  }

  if let Some(replace) = replace {
    return ResolveExportsOrImportsResult {
      resolved: injects(v, &replace),
      warnings: vec![],
    };
  }

  ResolveExportsOrImportsResult {
    resolved: v,
    warnings: vec![],
  }
}
