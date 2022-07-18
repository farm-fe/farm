use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "camelCase", default)]
pub struct Config {
  pub input: HashMap<String, String>,
  pub root: String,
  pub mode: Mode,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      input: HashMap::new(),
      root: std::env::current_dir()
        .unwrap()
        .to_string_lossy()
        .to_string(),
      mode: Mode::Development,
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub enum Mode {
  Development,
  Production,
}

impl Default for Mode {
  fn default() -> Self {
    Self::Development
  }
}
