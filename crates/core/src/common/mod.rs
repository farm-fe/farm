use farmfe_macro_cache_item::cache_item;
use relative_path::RelativePath;
use serde_json::Value;

#[cache_item]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ParsedSideEffects {
  Bool(bool),
  Array(Vec<String>),
}

impl Default for ParsedSideEffects {
  fn default() -> Self {
    Self::Bool(false)
  }
}

/// package json info that farm used.
/// **Note**: if you want to use the field that not defined here, you can deserialize raw and get the raw package.json [serde_json::Value]
#[cache_item]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageJsonInfo {
  pub name: Option<String>,
  pub version: Option<String>,

  parsed_side_effects: Option<ParsedSideEffects>,
  raw: Option<String>,
  /// the directory this package.json belongs to
  dir: Option<String>,
}

impl PackageJsonInfo {
  pub fn set_raw(&mut self, raw: String) {
    self.raw = Some(raw);
  }

  pub fn raw(&self) -> &String {
    self.raw.as_ref().unwrap()
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

  pub fn side_effects(&self) -> &ParsedSideEffects {
    self.parsed_side_effects.as_ref().unwrap()
  }

  fn analyze_parsed_side_effects(&mut self, package_value: &serde_json::Map<String, Value>) {
    let parsed_side_effects = if let Some(side_effects) = package_value.get("sideEffects") {
      if let Value::Bool(b) = side_effects {
        ParsedSideEffects::Bool(*b)
      } else if let Value::Array(arr) = side_effects {
        let mut res = vec![];

        for item in arr {
          if let Value::String(str) = item {
            let abs_path = RelativePath::new(str).to_logical_path(self.dir());
            // TODO throw a graceful error
            let paths = glob::glob(abs_path.to_str().unwrap()).unwrap();

            for p in paths.flatten() {
              let path = p.to_str().unwrap().to_string();
              res.push(path);
            }
          }
        }

        ParsedSideEffects::Array(res)
      } else {
        ParsedSideEffects::Bool(false)
      }
    } else {
      ParsedSideEffects::Bool(false)
    };

    self.parsed_side_effects = Some(parsed_side_effects);
  }
}
