use farmfe_core::{
  module::{
    meta_data::script::{
      statement::{ExportSpecifierInfo, ImportSpecifierInfo, Statement, StatementId, SwcId},
      CommentsMetaData, ScriptModuleMetaData,
    },
    module_graph::ModuleGraph,
    Module, ModuleId,
  },
  swc_common::{
    comments::{Comment, CommentKind, Comments, SingleThreadedComments},
    BytePos, Mark, DUMMY_SP,
  },
  swc_ecma_ast::{EmptyStmt, Expr, ExprStmt, Module as SwcModule, ModuleDecl, ModuleItem, Stmt},
  HashSet,
};

use super::{
  unique_idents::{TopLevelIdentsRenameHandler, EXPORT_DEFAULT, EXPORT_NAMESPACE},
  utils::{create_export_default_expr_item, create_export_default_ident},
};

/// Result after calling `strip_module_decl`
pub struct StripModuleDeclResult {
  /// the ast that removed the import/export statements
  pub ast: SwcModule,
  pub comments: CommentsMetaData,
  /// the preserved import or export from statements
  pub preserved_module_decls: Vec<(ModuleItem, ModuleId)>,
}

pub fn strip_module_decl(
  module_id: &ModuleId,
  module_ids: &HashSet<ModuleId>,
  module_graph: &ModuleGraph,
  rename_handler: &mut TopLevelIdentsRenameHandler,
) -> StripModuleDeclResult {
  let module = module_graph.module(module_id).unwrap();
  let script_meta = module.meta.as_script();

  let comments = script_meta.comments.clone();
  let swc_comments: SingleThreadedComments = comments.into();
  // prepend filename comment
  swc_comments.add_leading(
    BytePos::DUMMY,
    Comment {
      kind: CommentKind::Line,
      span: DUMMY_SP,
      text: module_id.to_string().into(),
    },
  );

  let mut result = StripModuleDeclResult {
    ast: script_meta.ast.clone(),
    comments: swc_comments.into(),
    preserved_module_decls: vec![],
  };

  let mut statements_to_remove = vec![];
  let mut params = StripModuleDeclStatementParams {
    module_id,
    module_ids,
    script_meta,
    result: &mut result,
    rename_handler,
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
  module_id: &'a ModuleId,
  module_ids: &'a HashSet<ModuleId>,
  script_meta: &'a ScriptModuleMetaData,
  result: &'a mut StripModuleDeclResult,
  rename_handler: &'a mut TopLevelIdentsRenameHandler,
  module_graph: &'a ModuleGraph,
}

fn strip_import_statements(params: &mut StripModuleDeclStatementParams) -> Vec<StatementId> {
  let mut statements_to_remove = vec![];
  let StripModuleDeclStatementParams {
    module_id,
    module_ids,
    script_meta,
    result,
    rename_handler,
    module_graph,
  } = params;

  for statement in &script_meta.statements {
    if let Some(import_info) = &statement.import_info {
      // always remove import statement
      statements_to_remove.push(statement.id);

      let source_module_id = module_graph.get_dep_by_source(module_id, &import_info.source, None);
      let source_module = module_graph.module(&source_module_id).unwrap();
      // if the source module is external, we should keep the import statement
      if is_module_external(source_module, module_ids) {
        handle_external_modules(source_module_id, statement, result, rename_handler);
        continue;
      }

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

        rename_imported_ident(ident, export_str, source_module_script_meta, rename_handler);
      }
    }
  }

  statements_to_remove
}

