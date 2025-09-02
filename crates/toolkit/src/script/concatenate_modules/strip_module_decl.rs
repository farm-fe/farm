use farmfe_core::{
  module::{
    meta_data::script::{
      statement::{ExportSpecifierInfo, ImportSpecifierInfo, StatementId, SwcId},
      CommentsMetaData, ModuleExportIdentType, ModuleReExportIdentType, ScriptModuleMetaData,
      AMBIGUOUS_EXPORT_ALL,
    },
    module_graph::ModuleGraph,
    Module, ModuleId,
  },
  swc_ecma_ast::{
    ClassDecl, Decl, Expr, FnDecl, Ident, Module as SwcModule, ModuleDecl, ModuleItem, Stmt,
  },
  HashMap, HashSet,
};

use crate::script::{
  analyze_statement::analyze_statement_info,
  concatenate_modules::{
    handle_external_modules::{
      add_ambiguous_ident_decl, handle_ambiguous_export_all, HandleAmbiguousExportAllOptions,
      HandleExternalModuleOptions,
    },
    utils::{create_import_specifiers, create_import_stmt},
  },
};

use super::{
  handle_external_modules::handle_external_module,
  unique_idents::{TopLevelIdentsRenameHandler, EXPORT_DEFAULT, EXPORT_NAMESPACE},
  utils::{create_export_default_expr_item, create_export_default_ident, replace_module_decl},
  StripModuleContext,
};

#[derive(Debug)]
pub enum PreservedImportDeclType {
  /// `import * as external_namespace_farm_internal_ from 'module';` generated when handling external module
  ExternalGenerated,
  /// `import { m as bar } from 'module';`. Original import statement when handling external module
  ExternalOriginal,
}

#[derive(Debug)]
pub struct PreservedImportDeclItem {
  pub import_item: ModuleItem,
  pub source_module_id: ModuleId,
  pub preserved_type: PreservedImportDeclType,

  pub namespace_ident: Option<Ident>,
  pub is_namespace_import: bool,
}

/// Result after calling `strip_module_decl`
pub struct StripModuleDeclResult {
  /// the ast that removed the import/export statements
  pub ast: SwcModule,
  pub comments: CommentsMetaData,
  pub items_to_prepend: Vec<ModuleItem>,
  pub items_to_append: Vec<ModuleItem>,
}

pub fn strip_module_decl(
  module_id: &ModuleId,
  module_ids: &HashSet<ModuleId>,
  is_entry_module: bool,
  module_graph: &ModuleGraph,
  strip_context: &mut StripModuleContext,
) -> StripModuleDeclResult {
  let module = module_graph.module(module_id).unwrap();
  let script_meta = module.meta.as_script();

  let comments = script_meta.comments.clone();

  let mut result = StripModuleDeclResult {
    ast: script_meta.ast.clone(),
    comments,
    items_to_prepend: vec![],
    items_to_append: vec![],
  };

  let mut statements_to_remove = vec![];
  let mut params = StripModuleDeclStatementParams {
    module_id,
    module_ids,
    script_meta,
    result: &mut result,
    strip_context,
    is_entry_module,
    module_graph,
  };

  // strip all export statements
  statements_to_remove.extend(strip_export_statements(&mut params));
  // strip all import statements
  statements_to_remove.extend(strip_import_statements(&mut params));

  // remove the import statements in reverse order to avoid index shifting
  statements_to_remove.sort();
  statements_to_remove.reverse();

  for statement_id in statements_to_remove {
    result.ast.body.remove(statement_id);
  }

  result
}

pub struct StripModuleDeclStatementParams<'a> {
  pub module_id: &'a ModuleId,
  pub module_ids: &'a HashSet<ModuleId>,
  pub script_meta: &'a ScriptModuleMetaData,
  pub result: &'a mut StripModuleDeclResult,
  pub strip_context: &'a mut StripModuleContext,
  pub is_entry_module: bool,
  pub module_graph: &'a ModuleGraph,
}

