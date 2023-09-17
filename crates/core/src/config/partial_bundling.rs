use serde::{Deserialize, Serialize};

use super::config_regex::ConfigRegex;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PartialBundlingConfig {
  // target concurrent requests for every resource loading
  pub target_concurrent_requests: usize,
  // target min size for every resource loading
  pub target_min_size: usize,
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
}

impl Default for PartialBundlingConfig {
  fn default() -> Self {
    Self {
      target_concurrent_requests: 25,
      // 20KB
      target_min_size: 1024 * 20,
      groups: vec![],
      enforce_resources: vec![],
      enforce_target_concurrent_requests: false,
      enforce_target_min_size: false,
      immutable_modules: vec![ConfigRegex::default()],
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PartialBundlingGroupConfig {
  pub name: String,
  /// Regex vec to match the modules in the module bucket
  pub test: Vec<ConfigRegex>,
  /// mutable or immutable
  pub group_type: String,
  /// all, initial, async
  pub resource_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PartialBundlingEnforceResourceConfig {
  pub name: String,
  /// Regex vec to match the modules in the module bucket
  pub test: Vec<ConfigRegex>,
}
