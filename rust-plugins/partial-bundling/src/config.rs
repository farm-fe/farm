use farmfe_core::{config::config_regex::ConfigRegex, serde::Deserialize, serde::Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default, crate = "farmfe_core::serde")]
pub struct PartialBundlingModuleBucketsConfig {
  pub name: String,
  /// Regex vec to match the modules in the module bucket
  pub test: Vec<ConfigRegex>,
  pub min_size: Option<usize>,
  pub max_concurrent_requests: Option<u32>,
  pub weight: isize,
}

impl Default for PartialBundlingModuleBucketsConfig {
  fn default() -> Self {
    Self {
      name: "".to_string(),
      test: vec![],
      max_concurrent_requests: None,
      min_size: None,
      weight: 0,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", crate = "farmfe_core::serde")]
pub struct PartialBundlingConfig {
  #[serde(default)]
  pub module_bucket: Vec<PartialBundlingModuleBucketsConfig>,
}
