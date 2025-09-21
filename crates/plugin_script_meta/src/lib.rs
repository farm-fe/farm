use std::sync::Arc;

use farmfe_core::{config::Config, context::CompilationContext, plugin::Plugin};
/// A set of Plugins that are used to fill module.meta for the script module.
pub use plugin_exports::FarmPluginScriptMetaExports;
pub use plugin_features::FarmPluginScriptMetaFeatures;

/// Each module exports a specific Farm Plugin
mod plugin_exports;
mod plugin_features;

/// Helper function to update async modules
mod find_async_modules;

pub struct FarmPluginScriptMeta {}

impl FarmPluginScriptMeta {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginScriptMeta {
  fn name(&self) -> &str {
    "FarmPluginScriptMeta"
  }

  fn priority(&self) -> i32 {
    -99
  }

  fn module_graph_build_end(
    &self,
    module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let async_modules = find_async_modules::find_async_modules(module_graph);

    for async_module in async_modules {
      let module = module_graph.module_mut(&async_module).unwrap();

      if module.module_type.is_script() {
        module.meta.as_script_mut().is_async = true;
      }
    }

    Ok(Some(()))
  }

  fn module_graph_updated(
    &self,
    param: &farmfe_core::plugin::PluginModuleGraphUpdatedHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    find_async_modules::update_async_modules(param, context);

    Ok(Some(()))
  }
}
