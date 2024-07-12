use farmfe_utils::relative;
use relative_path::RelativePath;
use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub enum SideEffects {
  Bool(bool),
  Array(Vec<globset::GlobMatcher>),
}

impl Default for SideEffects {
  fn default() -> Self {
    Self::Bool(false)
  }
}

/// package json info that farm used.
/// **Note**: if you want to use the field that not defined here, you can deserialize raw and get the raw package.json [serde_json::Value]
#[derive(Debug, Clone, Default)]
pub struct PackageJsonInfo {
  pub name: Option<String>,
  pub version: Option<String>,

  parsed_side_effects: Option<SideEffects>,
  raw: Option<String>,
  raw_map: Option<Map<String, Value>>,
  /// the directory this package.json belongs to
  dir: Option<String>,
}

impl PackageJsonInfo {
  pub fn new(name: Option<String>, version: Option<String>) -> Self {
    Self {
      name,
      version,
      parsed_side_effects: None,
      raw: None,
      raw_map: None,
      dir: None,
    }
  }

  pub fn set_raw(&mut self, raw: String) {
    self.raw = Some(raw);
  }

  pub fn raw(&self) -> &String {
    self.raw.as_ref().unwrap()
  }

  pub fn set_raw_map(&mut self, raw_map: Map<String, Value>) {
    self.raw_map = Some(raw_map);
  }

  pub fn raw_map(&self) -> &Map<String, Value> {
    self.raw_map.as_ref().unwrap()
  }

  pub fn set_dir(&mut self, dir: String) {
    self.dir = Some(dir);
  }

  pub fn dir(&self) -> &String {
    self.dir.as_ref().unwrap()
  }

  /// parse the package.json and get parsed sideEffects info
  /// this method should be called after set_raw and set_dir
  pub fn parse(&mut self) {
    let package_value: serde_json::Map<String, Value> = serde_json::from_str(self.raw()).unwrap();

    self.analyze_parsed_side_effects(&package_value);
  }

  pub fn side_effects(&self) -> Option<&SideEffects> {
    self.parsed_side_effects.as_ref()
  }

  fn analyze_parsed_side_effects(&mut self, package_value: &serde_json::Map<String, Value>) {
    self.parsed_side_effects = if let Some(side_effects) = package_value.get("sideEffects") {
      if let Value::Bool(b) = side_effects {
        Some(SideEffects::Bool(*b))
      } else if let Value::Array(arr) = side_effects {
        let mut res = vec![];

        for item in arr {
          if let Value::String(str) = item {
            res.push(str.to_string());
          }
        }

        Some(SideEffects::Array(
          res
            .into_iter()
            .filter_map(|s| {
              let abs_path = RelativePath::new(&s).to_logical_path(self.dir());
              if let Ok(r) = globset::Glob::new(&relative(self.dir(), &abs_path.to_string_lossy()))
              {
                Some(r.compile_matcher())
              } else {
                None
              }
            })
            .collect::<Vec<_>>(),
        ))
      } else {
        // unknown side effects config, treat it as None
        None
      }
    } else {
      None
    };
  }
}
