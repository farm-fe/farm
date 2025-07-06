use farmfe_core::{config::Config, plugin::Plugin};

use crate::utils::transform_export_all_to_export_named;

pub struct FarmPluginMangleExports {}

mod ident_generator;
mod mangle_exports;
mod utils;

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

  fn freeze_module_graph_meta(
    &self,
    module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let can_not_be_minified = mangle_exports::find_idents_can_not_be_mangled(module_graph);

    let full_mangled_ident_map = mangle_exports::mangle_exports(&can_not_be_minified, module_graph);
    // collect imports of module_graph for renaming
    let full_imports_to_rename =
      mangle_exports::find_imports_to_rename(&full_mangled_ident_map, module_graph);

    let module_ids = module_graph
      .modules()
      .into_iter()
      .map(|m| m.id.clone())
      .collect::<Vec<_>>();

    // transform export all to export named
    for module_id in module_ids {
      transform_export_all_to_export_named(module_id, module_graph);
    }

    mangle_exports::update_exports_meta_and_module_decl(
      &full_mangled_ident_map,
      &full_imports_to_rename,
      module_graph,
    );

    Ok(Some(()))
  }
}
