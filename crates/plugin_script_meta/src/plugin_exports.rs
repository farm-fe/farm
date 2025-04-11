use expand_exports::expand_exports_of_module_graph;
use farmfe_core::{config::Config, plugin::Plugin};

mod expand_exports;

/// In optimize_module_graph hook, fill `meta.export_ident_map`.
/// Note that this plugin should be executed after plugin_tree_shake and before plugin_mangle_exports.
pub struct FarmPluginScriptMetaExports {}

impl FarmPluginScriptMetaExports {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginScriptMetaExports {
  fn name(&self) -> &str {
    "FarmPluginScriptMetaExports"
  }

  /// Must be executed after tree shake
  fn optimize_module_graph(
    &self,
    module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    expand_exports_of_module_graph(module_graph, context);

    Ok(Some(()))
  }
}
