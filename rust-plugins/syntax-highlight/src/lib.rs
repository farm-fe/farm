use farmfe_core::{
  config::Config,
  error::{CompilationError, Result},
  plugin::{DEFAULT_PRIORITY, Plugin},
  serde_json,
};
use farmfe_macro_plugin::farm_plugin;
use farmfe_syntax_highlight::{CONFIG_KEY, HighlightOptions};

#[farm_plugin]
pub struct FarmPluginSyntaxHighlight {
  options: std::result::Result<HighlightOptions, String>,
}
impl FarmPluginSyntaxHighlight {
  fn new(_config: &Config, options: String) -> Self {
    Self {
      options: serde_json::from_str(&options).map_err(|e| e.to_string()),
    }
  }
}
impl Plugin for FarmPluginSyntaxHighlight {
  fn name(&self) -> &str {
    "FarmPluginSyntaxHighlight"
  }
  fn priority(&self) -> i32 {
    DEFAULT_PRIORITY + 20
  }
  fn config(&self, config: &mut Config) -> Result<Option<()>> {
    let options = self
      .options
      .as_ref()
      .map_err(|e| CompilationError::GenericError(format!("[DOCS_HIGHLIGHT_OPTIONS] {e}")))?;
    config.custom.insert(
      CONFIG_KEY.into(),
      serde_json::to_string(options).map_err(|e| CompilationError::GenericError(e.to_string()))?,
    );
    Ok(Some(()))
  }
}
