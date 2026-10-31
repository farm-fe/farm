use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::Result,
  plugin::{DEFAULT_PRIORITY, Plugin, PluginHandleEntryResourceHookParam},
  resource::ResourceType,
};
use farmfe_macro_plugin::farm_plugin;

/// Keep CSS artifacts and module exports, but do not evaluate stylesheets in Node.
#[farm_plugin]
pub struct FarmPluginSsrCss;

impl FarmPluginSsrCss {
  fn new(_config: &Config, _options: String) -> Self {
    Self
  }
}

impl Plugin for FarmPluginSsrCss {
  fn name(&self) -> &str {
    "FarmPluginSsrCss"
  }

  // The runtime renderer consumes initial_resources at DEFAULT_PRIORITY.
  fn priority(&self) -> i32 {
    DEFAULT_PRIORITY + 1
  }

  fn handle_entry_resource(
    &self,
    param: &mut PluginHandleEntryResourceHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if !context.config.output.target_env.is_node() {
      return Ok(None);
    }
    param
      .initial_resources
      .retain(|(_, kind)| !matches!(kind, ResourceType::Css));
    Ok(Some(()))
  }
}
