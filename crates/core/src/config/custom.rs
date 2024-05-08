use std::{collections::HashMap, sync::Arc};

use crate::context::CompilationContext;

use super::{
  config_regex::ConfigRegex,
  external::{ExternalConfig, ExternalObject},
  Config,
};

const CUSTOM_CONFIG_RUNTIME_ISOLATE: &str = "runtime.isolate";
pub const CUSTOM_CONFIG_EXTERNAL_RECORD: &str = "external.record";

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

    let external: HashMap<String, String> = serde_json::from_str(val)
      .unwrap_or_else(|_| panic!("failed parse record external {:?}", val));

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
