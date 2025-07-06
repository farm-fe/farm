use farmfe_core::{
  module::{
    meta_data::script::{
      statement::{ExportSpecifierInfo, ImportSpecifierInfo, StatementId, SwcId},
      CommentsMetaData, ModuleExportIdentType, ScriptModuleMetaData,
    },
    module_graph::ModuleGraph,
    Module, ModuleId, ModuleSystem,
  },
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    ClassDecl, Decl, Expr, FnDecl, Ident, IdentName, MemberExpr, MemberProp, Module as SwcModule,
    ModuleDecl, ModuleItem, Stmt,
  },
  HashSet,
};
use swc_ecma_utils::StmtLikeInjector;

use super::{
  handle_external_modules::handle_external_modules,
  unique_idents::{TopLevelIdentsRenameHandler, EXPORT_DEFAULT, EXPORT_NAMESPACE},
  utils::{
    create_export_default_expr_item, create_export_default_ident, create_export_namespace_ident,
    create_var_decl_item, replace_module_decl,
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
  pub used_idents: HashSet<SwcId>,
  pub namespace_ident: Option<Ident>,
  pub is_namespace_import: bool,
}

/// Result after calling `strip_module_decl`
pub struct StripModuleDeclResult {
  /// the ast that removed the import/export statements
  pub ast: SwcModule,
  pub comments: CommentsMetaData,
  pub items_to_prepend: Vec<ModuleItem>,
}

pub fn strip_module_decl(
  module_id: &ModuleId,
  module_ids: &HashSet<ModuleId>,
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
  };

  let mut statements_to_remove = vec![];
  let mut params = StripModuleDeclStatementParams {
    module_id,
    module_ids,
    script_meta,
    result: &mut result,
    strip_context,
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

  let items_to_prepend = std::mem::take(&mut result.items_to_prepend);
  // prepend preserved import decl
  result
    .ast
    .body
    .prepend_stmts(items_to_prepend.into_iter().rev());

  result
}

pub struct StripModuleDeclStatementParams<'a> {
  module_id: &'a ModuleId,
  module_ids: &'a HashSet<ModuleId>,
  script_meta: &'a ScriptModuleMetaData,
  result: &'a mut StripModuleDeclResult,
  strip_context: &'a mut StripModuleContext,
  module_graph: &'a ModuleGraph,
}

