use serde::{Deserialize, Serialize};

use super::{config_regex::ConfigRegex, Config};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ExternalObject {
  pub pattern: ConfigRegex,
  pub global_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ExternalConfigItem {
  Default(ConfigRegex),
  Object(ExternalObject),
}

impl Default for ExternalConfigItem {
  fn default() -> Self {
    Self::Default(ConfigRegex::default())
  }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ExternalConfig(pub Vec<ExternalConfigItem>);

impl ExternalConfig {
  pub fn new() -> Self {
    Self(Vec::new())
  }

  pub fn find_match(&self, source: &str) -> Option<&ExternalConfigItem> {
    self.0.iter().find(|item| item.is_match(source))
  }

  pub fn is_external(&self, source: &str) -> bool {
    self.find_match(source).is_some()
  }
}

impl ExternalConfigItem {
  pub fn is_match(&self, source: &str) -> bool {
    match self {
      Self::Default(regex) => regex.is_match(source),
      Self::Object(kv) => kv.pattern.is_match(source),
    }
  }

  pub fn source(&self, source: &str) -> String {
    match self {
      Self::Default(_) => source.to_string(),
      Self::Object(obj) => obj.global_name.to_string(),
    }
  }
}

impl From<&Config> for ExternalConfig {
  fn from(config: &Config) -> Self {
    let mut external_config = ExternalConfig::new();

    for (regex, name) in config.output.external_globals.clone() {
      external_config
        .0
        .push(super::external::ExternalConfigItem::Object(
          ExternalObject {
            pattern: ConfigRegex::new(&regex),
            global_name: name,
          },
        ));
    }

    for external in &config.external {
      external_config
        .0
        .push(ExternalConfigItem::Default(external.clone()))
    }

    external_config
  }
}

#[cfg(test)]
mod tests {
  use serde::{Deserialize, Serialize};
  use serde_json::json;

  use super::ExternalConfig;

  #[test]
  fn test() {
    #[derive(Debug, Deserialize, Serialize)]
    struct D {
      external: ExternalConfig,
    }

    let value: D = serde_json::from_str(
      json!({
        "external": ["^node:.+$", { "pattern": "jquery", "globalName": "$" }]
      })
      .to_string()
      .as_str(),
    )
    .unwrap();

    println!("{value:#?}");
  }
}
