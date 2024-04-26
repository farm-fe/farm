use std::sync::Arc;

use crate::context::CompilationContext;

const CUSTOM_CONFIG_RUNTIME_ISOLATE: &str = "runtime.isolate";

pub fn get_config_runtime_isolate(context: &Arc<CompilationContext>) -> bool {
  if let Some(val) = context.config.custom.get(CUSTOM_CONFIG_RUNTIME_ISOLATE) {
    val == "true"
  } else {
    false
  }
}