fn strip_import_statements(params: &mut StripModuleDeclStatementParams) -> Vec<StatementId> {
  let mut statements_to_remove = vec![];

  for statement in &params.script_meta.statements {
    if let Some(import_info) = &statement.import_info {
      // always remove import statement
      statements_to_remove.push(statement.id);

      let source_module_id =
        params
          .module_graph
          .get_dep_by_source(params.module_id, &import_info.source, None);
      let source_module = params.module_graph.module(&source_module_id).unwrap();

      // if the source module is external, we should keep the import statement
      if is_module_external(source_module, params.module_ids) {
        handle_external_module(HandleExternalModuleOptions::from(
          params,
          &source_module_id,
          statement,
        ));
        continue;
      }

      let StripModuleDeclStatementParams {
        module_id,
        strip_context,
        module_ids,
        module_graph,
        ..
      } = params;

      let mut external_module_idents_map = HashMap::default();

      for specifier in &import_info.specifiers {
        let (ident, export_str) = match specifier {
          // import { foo, bar as baz } from './module'; // in './module': export { m as bar }
          // =>
          // 1. rename foo#1 to foo#2, foo#2 means the foo ident defined in source module
          // 2. rename baz#1 to m#1, m#1 is the ident defined in source module
          ImportSpecifierInfo::Named { local, imported } => {
            let export_str = if let Some(imported) = imported {
              // imported.sym may be "default", the same as EXPORT_DEFAULT, we don't need to handle it specially
              imported.sym.as_str()
            } else {
              local.sym.as_str()
            };

            (local, export_str)
          }

          // import foo from './module';
          // =>
          // rename foo#1 = module_default#1;
          ImportSpecifierInfo::Default(ident) => (ident, EXPORT_DEFAULT),

          // import * as ns from './module';
          // =>
          // rename ns#1 to module_ns#1;
          ImportSpecifierInfo::Namespace(ident) => (ident, EXPORT_NAMESPACE),
        };

        let mut rename_handler = strip_context.rename_handler.borrow_mut();

        // if the ident is not renamed and the ident is defined in a external module
        if rename_handler.get_renamed_ident(module_id, ident).is_none()
          && let Some((source_module_id, export_str)) = is_ident_reexported_from_external_module(
            module_ids,
            &source_module_id,
            export_str,
            module_graph,
            &rename_handler,
            &mut HashSet::default(),
          )
        {
          external_module_idents_map
            .entry(source_module_id)
            .or_insert(vec![])
            .push((ident.clone(), export_str));
          continue;
        }

        let source_module_script_meta = source_module.meta.as_script();

        // true means the ident is an unresolved ident
        if !rename_imported_ident(
          module_id,
          ident,
          export_str,
          source_module_script_meta,
          &mut rename_handler,
        ) && let Some(module_export_ident) =
          source_module_script_meta.export_ident_map.get(export_str)
        {
          strip_context
            .cyclic_idents
            .entry(module_id.clone())
            .or_default()
            .insert((Some(ident.clone()), module_export_ident.clone()));
        } else {
          let export_ident = source_module_script_meta
            .export_ident_map
            .get(export_str)
            .unwrap();

          if strip_context
            .cyclic_idents
            .get(&source_module_id)
            .map(|idents| idents.contains(&(None, export_ident.clone())))
            .unwrap_or(false)
          {
            // the import is a cyclic idents
            strip_context
              .cyclic_idents
              .entry(module_id.clone())
              .or_default()
              .insert((Some(ident.clone()), export_ident.clone()));
          }
        }
      }

      // handle external module idents
      for (source_module_id, idents) in external_module_idents_map {
        statements_to_remove.extend(handle_external_module_idents(
          params,
          &source_module_id,
          idents,
        ));
      }
    }
  }

  statements_to_remove
}

