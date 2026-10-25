#![allow(clippy::mutable_key_type)]

use farmfe_core::{config::Config, plugin::Plugin};

use crate::utils::transform_export_all_to_export_named;

pub struct FarmPluginMangleExports {}

mod ident_generator;
mod mangle_exports;
mod utils;

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::{
    config::Mode, context::CompilationContext,
    module::{Module, ModuleMetaData, ModuleSystem, ModuleType, module_graph::ModuleGraph,
      meta_data::script::{ModuleExportIdent, ModuleExportIdentType, statement::SwcId}},
  };
  use std::sync::Arc;

  #[test]
  fn star_reexport_metadata_tracks_independent_mangled_names() {
    use farmfe_core::{HashMap, module::meta_data::script::ModuleReExportIdentType};
    let mut graph = ModuleGraph::new();
    let mut barrel = Module::new("barrel.js".into());
    barrel.module_type = ModuleType::Js;
    *barrel.meta = ModuleMetaData::Script(Default::default());
    barrel.meta.as_script_mut().reexport_ident_map.insert("Model".into(),
      ModuleReExportIdentType::FromExportAll("source.js".into()));
    graph.add_module(barrel);
    let mappings = HashMap::from_iter([
      ("source.js".into(), HashMap::from_iter([("Model".into(), "a".into())])),
      ("barrel.js".into(), HashMap::from_iter([("Model".into(), "b".into())])),
    ]);
    mangle_exports::update_exports_meta_and_module_decl(&mappings, &HashMap::default(), &mut graph);
    let meta = graph.module(&"barrel.js".into()).unwrap().meta.as_script();
    assert!(matches!(meta.reexport_ident_map.get("b"),
      Some(ModuleReExportIdentType::FromExportNamed { local, .. }) if local == "a"));
  }

  #[test]
  fn named_reexport_metadata_tracks_mangled_source_export() {
    use farmfe_core::{HashMap, module::meta_data::script::ModuleReExportIdentType};
    let source_id = "source.js".into();
    let barrel_id = "barrel.js".into();
    let mut graph = ModuleGraph::new();
    let mut barrel = Module::new(barrel_id);
    barrel.module_type = ModuleType::Js;
    *barrel.meta = ModuleMetaData::Script(Default::default());
    barrel.meta.as_script_mut().reexport_ident_map.insert(
      "Model".into(),
      ModuleReExportIdentType::FromExportNamed { local: "default".into(), from_module_id: source_id },
    );
    graph.add_module(barrel);
    let mappings = HashMap::from_iter([
      ("source.js".into(), HashMap::from_iter([("default".into(), "a".into())])),
      ("barrel.js".into(), HashMap::from_iter([("Model".into(), "b".into())])),
    ]);
    mangle_exports::update_exports_meta_and_module_decl(&mappings, &HashMap::default(), &mut graph);
    let meta = graph.module(&"barrel.js".into()).unwrap().meta.as_script();
    assert!(matches!(meta.reexport_ident_map.get("b"),
      Some(ModuleReExportIdentType::FromExportNamed { local, .. }) if local == "a"));
  }

  #[test]
  fn development_keeps_export_names_for_incremental_imports() {
    let config = Config { mode: Mode::Development, ..Default::default() };
    let plugin = FarmPluginMangleExports::new(&config);
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let mut graph = ModuleGraph::new();
    let mut module = Module::new("ui.js".into());
    module.module_type = ModuleType::Js;
    *module.meta = ModuleMetaData::Script(Default::default());
    module.meta.as_script_mut().module_system = ModuleSystem::EsModule;
    let id = module.id.clone();
    module.meta.as_script_mut().export_ident_map.insert("PageTitle".to_string(),
      ModuleExportIdent::new(id.clone(), SwcId::new("PageTitle".into(), 0), ModuleExportIdentType::Declaration));
    graph.add_module(module);
    plugin.freeze_module_graph_meta(&mut graph, &context).unwrap();
    assert!(graph.module(&id).unwrap().meta.as_script().export_ident_map.contains_key("PageTitle"));
  }
}

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
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // Export mangling rewrites shared modules across the whole graph. HMR
    // recompiles only changed sources, whose imports still use original names.
    // Preserve the source-level export contract in development.
    if matches!(context.config.mode, farmfe_core::config::Mode::Development) {
      return Ok(None);
    }
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
