use std::collections::HashMap;

pub struct Config {
  pub input: HashMap<String, String>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      input: HashMap::new(),
    }
  }
}