fn strip_export_statements(params: &mut StripModuleDeclStatementParams) -> Vec<StatementId> {
  let mut statements_to_remove = vec![];
  let mut ambiguous_export_all_idents = vec![];

  for statement in &params.script_meta.statements {
    if let Some(export_info) = &statement.export_info {
      if let Some(source) = &export_info.source {
        let source_module_id =
          params
            .module_graph
            .get_dep_by_source(params.module_id, source, None);
        let source_module = params.module_graph.module(&source_module_id).unwrap();

        // if the source module is external, we should keep the export statement
        if is_module_external(source_module, params.module_ids) {
          let handle_external_module_result = handle_external_module(
            HandleExternalModuleOptions::from(params, &source_module_id, statement),
          );
          ambiguous_export_all_idents
            .extend(handle_external_module_result.ambiguous_export_all_idents);
        } else {
          for specifier in &export_info.specifiers {
            match specifier {
              // Do nothing for export * from './module'; It's handled in `strip_import_statements` when tracing imported idents
              ExportSpecifierInfo::All => {
                let module = params.module_graph.module(params.module_id).unwrap();
                let module_meta = module.meta.as_script();
                // should handle ambiguous export all here
                if module_meta
                  .ambiguous_export_ident_map
                  .contains_key(AMBIGUOUS_EXPORT_ALL)
                {
                  ambiguous_export_all_idents.extend(handle_ambiguous_export_all(
                    HandleAmbiguousExportAllOptions {
                      module_id: params.module_id,
                      is_entry_module: params.is_entry_module,
                      result: params.result,
                      strip_context: params.strip_context,
                      module_graph: params.module_graph,
                      module_ids: params.module_ids,
                      source: &source,
                    },
                  ));
                }
              }
              ExportSpecifierInfo::Default => {
                unreachable!("export default xxx from is not valid esm syntax")
              }
              // export { foo, bar as baz, default as default } from './module';
              // =>
              // do nothing, the reexport should be traced when collecting export_ident_map and renamed in `strip_import_statements`
              ExportSpecifierInfo::Named { .. } => {}
              // export * as ns from './module';
              // =>
              // rename ns#1 to module_ns#1, where module_ns#1 is the ident generated by Farm and defined in source module
              ExportSpecifierInfo::Namespace(ident) => {
                let rename_handler = params.strip_context.rename_handler.clone();
                let mut rename_handler = rename_handler.borrow_mut();
                let source_module_script_meta = source_module.meta.as_script();

                rename_imported_ident(
                  params.module_id,
                  ident,
                  EXPORT_NAMESPACE,
                  source_module_script_meta,
                  &mut rename_handler,
                );
              }
            }
          }
        }

        statements_to_remove.push(statement.id);

        continue;
      }

      let StripModuleDeclStatementParams {
        module_id,
        result,
        strip_context,
        ..
      } = params;

      let rename_handler = strip_context.rename_handler.clone();
      let mut rename_handler = rename_handler.borrow_mut();

      let mut is_replace_module_decl = false;

      for specifier in &export_info.specifiers {
        match specifier {
          // Do nothing for export * from './module'; It's handled in `strip_import_statements` when tracing imported idents
          ExportSpecifierInfo::All => {}
          // export default xxx;
          // =>
          // rename xxx#1 to module_default#1;
          ExportSpecifierInfo::Default => {
            let item = replace_module_decl(statement, result);
            let expr = match item.expect_module_decl() {
              ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                match export_default_decl.decl {
                  farmfe_core::swc_ecma_ast::DefaultDecl::Class(class_expr) => {
                    Box::new(Expr::Class(class_expr))
                  }
                  farmfe_core::swc_ecma_ast::DefaultDecl::Fn(fn_expr) => {
                    Box::new(Expr::Fn(fn_expr))
                  }
                  farmfe_core::swc_ecma_ast::DefaultDecl::TsInterfaceDecl(_) => {
                    unreachable!()
                  }
                }
              }
              farmfe_core::swc_ecma_ast::ModuleDecl::ExportDefaultExpr(export_default_expr) => {
                export_default_expr.expr
              }
              _ => {
                unreachable!("statement other than `export default` should not be handled here")
              }
            };

            if statement.defined_idents.is_empty() {
              // export default '123' => var module_default = '123';
              let default_ident = create_export_default_ident(module_id);
              rename_handler.rename_ident_if_conflict(&module_id, &default_ident.to_id().into());
              result.ast.body[statement.id] = create_export_default_expr_item(expr, default_ident);
            } else {
              // export default function foo() {}
              // =>
              // function foo() {}
              for defined_ident in &statement.defined_idents {
                rename_handler.rename_ident_if_conflict(&module_id, defined_ident);
              }
              result.ast.body[statement.id] = ModuleItem::Stmt(Stmt::Decl(if expr.is_fn_expr() {
                let fn_expr = expr.expect_fn_expr();
                Decl::Fn(FnDecl {
                  ident: fn_expr.ident.unwrap(),
                  declare: false,
                  function: fn_expr.function,
                })
              } else {
                let class_expr = expr.expect_class();
                Decl::Class(ClassDecl {
                  ident: class_expr.ident.unwrap(),
                  declare: false,
                  class: class_expr.class,
                })
              }));
            }
          }
          // export { foo, bar as baz, foo as default };
          // export function foo() {}
          // =>
          // remove export named, it's already renamed in top level idents
          // rename foo#1 to foo$1#1 if there are conflicts
          ExportSpecifierInfo::Named { local, .. } => {
            if result.ast.body[statement.id]
              .as_module_decl()
              .unwrap()
              .as_export_decl()
              .is_some()
            {
              // a module decl statement may have multiple named exports like export const { foo, bar } = window;
              // we should only replace the module decl statement once after all named exports are handled
              is_replace_module_decl = true;
              rename_handler.rename_ident_if_conflict(&module_id, local);
            } else if !statements_to_remove.contains(&statement.id) {
              statements_to_remove.push(statement.id);
            }
          }
          ExportSpecifierInfo::Namespace(_) => {
            unreachable!("export * as xxx from should not be handled here")
          }
        }
      }

      if is_replace_module_decl {
        let item = replace_module_decl(statement, result);
        result.ast.body[statement.id] = ModuleItem::Stmt(Stmt::Decl(
          item.expect_module_decl().expect_export_decl().decl,
        ));
      }

      // remove this statement if specifiers is empty
      if export_info.specifiers.is_empty() {
        statements_to_remove.push(statement.id);
      }
    }
  }

  add_ambiguous_ident_decl(
    params.module_graph,
    params.module_id,
    ambiguous_export_all_idents,
    params.strip_context,
  );

  statements_to_remove
}

