use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommentsConfig {
  /// true: preserve all comments. false: remove all comments
  Bool(bool),
  /// Only preserve license comments
  License,
}

impl CommentsConfig {
  pub fn enabled(&self) -> bool {
    match self {
      CommentsConfig::Bool(b) => *b,
      CommentsConfig::License => true,
    }
  }
}

impl Default for CommentsConfig {
  fn default() -> Self {
    CommentsConfig::License
  }
}
