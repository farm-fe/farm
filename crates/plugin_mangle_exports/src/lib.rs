use farmfe_core::{
  config::Config,
  module::meta_data::script::statement::Statement,
  plugin::{hooks::freeze_module::PluginFreezeModuleHookParam, Plugin},
  swc_common::Mark,
};
use farmfe_toolkit::{
  script::{
    analyze_statement::{analyze_statement_info, AnalyzedStatementInfo},
    concatenate_modules::expand_exports_of_module_graph,
    idents_collector::UnresolvedIdentCollector,
    swc_try_with::try_with,
  },
  swc_ecma_visit::VisitWith,
};
use features_analyzer::FeaturesAnalyzer;

pub struct FarmPluginMangleExports {}

impl FarmPluginMangleExports {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginMangleExports {
  fn name(&self) -> &str {
    "FarmPluginMangleExports"
  }

  /// This plugin should be executed after FarmPluginScriptMeta, as it depends on meta data collected in that plugin
  fn priority(&self) -> i32 {
    -100
  }

  fn optimize_module_graph(
    &self,
    module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    expand_exports_of_module_graph(module_graph, context);

    Ok(Some(()))
  }
}
