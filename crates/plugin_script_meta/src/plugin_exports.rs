use std::i32;

use expand_exports::{expand_exports_of_module_graph, get_basic_module_export_ident};
use farmfe_core::{
  config::Config,
  module::{
    meta_data::script::{ScriptModuleMetaData, EXPORT_DEFAULT},
    module_graph::ModuleGraph,
    ModuleId,
  },
  plugin::{Plugin, ResolveKind},
};

mod expand_exports;

/// In optimize_module_graph hook, fill `meta.export_ident_map`.
/// Note that this plugin should be executed after plugin_tree_shake and before plugin_mangle_exports.
pub struct FarmPluginScriptMetaExports {
  should_expand_exports: bool,
}

impl FarmPluginScriptMetaExports {
  pub fn new(config: &Config) -> Self {
    Self {
      should_expand_exports: Self::should_expand_exports(config),
    }
  }

  // For performance, Only enabled exports analysis when one of following conditions are met:
  // 1. Library bundle is enabled
  // 2. concatenateModules is enabled
  // 3. mangleExports is enabled
  // 4. tree shake is enabled
  pub fn should_expand_exports(config: &Config) -> bool {
    let library_enabled = config.output.target_env.is_library();
    let concatenate_modules_enabled = config.concatenate_modules;

    let mangle_exports_enabled = config
      .minify
      .as_obj()
      .map(|obj| obj.mangle_exports)
      .unwrap_or(true);

    let tree_shake_enabled = config.tree_shaking.enabled();

    library_enabled || concatenate_modules_enabled || mangle_exports_enabled || tree_shake_enabled
  }

  pub fn is_export_ident_changed(
    &self,
    module_id: &ModuleId,
    module_script_meta: &ScriptModuleMetaData,
  ) -> bool {
    let mut result = false;

    let module_export_idents = get_basic_module_export_ident(module_id, module_script_meta, true);

    for (export_str, export_ident) in module_export_idents {
      if result {
        continue;
      }

      if !module_script_meta
        .export_ident_map
        .contains_key(&export_str)
      {
        result = true;
      } else if let Some(ident) = module_script_meta.export_ident_map.get(&export_str) {
        if export_ident != *ident {
          result = true;
        }
      }
    }

    result
  }

  pub fn is_import_ident_changed(
    &self,
    module_id: &ModuleId,
    module_script_meta: &ScriptModuleMetaData,
    module_graph: &ModuleGraph,
  ) -> bool {
    let mut result = false;

    for statement in &module_script_meta.statements {
      if let Some(import_info) = &statement.import_info {
        let dep_module_id = module_graph.get_dep_by_source_optional(
          module_id,
          &import_info.source,
          Some(ResolveKind::Import),
        );

        if dep_module_id.is_none() {
          continue;
        }

        let dep_module_id = dep_module_id.unwrap();
        let dep_module = module_graph.module(&dep_module_id).unwrap();

        if !dep_module.module_type.is_script() {
          continue;
        }

        let dep_module_script_meta = dep_module.meta.as_script();

        for specifier in &import_info.specifiers {
          match specifier {
            farmfe_core::module::meta_data::script::statement::ImportSpecifierInfo::Namespace(
              ..,
            ) => { /* do nothing */ }
            farmfe_core::module::meta_data::script::statement::ImportSpecifierInfo::Named {
              local,
              imported,
            } => {
              let imported_str = imported
                .as_ref()
                .map(|i| i.sym.to_string())
                .unwrap_or(local.sym.to_string());
              let ident = dep_module_script_meta.export_ident_map.get(&imported_str);
              let ambiguous_idents = dep_module_script_meta
                .ambiguous_export_ident_map
                .get(&imported_str);

              if ident.is_none() && ambiguous_idents.is_none() {
                result = true;
              }
            }
            farmfe_core::module::meta_data::script::statement::ImportSpecifierInfo::Default(..) => {
              let ident = dep_module_script_meta.export_ident_map.get(EXPORT_DEFAULT);
              let ambiguous_idents = dep_module_script_meta
                .ambiguous_export_ident_map
                .get(EXPORT_DEFAULT);

              if ident.is_none() && ambiguous_idents.is_none() {
                result = true;
              }
            }
          }
        }
      }
    }

    result
  }
}

impl Plugin for FarmPluginScriptMetaExports {
  fn name(&self) -> &str {
    "FarmPluginScriptMetaExports"
  }

  /// This plugin should be executed first before all other plugins for freeze_module_graph_meta hook
  fn priority(&self) -> i32 {
    i32::MAX
  }

  fn optimize_module_graph(
    &self,
    module_graph: &mut ModuleGraph,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !self.should_expand_exports {
      return Ok(None);
    }

    expand_exports_of_module_graph(module_graph, context);

    Ok(Some(()))
  }

  fn module_graph_updated(
    &self,
    param: &farmfe_core::plugin::PluginModuleGraphUpdatedHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !self.should_expand_exports {
      return Ok(None);
    }

    // update exports of the affected modules
    // 1. determine whether the exports of the module is changed,
    // 2. if the exports of the module is changed, do full expand of the module graph
    let mut should_do_full_update = false;

    // if there are dependency changes, we should always do full update. Because there may be new export * from are added or removed.
    // We may optimize this in the future, support incremental update
    if param.deps_changes.len() > 0 {
      should_do_full_update = true;
    }

    let mut module_graph = context.module_graph.write();
    // if there any import/export ident changed, we should do full update
    if !should_do_full_update {
      for update_module_id in &param.updated_modules_ids {
        let module = module_graph.module(update_module_id).unwrap();

        if !module.module_type.is_script() {
          continue;
        }

        let module_script_meta = module.meta.as_script();
        if self.is_export_ident_changed(&module.id, module_script_meta)
          || self.is_import_ident_changed(&module.id, module_script_meta, &module_graph)
        {
          should_do_full_update = true;
        }
      }
    }

    if should_do_full_update {
      expand_exports_of_module_graph(&mut module_graph, context);
    }

    Ok(None)
  }
}
