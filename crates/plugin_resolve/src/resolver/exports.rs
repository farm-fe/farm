use std::{collections::BTreeMap, fmt::Display, str::FromStr, sync::Arc};

use farmfe_core::{
  common::PackageJsonInfo,
  config::Mode,
  context::CompilationContext,
  farm_profile_function,
  plugin::ResolveKind,
  rayon::iter::{IntoParallelRefIterator, ParallelIterator},
  regex,
  serde_json::Value,
  HashSet,
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

impl Display for &Condition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Condition::Default => write!(f, "default"),
      Condition::Require => write!(f, "require"),
      Condition::Import => write!(f, "import"),
      Condition::Browser => write!(f, "browser"),
      Condition::Node => write!(f, "node"),
      Condition::Development => write!(f, "development"),
      Condition::Production => write!(f, "production"),
      Condition::Module => write!(f, "module"),
      Condition::Custom(c) => write!(f, "{}", c),
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

pub fn resolve_exports_or_imports(
  package_json_info: &PackageJsonInfo,
  key: &str,
  field_type: &str,
  kind: &ResolveKind,
  context: &Arc<CompilationContext>,
) -> Option<Vec<String>> {
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
  let is_require = matches!(kind, ResolveKind::Require);
  let condition_config = ConditionOptions {
    browser: is_browser && !additional_conditions.contains(&String::from("node")),
    require: is_require && !additional_conditions.contains(&String::from("import")),
    conditions: additional_conditions,
    // set default unsafe_flag to insert require & import field
    unsafe_flag: false,
  };

  let result: Option<Vec<String>> = if field_type == "imports" {
    imports(package_json_info, key, &condition_config)
  } else {
    exports(package_json_info, key, &condition_config)
  };
  result
}

fn exports(
  package_json_info: &PackageJsonInfo,
  source: &str,
  config: &ConditionOptions,
) -> Option<Vec<String>> {
  if let Some(exports_field) = get_field_value_from_package_json_info(package_json_info, "exports")
  {
    // TODO If the current package does not have a name, then look up for the name of the folder
    let name = match get_field_value_from_package_json_info(package_json_info, "name") {
      Some(n) => n,
      None => {
        eprintln!("Missing \"name\" field in package.json {package_json_info:?}");
        return None;
      }
    };
    let mut map: BTreeMap<String, Value> = BTreeMap::new();
    match exports_field {
      Value::String(string_value) => {
        map.insert(".".to_string(), Value::String(string_value.clone()));
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
      return Some(walk(name.as_str().unwrap(), &map, source, config));
    }
  }

  None
}

fn imports(
  package_json_info: &PackageJsonInfo,
  source: &str,
  config: &ConditionOptions,
) -> Option<Vec<String>> {
  if let Some(imports_field) = get_field_value_from_package_json_info(package_json_info, "imports")
  {
    // TODO If the current package does not have a name, then look up for the name of the folder
    let name = match get_field_value_from_package_json_info(package_json_info, "name") {
      Some(n) => n,
      None => {
        eprintln!("Missing \"name\" field in package.json {package_json_info:?}");
        return None;
      }
    };
    let mut imports_map: BTreeMap<String, Value> = BTreeMap::new();

    match imports_field {
      Value::Object(object_value) => {
        imports_map.extend(object_value.clone());
      }
      _ => {
        eprintln!("Unexpected imports field format");
        return None;
      }
    }
    return Some(walk(name.as_str().unwrap(), &imports_map, source, config));
  }
  None
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

fn injects(items: &mut [String], value: &str) -> Option<Vec<String>> {
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

  items.iter().cloned().map(Some).collect()
}

fn loop_value(
  m: Value,
  conditions: &HashSet<Condition>,
  result: &mut Option<HashSet<String>>,
) -> Option<Vec<String>> {
  match m {
    Value::String(s) => {
      if let Some(result_set) = result {
        result_set.insert(s.clone());
      }
      Some(vec![s])
    }
    Value::Array(values) => {
      let arr_result = result.clone().unwrap_or_default();
      values
        .par_iter()
        .find_map_first(|item| loop_value(item.clone(), conditions, &mut Some(arr_result.clone())))
    }
    Value::Object(map) => {
      for (condition, val) in map.iter() {
        if conditions.contains(&Condition::from_str(condition.as_str()).unwrap()) {
          return loop_value(val.clone(), conditions, result);
        };
      }
      None
    }
    Value::Null => None,
    _ => None,
  }
}

fn throws(name: &str, entry: &str, condition: Option<i32>) {
  let message = match condition {
    Some(cond) if cond != 0 => {
      format!("No known conditions for \"{entry}\" specifier in \"{name}\" package")
    }
    _ => {
      format!("Missing \"{entry}\" specifier in \"{name}\" package")
    }
  };
  eprintln!("{message}");
}

fn walk(
  name: &str,
  mapping: &BTreeMap<String, Value>,
  input: &str,
  options: &ConditionOptions,
) -> Vec<String> {
  let entry: String = if input.starts_with(".") || input.starts_with("#") {
    input.to_string()
  } else {
    panic!(
      "input must start with \".\" or \"#\" when walk `exports` or `imports` field of package.json"
    )
  };

  let c = conditions(options);
  let mut m: Option<&Value> = mapping.get(&entry);
  let mut replace: Option<String> = None;
  if m.is_none() {
    let mut longest: Option<&str> = None;

    for (key, _value) in mapping.iter() {
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
    throws(name, &entry, None);
    return Vec::new();
  }
  let v = loop_value(m.unwrap().clone(), &c, &mut None);
  if v.is_none() {
    throws(name, &entry, Some(1));
    return Vec::new();
  }
  let cloned_v = v.clone();
  if let Some(replace) = replace {
    if let Some(v1) = injects(&mut cloned_v.unwrap(), &replace) {
      return v1;
    }
  }
  v.unwrap()
}
