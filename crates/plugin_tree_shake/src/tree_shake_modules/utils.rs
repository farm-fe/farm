use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::ResolveKind,
  swc_ecma_ast::Id,
};

use crate::{
  module::{TreeShakeModule, UsedExportsIdent},
  statement_graph::{ExportInfo, ExportSpecifierInfo, ImportInfo, ImportSpecifierInfo},
};

// Add all imported to used_exports
pub fn add_used_exports_by_import_info(
  tree_shake_modules_map: &mut std::collections::HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
  tree_shake_module_id: &ModuleId,
  import_info: ImportInfo,
) {
  let imported_module_id = module_graph.get_dep_by_source(
    tree_shake_module_id,
    &import_info.source,
    Some(ResolveKind::Import),
  );
  let imported_module = module_graph.module(&imported_module_id).unwrap();

  if imported_module.external || !imported_module.module_type.is_script() {
    return;
  }

  let imported_tree_shake_module = tree_shake_modules_map
    .get_mut(&imported_module_id)
    .unwrap_or_else(|| {
      panic!("imported module not found: {:?}", imported_module_id);
    });

  if import_info.specifiers.is_empty() {
    return;
  }

  for sp in import_info.specifiers {
    match sp {
      ImportSpecifierInfo::Namespace(_) => imported_tree_shake_module
        .pending_used_exports
        .set_export_all(),
      ImportSpecifierInfo::Named { local, imported } => {
        if let Some(ident) = imported {
          if ident.0.as_str() == "default" {
            imported_tree_shake_module
              .pending_used_exports
              .add_used_export(UsedExportsIdent::Default);
          } else {
            imported_tree_shake_module
              .pending_used_exports
              .add_used_export(UsedExportsIdent::SwcIdent(strip_context(&ident)));
          }
        } else {
          imported_tree_shake_module
            .pending_used_exports
            .add_used_export(UsedExportsIdent::SwcIdent(strip_context(&local)));
        }
      }
      ImportSpecifierInfo::Default(_) => {
        imported_tree_shake_module
          .pending_used_exports
          .add_used_export(UsedExportsIdent::Default);
      }
    }
  }
}

/// All all exported to used_exports
pub fn add_used_exports_by_export_info(
  tree_shake_modules_map: &mut std::collections::HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
  tree_shake_module_id: &ModuleId,
  export_info: ExportInfo,
) {
  if let Some(source) = &export_info.source {
    let exported_module_id =
      module_graph.get_dep_by_source(tree_shake_module_id, source, Some(ResolveKind::ExportFrom));
    let exported_module = module_graph.module(&exported_module_id).unwrap();

    if exported_module.external {
      return;
    }

    let exported_tree_shake_module = tree_shake_modules_map.get_mut(&exported_module_id).unwrap();

    for sp in export_info.specifiers {
      match sp {
        ExportSpecifierInfo::Namespace(_) => exported_tree_shake_module
          .pending_used_exports
          .set_export_all(),
        ExportSpecifierInfo::Named { local, .. } => {
          if local.0.as_str() == "default" {
            exported_tree_shake_module
              .pending_used_exports
              .add_used_export(UsedExportsIdent::Default);
          } else {
            exported_tree_shake_module
              .pending_used_exports
              .add_used_export(UsedExportsIdent::SwcIdent(strip_context(&local)));
          }
        }
        ExportSpecifierInfo::Default => {
          exported_tree_shake_module
            .pending_used_exports
            .add_used_export(UsedExportsIdent::Default);
        }
        ExportSpecifierInfo::All => {
          exported_tree_shake_module
            .pending_used_exports
            .add_used_export(UsedExportsIdent::ExportAll);
        }
      }
    }
  }
}

pub fn strip_context(ident: &Id) -> String {
  ident.0.to_string()
}
