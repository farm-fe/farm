use farmfe_core::{config::Config, plugin::Plugin};

use features_analyzer::FeaturesAnalyzer;
use statements::analyze_statements;

mod features_analyzer;
mod idents;
mod statements;

/// In finalize_module hook, fill the es module features like `meta.feature_flags`, `meta.module_system`, `meta.hmr_accepted`, etc.
pub struct FarmPluginScriptMetaFeatures {}

impl FarmPluginScriptMetaFeatures {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginScriptMetaFeatures {
  fn name(&self) -> &str {
    "FarmPluginScriptMetaFeatures"
  }

  /// ast should be processed in process_module hook, ast modification after process_module hook
  /// should update meta data manually or executed before internal plugins
  fn finalize_module(
    &self,
    param: &mut farmfe_core::plugin::PluginFinalizeModuleHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !param.module.module_type.is_script() {
      return Ok(None);
    }

    let meta = param.module.meta.as_script_mut();

    // analyze statements
    meta.statements = analyze_statements(meta);

    // unresolved idents and top level idents
    meta.unresolved_idents = idents::analyze_unresolved_idents(&param.module.id, meta, _context);
    meta.top_level_idents = idents::analyze_top_level_idents(meta);

    // is_async
    meta.is_async = meta.statements.iter().any(|s| s.top_level_await);

    // analyze features used
    let features_analyzer = FeaturesAnalyzer::new(&param.deps, &meta.statements);
    meta.feature_flags = features_analyzer.analyze();
    Ok(None)
  }
}
