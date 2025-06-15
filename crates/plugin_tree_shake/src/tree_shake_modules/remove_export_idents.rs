use farmfe_core::{
  module::{
    meta_data::script::{
      statement::{ExportSpecifierInfo, SwcId},
      ModuleExportIdent, ModuleExportIdentType, ScriptModuleMetaData,
    },
    module_graph::ModuleGraph,
    ModuleId,
  },
  HashMap, HashSet,
};
use farmfe_toolkit::script::create_export_default_ident;

fn is_module_contains_export_ident(
  meta: &ScriptModuleMetaData,
  export_ident: &ModuleExportIdent,
) -> bool {
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
          ExportSpecifierInfo::All => unreachable!(),
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

pub fn remove_export_idents(module_graph: &mut ModuleGraph) {
  let mut module_ident_to_remove = HashMap::<ModuleId, HashSet<String>>::default();

  for module in module_graph.modules() {
    if module.module_type.is_script() {
      let meta = module.meta.as_script();

      for (key, export_ident) in &meta.export_ident_map {
        if let Some(defined_module) = module_graph.module(&export_ident.module_id) {
          let defined_module_meta = defined_module.meta.as_script();

          if !is_module_contains_export_ident(defined_module_meta, export_ident) {
            // remove export ident cause the module is being removed
            module_ident_to_remove
              .entry(module.id.clone())
              .or_default()
              .insert(key.clone());
          }
        } else {
          // remove export ident cause the module is being removed
          module_ident_to_remove
            .entry(module.id.clone())
            .or_default()
            .insert(key.clone());
        }
      }
    }
  }

  for (module_id, idents) in module_ident_to_remove {
    let module = module_graph.module_mut(&module_id).unwrap();
    let meta = module.meta.as_script_mut();

    for ident in idents {
      meta.export_ident_map.remove(&ident);
    }
  }
}
