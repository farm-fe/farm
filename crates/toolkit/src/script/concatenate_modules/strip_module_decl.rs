use farmfe_core::{
  module::{
    meta_data::script::{
      statement::{ExportSpecifierInfo, ImportSpecifierInfo, StatementId, SwcId},
      CommentsMetaData, ModuleExportIdentType, ScriptModuleMetaData, AMBIGUOUS_EXPORT_ALL,
    },
    module_graph::ModuleGraph,
    Module, ModuleId,
  },
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    ClassDecl, Decl, Expr, FnDecl, Ident, Module as SwcModule, ModuleDecl, ModuleItem, Stmt,
  },
  HashSet,
};

use crate::script::concatenate_modules::handle_external_modules::{
  add_ambiguous_ident_decl, handle_ambiguous_export_all, HandleAmbiguousExportAllOptions,
  HandleExternalModuleOptions,
};

use super::{
  handle_external_modules::handle_external_module,
  unique_idents::{TopLevelIdentsRenameHandler, EXPORT_DEFAULT, EXPORT_NAMESPACE},
  utils::{
    create_export_default_expr_item, create_export_default_ident, create_var_decl_item,
    replace_module_decl,
  },
  StripModuleContext,
};

#[derive(Debug)]
pub enum PreservedImportDeclType {
  /// `import * as external_namespace_farm_internal_ from 'module';` generated when handling external module
  ExternalGenerated,
  /// `import { m as bar } from 'module';`. Original import statement when handling external module
  ExternalOriginal,
}

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
        script_meta,
        result,
        strip_context,
        ..
      } = params;

      let source_module_script_meta = source_module.meta.as_script();

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

        // true means the ident is an unresolved ident
        if !rename_imported_ident(
          module_id,
          ident,
          export_str,
          script_meta,
          source_module_script_meta,
          &mut rename_handler,
          result,
        ) {
          println!(
            "[Farm warn] rename imported ident failed (module_id: {:?}), please make sure export {export_str} is defined in {:?}",
            module_id.to_string(),
            source_module_id.to_string()
          );
        };
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
                  params.script_meta,
                  source_module_script_meta,
                  &mut rename_handler,
                  params.result,
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
  script_meta: &ScriptModuleMetaData,
  source_module_script_meta: &ScriptModuleMetaData,
  rename_handler: &mut TopLevelIdentsRenameHandler,
  result: &mut StripModuleDeclResult,
) -> bool {
  // if the ident is already renamed, skip it
  if rename_handler.get_renamed_ident(module_id, ident).is_some() {
    return true;
  }

  // export { m as bar }
  // =>
  // Map<String, SwcId>(bar -> m#1)
  let module_export_ident = source_module_script_meta.export_ident_map.get(export_str);

  if module_export_ident.is_none() {
    return false;
  }

  let module_export_ident = module_export_ident.unwrap().as_internal();

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

  if script_meta
    .all_deeply_declared_idents
    .contains(&final_ident.sym)
  {
    // there are name conflicts deeply in the module, for example:
    // ```
    // // xxx
    // export const a = 'a';
    //
    // // index.js
    // import { a as renamedA } from 'xxx'
    // function A() {
    //   const a = 2;
    //   console.log(renamedA);
    // }
    // ```
    // should be renamed to:
    // ```
    // const a = 'a'
    // const a$1 = a;
    // function A() {
    //   const a = 2;
    //   console.log(a$1)
    // }
    // ```
    // we have to rename a to a$1 to avoid ident conflicts
    let renamed_ident = rename_handler
      .get_unique_ident(&final_ident)
      .unwrap_or(final_ident.clone());

    result.items_to_prepend.push(create_var_decl_item(
      Ident::new(renamed_ident.sym.clone(), DUMMY_SP, renamed_ident.ctxt()),
      Box::new(Expr::Ident(Ident::new(
        final_ident.sym.clone(),
        DUMMY_SP,
        SyntaxContext::empty(), // there may be same ident in different module, so we should use empty context to make sure it's won't renamed
      ))),
    ));
    rename_handler.rename_ident(module_id.clone(), ident.clone(), renamed_ident);
  } else {
    // rename local to final_ident
    rename_handler.rename_ident(module_id.clone(), ident.clone(), final_ident);
  }

  true
}

pub fn is_module_external(source_module: &Module, module_ids: &HashSet<ModuleId>) -> bool {
  source_module.external || !module_ids.contains(&source_module.id)
}