/// Return false means the ident is not renamed, it maybe an unresolved ident, we should print warning
fn rename_imported_ident(
  module_id: &ModuleId,
  ident: &SwcId,
  export_str: &str,
  source_module_script_meta: &ScriptModuleMetaData,
  rename_handler: &mut TopLevelIdentsRenameHandler,
) -> bool {
  // if the ident is already renamed, skip it
  if rename_handler.get_renamed_ident(module_id, ident).is_some() {
    return true;
  }

  // export { m as bar }
  // =>
  // Map<String, SwcId>(bar -> m#1)
  let module_export_ident =
    if let Some(module_export_ident) = source_module_script_meta.export_ident_map.get(export_str) {
      module_export_ident
    } else {
      return false;
    };

  let module_export_ident = module_export_ident.as_internal();

  // get the renamed ident if export_ident is renamed
  let final_ident = if matches!(
    module_export_ident.export_type,
    ModuleExportIdentType::Declaration | ModuleExportIdentType::VirtualNamespace
  ) {
    rename_handler
      .get_renamed_ident(&module_export_ident.module_id, &module_export_ident.ident)
      .unwrap_or(module_export_ident.ident.clone())
  } else if let Some(renamed_ident) =
    rename_handler.get_renamed_ident(&module_export_ident.module_id, &module_export_ident.ident)
  {
    renamed_ident
  } else {
    return false;
  };

  // rename local to final_ident
  rename_handler.rename_ident(module_id.clone(), ident.clone(), final_ident);

  true
}

