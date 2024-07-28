use serde::{Deserialize, Serialize};

use super::config_regex::ConfigRegex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PartialBundlingConfig {
  /// target concurrent requests for every resource loading
  pub target_concurrent_requests: usize,
  /// target min size for every resource loading
  pub target_min_size: usize,
  /// target max size for every resource loading
  pub target_max_size: usize,
  /// A group of modules that should be placed together.
  /// Note that this group config is only a hit to the compiler that these modules should be placed together,
  /// it may produce multiple resources, if you want to enforce modules in only one resource, you should use `enforceResources`.
  pub groups: Vec<PartialBundlingGroupConfig>,
  /// Array to match the modules that should always be in the same output resource, ignore all other constraints.
  pub enforce_resources: Vec<PartialBundlingEnforceResourceConfig>,
  /// enforce target concurrent requests for every resource loading,
  /// when tue, smaller resource will be merged into bigger resource to meet the target concurrent requests
  /// this may cause issue for css resource, be careful to use this option
  pub enforce_target_concurrent_requests: bool,
  /// enforce target min size for every resource loading,
  /// when tue, smaller resource will be merged into bigger resource to meet the target min size
  pub enforce_target_min_size: bool,
  /// immutable module paths, set to empty array to disable
  pub immutable_modules: Vec<ConfigRegex>,
  /// Default to 0.8, immutable module will have 80% request numbers.
  /// TODO check if it is between 0 and 1
  pub immutable_modules_weight: f32,
}

impl Default for PartialBundlingConfig {
  fn default() -> Self {
    Self {
      target_concurrent_requests: 25,
      // 100KB before minimize and gzip
      target_min_size: 1024 * 100,
      // 1.5 MB before minimize and gzip
      target_max_size: 1024 * 1500,
      groups: vec![],
      enforce_resources: vec![],
      enforce_target_concurrent_requests: false,
      enforce_target_min_size: false,
      immutable_modules: vec![ConfigRegex::default()],
      immutable_modules_weight: 0.8,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PartialBundlingGroupConfig {
  pub name: String,
  /// Regex vec to match the modules in the module bucket
  pub test: Vec<ConfigRegex>,
  /// all, mutable or immutable
  pub group_type: PartialBundlingGroupConfigGroupType,
  /// all, initial, async
  pub resource_type: PartialBundlingGroupConfigResourceType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PartialBundlingGroupConfigGroupType {
  #[serde(rename = "all")]
  All,
  #[serde(rename = "mutable")]
  Mutable,
  #[serde(rename = "immutable")]
  Immutable,
}

impl PartialBundlingGroupConfigGroupType {
  pub fn is_match(&self, immutable: bool) -> bool {
    match self {
      Self::All => true,
      Self::Mutable => !immutable,
      Self::Immutable => immutable,
    }
  }
}

impl Default for PartialBundlingGroupConfigGroupType {
  fn default() -> Self {
    Self::All
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PartialBundlingGroupConfigResourceType {
  #[serde(rename = "all")]
  All,
  #[serde(rename = "initial")]
  Initial,
  #[serde(rename = "async")]
  Async,
}

impl Default for PartialBundlingGroupConfigResourceType {
  fn default() -> Self {
    Self::All
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PartialBundlingEnforceResourceConfig {
  pub name: String,
  /// Regex vec to match the modules in the module bucket
  pub test: Vec<ConfigRegex>,
}
