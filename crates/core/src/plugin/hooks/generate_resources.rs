use farmfe_macro_cache_item::cache_item;

use crate::resource::Resource;

#[cache_item]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedResource {
  pub resource: Resource,
  pub source_map: Option<Resource>,
}

#[cache_item]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginGenerateResourcesHookResult {
  /// A resource pot can generate multiple resources, for example: generating cjs/esm at the same time
  pub resources: Vec<GeneratedResource>,
}
