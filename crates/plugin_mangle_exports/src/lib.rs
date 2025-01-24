use farmfe_core::{config::Config, plugin::Plugin};

pub struct FarmPluginMangleExports {}

mod ident_generator;

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
    Ok(Some(()))
  }
}
