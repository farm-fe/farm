use std::sync::Arc;

use farmfe_core::{swc_ecma_ast::{Module as SwcModule, ModuleItem, ModuleDecl, Str, ImportDecl, ImportSpecifier, ImportDefaultSpecifier, Ident, CallExpr, Callee, Expr, MemberExpr, MemberProp, ExprOrSpread, Stmt, ExprStmt, ArrayLit}, context::CompilationContext, swc_common::DUMMY_SP, config::{FARM_GLOBAL_THIS, FARM_MODULE_SYSTEM}};

pub fn insert_runtime_plugins(ast: &mut SwcModule, context: &Arc<CompilationContext>) {
  // find the last import statement and insert the runtime plugins after it
  let last_import_stmt_index = ast.body.iter().enumerate().rev().find_map(|(i, stmt)| {
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(_)) = stmt {
      Some(i)
    } else {
      None
    }
  }).unwrap_or(0);

  let plugin_var_prefix = "__farm_plugin_";

  let import_plugin_stmts = context.config.runtime.plugins.iter().enumerate().map(|(i, plugin_path)| {
    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
      span: DUMMY_SP,
      specifiers: vec![
        ImportSpecifier::Default(ImportDefaultSpecifier {
          span: DUMMY_SP,
          local: Ident::new(format!("{}{}", plugin_var_prefix, i).as_str().into(), DUMMY_SP),
        })
      ],
      src: Str {
        span: DUMMY_SP,
        value: plugin_path.as_str().into(),
        raw: None
      },
      type_only: false,
      asserts: None,
    }))
  });

  ast.body.splice(last_import_stmt_index..last_import_stmt_index, import_plugin_stmts);

  // insert the setPlugins call at the end of the module
  let set_plugins_call = CallExpr {
    span: DUMMY_SP,
    callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(Ident::new(FARM_GLOBAL_THIS.into(), DUMMY_SP))),
        prop: MemberProp::Ident(Ident::new(FARM_MODULE_SYSTEM.into(), DUMMY_SP)),
      })),
      prop: MemberProp::Ident(Ident::new("setPlugins".into(), DUMMY_SP)),
    }))),
    args: vec![ExprOrSpread {
      spread: None,
      expr: Box::new(Expr::Array(ArrayLit {
        span: DUMMY_SP,
        elems: context.config.runtime.plugins.iter().enumerate().map(|(i, _)| {
          Some(ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Ident(Ident::new(format!("{}{}", plugin_var_prefix, i).as_str().into(), DUMMY_SP)))
          })
        }).collect()
      }))
    }],
    type_args: None,
};
  ast.body.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Call(set_plugins_call)),
  })));
}