fn strip_import_statements(params: &mut StripModuleDeclStatementParams) -> Vec<StatementId> {
  let mut statements_to_remove = vec![];
  let mut items_to_prepend = vec![];

  let StripModuleDeclStatementParams {
    module_id,
    module_ids,
    script_meta,
    result,
    strip_context,
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
        handle_external_modules(
          &module_id,
          &source_module_id,
          statement,
          result,
          strip_context,
        );
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

        let mut rename_handler = strip_context.rename_handler.borrow_mut();
        // true means the ident is an unresolved ident
        if rename_imported_ident(
          module_id,
          ident,
          export_str,
          script_meta,
          source_module_script_meta,
          &mut rename_handler,
          result,
        ) {
          // try read ambiguous export, for export * from 'xx.cjs', the imported ident will be stored in ambiguous_export_ident_map
          if let Some(ambiguous_export_idents) = source_module_script_meta
            .ambiguous_export_ident_map
            .get(export_str)
          {
            if ambiguous_export_idents.len() > 1 {
              println!(
                "[Farm warn] {} export is ambiguous in module {}. The first one will be used, which is {} defined in {}.",
                export_str,
                source_module_id.to_string(),
                ambiguous_export_idents[0].as_internal().ident.sym,
                ambiguous_export_idents[0].as_internal().module_id.to_string(),
              );
            }

            // if the source module is a es module, we should use the export ident in the source module
            // for example, cjs module will be transformed to es module, and the imported ident will be generated by plugin_library
            if ambiguous_export_idents.len() > 0 {
              let module_export_ident = &ambiguous_export_idents[0].as_internal();
              // if the module where the ident is defined is a es module
              if let Some(original_defined_module) =
                module_graph.module(&module_export_ident.module_id)
              {
                if original_defined_module.module_type.is_script()
                  && !original_defined_module.external
                  && module_export_ident.export_type == ModuleExportIdentType::Unresolved
                {
                  let original_meta = original_defined_module.meta.as_script();

                  if original_meta.module_system == ModuleSystem::EsModule {
                    let final_ident = rename_handler
                      .get_renamed_ident(&module_export_ident.module_id, &module_export_ident.ident)
                      .unwrap_or(module_export_ident.ident.clone());
                    rename_handler.rename_ident(module_id.clone(), ident.clone(), final_ident);

                    continue;
                  }
                }
              }
            }
          }

          // rename the imported ident to the unique name
          rename_handler.rename_ident_if_conflict(&source_module_id, &ident);
          // make sure the ident is unique when creating the var decl item
          let renamed_ident = if let Some(renamed_ident) =
            rename_handler.get_renamed_ident(&source_module_id, &ident)
          {
            rename_handler.rename_ident(module_id.clone(), ident.clone(), renamed_ident.clone());
            renamed_ident
          } else {
            ident.clone()
          };

          // read export namespace ident from source module,
          // for cjs module, the export namespace ident is __farm_cjs_export__ instead of the default namespace ident
          let namespace_var_ident = if let Some(export_namespace_ident) = source_module_script_meta
            .export_ident_map
            .get(EXPORT_NAMESPACE)
          {
            export_namespace_ident.as_internal().ident.clone()
          } else {
            create_export_namespace_ident(&source_module_id)
              .to_id()
              .into()
          };

          let renamed_namespace_var_ident = rename_handler
            .get_renamed_ident(module_id, &namespace_var_ident)
            .unwrap_or(namespace_var_ident);

          let item = create_var_decl_item(
            Ident::new(renamed_ident.sym.clone(), DUMMY_SP, renamed_ident.ctxt()),
            Box::new(Expr::Member(MemberExpr {
              span: DUMMY_SP,
              obj: Box::new(Expr::Ident(Ident::new(
                renamed_namespace_var_ident.sym.clone(),
                DUMMY_SP,
                renamed_namespace_var_ident.ctxt(),
              ))),
              prop: MemberProp::Ident(IdentName::new(ident.sym.clone(), DUMMY_SP)),
            })),
          );

          items_to_prepend.push(item);
        };
      }
    }
  }

  params.result.items_to_prepend.extend(items_to_prepend);

  statements_to_remove
}

fn strip_export_statements(params: &mut StripModuleDeclStatementParams) -> Vec<StatementId> {
  let StripModuleDeclStatementParams {
    module_id,
    module_ids,
    script_meta,
    result,
    strip_context,
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
          handle_external_modules(
            &module_id,
            &source_module_id,
            statement,
            result,
            strip_context,
          );
        } else {
          let rename_handler = strip_context.rename_handler.clone();
          let mut rename_handler = rename_handler.borrow_mut();

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
              // rename ns#1 to module_ns#1, where module_ns#1 is the ident generated by Farm and defined in source module
              ExportSpecifierInfo::Namespace(ident) => {
                let source_module_script_meta = source_module.meta.as_script();

                rename_imported_ident(
                  module_id,
                  ident,
                  EXPORT_NAMESPACE,
                  script_meta,
                  source_module_script_meta,
                  &mut rename_handler,
                  result,
                );
              }
            }
          }
        }

        statements_to_remove.push(statement.id);

        continue;
      }

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

  statements_to_remove
}

/// Return true means the ident is not renamed, it maybe an unresolved ident
fn rename_imported_ident(
  module_id: &ModuleId,
  ident: &SwcId,
  export_str: &str,
  script_meta: &ScriptModuleMetaData,
  source_module_script_meta: &ScriptModuleMetaData,
  rename_handler: &mut TopLevelIdentsRenameHandler,
  result: &mut StripModuleDeclResult,
) -> bool {
  // export { m as bar }
  // =>
  // Map<String, SwcId>(bar -> m#1)
  let module_export_ident = source_module_script_meta.export_ident_map.get(export_str);

  if module_export_ident.is_none() {
    return true;
  }

  let module_export_ident = module_export_ident.unwrap().as_internal();

  // get the renamed ident if export_ident is renamed
  let final_ident = rename_handler
    .get_renamed_ident(&module_export_ident.module_id, &module_export_ident.ident)
    .unwrap_or(module_export_ident.ident.clone());

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

  false
}

fn is_module_external(source_module: &Module, module_ids: &HashSet<ModuleId>) -> bool {
  source_module.external || !module_ids.contains(&source_module.id)
}
