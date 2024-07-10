use serde::{Deserialize, Serialize};

use super::config_regex::ConfigRegex;

// TODO: implement this for treeShaking
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeShakingConfig {
  /// exclude some side effects files
  side_effects: Vec<ConfigRegex>,
}

impl TreeShakingConfig {
  pub fn is_match(&self, source: &str) -> bool {
    self.side_effects.iter().any(|i| i.is_match(source))
  }
}
