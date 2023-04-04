use farmfe_core::swc_ecma_ast::Module as SwcModule;

use crate::{
  module::TreeShakeModule,
  statement_graph::{ExportInfo, ImportInfo},
};

pub fn remove_useless_stmts(
  tree_shake_module: &mut TreeShakeModule,
  swc_module: &mut SwcModule,
) -> (Vec<ImportInfo>, Vec<ExportInfo>) {
  // analyze the statement graph start from the used statements
  let used_stmts = tree_shake_module.used_statements().clone();

  // TODO remove unused specifiers in export statement

  // remove the unused statements from the module
  let mut stmts_to_remove = swc_module
    .body
    .iter()
    .enumerate()
    .filter_map(|(index, _)| {
      if !used_stmts.contains(&index) {
        Some(index)
      } else {
        None
      }
    })
    .collect::<Vec<_>>();
  // remove from the end to the start
  stmts_to_remove.reverse();

  for stmt in stmts_to_remove {
    swc_module.body.remove(stmt);
  }

  // get used indents from the used statements
  let used_imports = tree_shake_module
    .imports()
    .into_iter()
    .filter_map(|import_info| {
      // if the import statement is used(contains at least one used specifier)
      if used_stmts.contains(&import_info.stmt_id) && !import_info.specifiers.is_empty() {
        Some(import_info)
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  let used_exports = tree_shake_module
    .exports()
    .into_iter()
    .filter_map(|export_info| {
      // if the export statement is used(contains at least one used specifier)
      if export_info.source.is_some()
        && used_stmts.contains(&export_info.stmt_id)
        && !export_info.specifiers.is_empty()
      {
        Some(export_info)
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  (used_imports, used_exports)
}
