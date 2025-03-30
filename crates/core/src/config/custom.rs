use std::sync::Arc;

use serde::de::DeserializeOwned;

use crate::context::CompilationContext;
use crate::HashMap;

use super::{
  asset::AssetFormatMode,
  config_regex::ConfigRegex,
  css::NameConversion,
  external::{ExternalConfig, ExternalObject},
  Config,
};

const CUSTOM_CONFIG_RUNTIME_ISOLATE: &str = "runtime.isolate";
pub const CUSTOM_CONFIG_EXTERNAL_RECORD: &str = "external.record";
pub const CUSTOM_CONFIG_RESOLVE_DEDUPE: &str = "resolve.dedupe";
pub const CUSTOM_CONFIG_CSS_MODULES_LOCAL_CONVERSION: &str = "css.modules.locals_conversion";
pub const CUSTOM_CONFIG_ASSETS_MODE: &str = "assets.mode";

pub fn get_config_runtime_isolate(context: &Arc<CompilationContext>) -> bool {
  if let Some(val) = context.config.custom.get(CUSTOM_CONFIG_RUNTIME_ISOLATE) {
    val == "true"
  } else {
    false
  }
}

pub fn get_config_external_record(config: &Config) -> ExternalConfig {
  if let Some(val) = config.custom.get(CUSTOM_CONFIG_EXTERNAL_RECORD) {
    if val.is_empty() {
      return ExternalConfig::new();
    }

    let external: HashMap<String, String> =
      serde_json::from_str(val).unwrap_or_else(|_| panic!("failed parse record external {val:?}"));

    let mut external_config = ExternalConfig::new();

    for (regex, name) in external {
      external_config
        .0
        .push(super::external::ExternalConfigItem::Object(
          ExternalObject {
            pattern: ConfigRegex::new(&regex),
            global_name: name,
          },
        ));
    }
    external_config
  } else {
    ExternalConfig::new()
  }
}

pub fn get_config_resolve_dedupe(config: &Config) -> Vec<String> {
  get_field_or_default_from_custom(config, CUSTOM_CONFIG_RESOLVE_DEDUPE)
}

pub fn get_config_css_modules_local_conversion(config: &Config) -> NameConversion {
  get_field_or_default_from_custom(config, CUSTOM_CONFIG_CSS_MODULES_LOCAL_CONVERSION)
}

pub fn get_config_assets_mode(config: &Config) -> Option<AssetFormatMode> {
  get_field_or_default_from_custom(config, CUSTOM_CONFIG_ASSETS_MODE)
}

fn get_field_or_default_from_custom<T: Default + DeserializeOwned>(
  config: &Config,
  field: &str,
) -> T {
  config
    .custom
    .get(field)
    .map(|val| serde_json::from_str(val).unwrap_or_default())
    .unwrap_or_default()
}