fn strip_export_statements(params: &mut StripModuleDeclStatementParams) -> Vec<StatementId> {
  let StripModuleDeclStatementParams {
    module_id,
    module_ids,
    script_meta,
    result,
    rename_handler,
    module_graph,
  } = params;
  let mut statements_to_remove = vec![];

  for statement in &script_meta.statements {
    if let Some(export_info) = &statement.export_info {
      if let Some(source) = &export_info.source {
        let source_module_id = module_graph.get_dep_by_source(module_id, source, None);
        let source_module = module_graph.module(&source_module_id).unwrap();
        // if the source module is external, we should keep the export statement
        if is_module_external(source_module, module_ids) {
          handle_external_modules(source_module_id, statement, result, rename_handler);
        } else {
          for specifier in &export_info.specifiers {
            match specifier {
              // Do nothing for export * from './module'; It's handled in `strip_import_statements` when tracing imported idents
              ExportSpecifierInfo::All => {}
              ExportSpecifierInfo::Default => {
                unreachable!("export default xxx from is not valid esm syntax")
              }
              // export { foo, bar as baz, default as default } from './module';
              // =>
              // do nothing, the reexport should be traced when collecting export_ident_map and renamed in `strip_import_statements`
              ExportSpecifierInfo::Named { .. } => {}
              // export * as ns from './module';
              // =>
              // rename ns#1 to module_ns#1;
              ExportSpecifierInfo::Namespace(ident) => {
                let source_module_script_meta = source_module.meta.as_script();
                rename_imported_ident(
                  ident,
                  EXPORT_NAMESPACE,
                  source_module_script_meta,
                  rename_handler,
                );
              }
            }
          }
        }

        statements_to_remove.push(statement.id);

        continue;
      }

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
              let top_level_mark = Mark::from_u32(script_meta.top_level_mark);
              // export default '123' => var module_default = '123';
              let default_ident = create_export_default_ident(module_id, top_level_mark);
              rename_handler.rename_ident_if_conflict(&default_ident.to_id().into());
              result.ast.body[statement.id] = create_export_default_expr_item(expr, default_ident);
            } else {
              // export default function foo() {}
              // =>
              // function foo() {}
              for defined_ident in &statement.defined_idents {
                rename_handler.rename_ident_if_conflict(defined_ident);
              }
              result.ast.body[statement.id] = ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                span: DUMMY_SP,
                expr,
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
              let item = replace_module_decl(statement, result);
              result.ast.body[statement.id] = ModuleItem::Stmt(Stmt::Decl(
                item.expect_module_decl().expect_export_decl().decl,
              ));
              rename_handler.rename_ident_if_conflict(local);
            } else if !statements_to_remove.contains(&statement.id) {
              statements_to_remove.push(statement.id);
            }
          }
          ExportSpecifierInfo::Namespace(_) => {
            unreachable!("export * as xxx from should not be handled here")
          }
        }
      }
    }
  }

  statements_to_remove
}

// replace the module decl statement to empty statement
fn replace_module_decl(statement: &Statement, result: &mut StripModuleDeclResult) -> ModuleItem {
  std::mem::replace(
    &mut result.ast.body[statement.id],
    ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP })),
  )
}

fn handle_external_modules(
  source_module_id: ModuleId,
  statement: &Statement,
  result: &mut StripModuleDeclResult,
  rename_handler: &mut TopLevelIdentsRenameHandler,
) {
  let item = replace_module_decl(statement, result);
  // rename the imported ident if there are conflicts
  for defined_ident in &statement.defined_idents {
    rename_handler.rename_ident_if_conflict(defined_ident);
  }

  result.preserved_module_decls.push((item, source_module_id));
}

fn rename_imported_ident(
  ident: &SwcId,
  export_str: &str,
  source_module_script_meta: &ScriptModuleMetaData,
  rename_handler: &mut TopLevelIdentsRenameHandler,
) {
  // export { m as bar }
  // =>
  // Map<String, SwcId>(bar -> m#1)
  let export_ident = source_module_script_meta
    .export_ident_map
    .get(export_str)
    .unwrap_or_else(|| panic!("export ident {export_str} not found"));
  // get the renamed ident if export_ident is renamed
  let final_ident = rename_handler
    .get_renamed_ident(export_ident)
    .unwrap_or(export_ident.clone());

  // rename local to final_ident
  rename_handler.rename_ident(ident.clone(), final_ident);
}

fn is_module_external(source_module: &Module, module_ids: &HashSet<ModuleId>) -> bool {
  source_module.external || !module_ids.contains(&source_module.id)
}
