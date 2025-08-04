use farmfe_core::{
  module::{
    meta_data::script::{
      statement::{ExportSpecifierInfo, SwcId},
      ModuleExportIdent, ModuleExportIdentType, ModuleReExportIdentType, ScriptModuleMetaData,
    },
    module_graph::ModuleGraph,
    ModuleId,
  },
  plugin::ResolveKind,
  HashMap, HashSet,
};
use farmfe_toolkit::script::create_export_default_ident;

fn is_module_contains_export_ident(
  meta: &ScriptModuleMetaData,
  export_ident: &ModuleExportIdent,
) -> bool {
  let export_ident = export_ident.as_internal();

  if !matches!(export_ident.export_type, ModuleExportIdentType::Declaration) {
    return true;
  }

  meta.statements.iter().any(|stmt| {
    if let Some(export_info) = &stmt.export_info {
      for sp in &export_info.specifiers {
        let is_ident_used = match sp {
          ExportSpecifierInfo::Default => {
            if stmt.defined_idents.contains(&export_ident.ident) {
              return true;
            }

            let default_ident: SwcId = create_export_default_ident(&export_ident.module_id)
              .to_id()
              .into();

            default_ident == export_ident.ident
          }
          ExportSpecifierInfo::All => false, // it's defined in other modules
          ExportSpecifierInfo::Named { local, .. } => {
            // is source is not none, means the ident is defined in other modules, we should always return false
            export_info.source.is_none() && local == &export_ident.ident
          }
          ExportSpecifierInfo::Namespace(swc_id) => swc_id == &export_ident.ident,
        };

        if is_ident_used {
          return true;
        }
      }
    }

    false
  })
}

fn is_module_contains_reexport_ident(
  module_id: &ModuleId,
  meta: &ScriptModuleMetaData,
  export_str: &str,
  reexport_ident_type: &ModuleReExportIdentType,
  module_graph: &ModuleGraph,
) -> bool {
  meta.statements.iter().any(|stmt| {
    if let Some(export_info) = &stmt.export_info
      && let Some(source) = export_info.source.as_ref()
    {
      let source_module_id =
        module_graph.get_dep_by_source(module_id, source, Some(ResolveKind::ExportFrom));

      for sp in &export_info.specifiers {
        let is_reexport_used = match sp {
          ExportSpecifierInfo::Named { local, exported } => {
            let local_exported_str = exported.as_ref().unwrap_or(local).sym.as_str();

            if let ModuleReExportIdentType::FromExportNamed {
              from_module_id,
              local: local_local,
            } = reexport_ident_type
            {
              source_module_id == *from_module_id
                && local_exported_str == export_str
                && *local_local == local.sym
            } else {
              false
            }
          }
          ExportSpecifierInfo::All => {
            if let ModuleReExportIdentType::FromExportAll(from_module_id) = reexport_ident_type {
              source_module_id == *from_module_id
            } else {
              false
            }
          }
          ExportSpecifierInfo::Default | ExportSpecifierInfo::Namespace(_) => {
            // default export and namespace export don't reexport idents
            false
          }
        };

        if is_reexport_used {
          return true;
        }
      }
    }

    false
  })
}

pub fn remove_export_idents(module_graph: &mut ModuleGraph) {
  let mut module_ident_to_remove = HashMap::<ModuleId, HashSet<String>>::default();

  for module in module_graph.modules() {
    if module.module_type.is_script() {
      let meta = module.meta.as_script();

      for (export_str, export_ident) in &meta.export_ident_map {
        if let Some(defined_module) = module_graph.module(&export_ident.as_internal().module_id) {
          if !defined_module.module_type.is_script() || defined_module.external {
            continue;
          }

          let defined_module_meta = defined_module.meta.as_script();

          if !is_module_contains_export_ident(defined_module_meta, export_ident) {
            // remove export ident cause the module is being removed
            module_ident_to_remove
              .entry(module.id.clone())
              .or_default()
              .insert(export_str.clone());
          }
        } else {
          // remove export ident cause the module is being removed
          module_ident_to_remove
            .entry(module.id.clone())
            .or_default()
            .insert(export_str.clone());
        }
      }

      // for reexport ident, makes sure the reexport ident is not removed
      for (export_str, reexport_ident) in &meta.reexport_ident_map {
        if !is_module_contains_reexport_ident(
          &module.id,
          meta,
          export_str,
          reexport_ident,
          module_graph,
        ) {
          module_ident_to_remove
            .entry(module.id.clone())
            .or_default()
            .insert(export_str.clone());
        }
      }
    }
  }

  for (module_id, idents) in module_ident_to_remove {
    let module = module_graph.module_mut(&module_id).unwrap();
    let meta = module.meta.as_script_mut();

    for ident in idents {
      meta.export_ident_map.remove(&ident);
      meta.reexport_ident_map.remove(&ident);
      meta.ambiguous_export_ident_map.remove(&ident);
    }
  }
}
