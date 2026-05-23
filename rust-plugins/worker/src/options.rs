use farmfe_core::config::Config;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
  pub compiler_config: Option<Config>,
}