pub fn is_module_external(source_module: &Module, module_ids: &HashSet<ModuleId>) -> bool {
  source_module.external || !module_ids.contains(&source_module.id)
}

/// Find the external source module that reexport the ident recursively
pub fn is_ident_reexported_from_external_module(
  module_ids: &HashSet<ModuleId>,
  source_module_id: &ModuleId,
  export_str: &str,
  module_graph: &ModuleGraph,
  rename_handler: &TopLevelIdentsRenameHandler,
  visited: &mut HashSet<ModuleId>,
) -> Option<(ModuleId, String)> {
  if visited.contains(source_module_id) {
    return None;
  }

  visited.insert(source_module_id.clone());

  let source_module = module_graph
    .module(source_module_id)
    .unwrap_or_else(|| panic!("source module {source_module_id:?} not found"));

  if source_module.external || !source_module.module_type.is_script() {
    return None;
  }

  let source_module_script_meta = source_module.meta.as_script();

  if !is_module_export_ident_declared(
    rename_handler,
    module_ids,
    export_str,
    source_module_script_meta,
  ) {
    return None;
  }

  // source module is external, we should preserve the import decl and rename the ident
  if is_module_external(source_module, module_ids) {
    return Some((source_module.id.clone(), export_str.to_string()));
  }

  if let Some(reexport) = source_module_script_meta.reexport_ident_map.get(export_str) {
    let (new_source_module_id, new_export_str) = match reexport {
      ModuleReExportIdentType::FromExportAll(from_module_id) => (from_module_id, export_str),
      ModuleReExportIdentType::FromExportNamed {
        local,
        from_module_id,
      } => (from_module_id, local.as_str()),
    };

    return is_ident_reexported_from_external_module(
      module_ids,
      new_source_module_id,
      new_export_str,
      module_graph,
      rename_handler,
      visited,
    );
  }

  None
}

fn is_module_export_ident_declared(
  rename_handler: &TopLevelIdentsRenameHandler,
  module_ids: &HashSet<ModuleId>,
  export_str: &str,
  source_module_script_meta: &ScriptModuleMetaData,
) -> bool {
  // export { m as bar }
  // =>
  // Map<String, SwcId>(bar -> m#1)
  let module_export_ident =
    if let Some(module_export_ident) = source_module_script_meta.export_ident_map.get(export_str) {
      module_export_ident
    } else {
      return false;
    };

  let module_export_ident = module_export_ident.as_internal();

  // the ident should be declared in the source module and the module defined this ident should be external(not in module_ids)
  matches!(
    module_export_ident.export_type,
    ModuleExportIdentType::Declaration
  ) && !module_ids.contains(&module_export_ident.module_id)
    && rename_handler
      .get_renamed_ident(&module_export_ident.module_id, &module_export_ident.ident)
      .is_none()
}

pub fn handle_external_module_idents(
  params: &mut StripModuleDeclStatementParams,
  source_module_id: &ModuleId,
  idents: Vec<(SwcId, String)>,
) -> Vec<StatementId> {
  let mut statements_to_remove = vec![];

  // add temporary import statement and reuse logic from handle_external_module
  let temp_import_stmt = create_import_stmt(create_import_specifiers(idents), &source_module_id);

  let temp_import_stmt_id = params.result.ast.body.len();
  let temp_statement = analyze_statement_info(&temp_import_stmt_id, &temp_import_stmt);
  // add temporary import statement to the ast
  params.result.ast.body.push(temp_import_stmt);
  // the temporary import should be removed after it's usage
  statements_to_remove.push(temp_import_stmt_id);

  handle_external_module(HandleExternalModuleOptions::from(
    params,
    &source_module_id,
    &temp_statement.into(),
  ));

  statements_to_remove
}
