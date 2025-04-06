use farmfe_core::{
  module::{
    meta_data::script::{
      statement::{ExportSpecifierInfo, ImportSpecifierInfo, StatementId, SwcId},
      CommentsMetaData, ModuleExportIdent, ModuleExportIdentType, ScriptModuleMetaData,
      EXPORT_EXTERNAL_NAMESPACE,
    },
    module_graph::ModuleGraph,
    Module, ModuleId,
  },
  swc_common::{
    comments::{Comment, CommentKind, Comments, SingleThreadedComments},
    BytePos, Mark, SyntaxContext, DUMMY_SP,
  },
  swc_ecma_ast::{
    ClassDecl, Decl, ExportNamedSpecifier, ExportSpecifier, Expr, FnDecl, Ident, IdentName,
    MemberExpr, MemberProp, Module as SwcModule, ModuleDecl, ModuleExportName, ModuleItem,
    NamedExport, Stmt,
  },
  HashMap, HashSet,
};
use swc_ecma_visit::VisitMutWith;

use super::{
  handle_external_modules::handle_external_modules,
  unique_idents::{RenameVisitor, TopLevelIdentsRenameHandler, EXPORT_DEFAULT, EXPORT_NAMESPACE},
  utils::{
    create_export_default_expr_item, create_export_default_ident, create_var_decl_item,
    replace_module_decl,
  },
  StripModuleContext,
};

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
  pub is_namespace_import: bool,
}

/// Result after calling `strip_module_decl`
pub struct StripModuleDeclResult {
  /// the ast that removed the import/export statements
  pub ast: SwcModule,
  pub comments: CommentsMetaData,
}

pub fn strip_module_decl(
  module_id: &ModuleId,
  module_ids: &HashSet<ModuleId>,
  module_graph: &ModuleGraph,
  strip_context: &mut StripModuleContext,
  module_export_ident_map: &mut HashMap<ModuleId, HashMap<String, SwcId>>,
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

  // add preserved export decl
  if let Some(module_export_ident) = module_export_ident_map.remove(module_id) {
    let mut specifiers = vec![];

    for (name, id) in module_export_ident {
      specifiers.push(ExportSpecifier::Named(ExportNamedSpecifier {
        span: DUMMY_SP,
        orig: ModuleExportName::Ident(Ident::new(id.sym.clone(), DUMMY_SP, id.ctxt())),
        exported: Some(ModuleExportName::Ident(Ident::new(
          name.as_str().into(),
          DUMMY_SP,
          SyntaxContext::empty(),
        ))),
        is_type_only: false,
      }));
    }

    let mut item = ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
      span: DUMMY_SP,
      specifiers,
      src: None,
      type_only: false,
      with: None,
    }));

    let mut rename_handler = strip_context.rename_handler.borrow_mut();
    let mut rename_visitor = RenameVisitor::new(module_id, &mut rename_handler);
    item.visit_mut_with(&mut rename_visitor);

    strip_context.preserved_export_decls.push(item);
  }

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
          &source_module_id,
          source_module_script_meta,
          &mut rename_handler,
        ) {
          if let Some(export_ident) = source_module_script_meta
            .export_ident_map
            .get(EXPORT_EXTERNAL_NAMESPACE)
          {
            let item = create_var_decl_item(
              Ident::new(ident.sym.clone(), DUMMY_SP, ident.ctxt()),
              Box::new(Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(Ident::new(
                  export_ident.ident.sym.clone(),
                  DUMMY_SP,
                  export_ident.ident.ctxt(),
                ))),
                prop: MemberProp::Ident(IdentName::new(ident.sym.clone(), DUMMY_SP)),
              })),
            );
            strip_context.extra_external_module_items.push(item);
          };
        }
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
                  &source_module_id,
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
              let top_level_mark = Mark::from_u32(script_meta.top_level_mark);
              // export default '123' => var module_default = '123';
              let default_ident = create_export_default_ident(module_id, top_level_mark);
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

fn rename_imported_ident(
  module_id: &ModuleId,
  ident: &SwcId,
  export_str: &str,
  source_module_id: &ModuleId,
  source_module_script_meta: &ScriptModuleMetaData,
  rename_handler: &mut TopLevelIdentsRenameHandler,
) -> bool {
  let default_ident = ModuleExportIdent {
    module_id: source_module_id.clone(),
    ident: ident.clone(),
    export_type: ModuleExportIdentType::Unresolved,
  };
  // export { m as bar }
  // =>
  // Map<String, SwcId>(bar -> m#1)
  let module_export_ident = source_module_script_meta
    .export_ident_map
    .get(export_str)
    .unwrap_or_else(|| {
      // all imported ident should be in the export ident map when expand_exports.
      // for case `export * from './module';` where ./module is a external module.
      // a virtual ident is generated for the module, and the ident type is [ModuleExportIdent::External]
      println!(
        "export ident map not found for export_str: {}, source_module_id: {}",
        export_str,
        source_module_id.to_string()
      );
      // TODO find dependency recursively to get the export ident
      &default_ident
    });

  // TODO: trace if there are unresolved export idents like external module or cjs module.
  // and we should create a new ident for it. for external module, see `handle_external_export_all`
  // for cjs module, `var external_all_farm_internal_ = __commonJS((module, exports) => {});`, then the same as external module
  // println!(
  //   "module_export_ident: {}, {:?}, module_id: {}, ident: {:?}",
  //   export_str,
  //   module_export_ident,
  //   module_id.to_string(),
  //   ident
  // );

  // get the renamed ident if export_ident is renamed
  let final_ident = rename_handler
    .get_renamed_ident(&module_export_ident.module_id, &module_export_ident.ident)
    .unwrap_or(module_export_ident.ident.clone());

  // rename local to final_ident
  rename_handler.rename_ident(module_id.clone(), ident.clone(), final_ident);

  // if ident is External or unresolved, we should try to find it in the namespace import
  // like `var createRequire = node_fs_external_namespace_internal_.createRequire`
  matches!(
    module_export_ident.export_type,
    ModuleExportIdentType::Unresolved
  )
}

fn is_module_external(source_module: &Module, module_ids: &HashSet<ModuleId>) -> bool {
  source_module.external || !module_ids.contains(&source_module.id)
}
