use std::{path::PathBuf, sync::Arc};

use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};

use farmfe_core::{
  config::ScriptParserConfig,
  module::ModuleType,
  swc_common::{comments::SingleThreadedComments, Mark, SourceMap, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    CallExpr, Callee, Decl, Expr, ExprOrSpread, Ident, Import, Module as SwcModule, ModuleItem,
    Stmt, VarDecl,
  },
};

pub use super::concatenate_modules::utils::{
  create_ambiguous_export_all_ident, create_export_default_ident, create_export_namespace_ident,
  create_import_farm_register_helper_stmt,
};

pub struct ParseScriptModuleResult {
  pub ast: SwcModule,
  pub comments: SingleThreadedComments,
  pub source_map: Arc<SourceMap>,
}

/// Get [ModuleType] from the resolved id's extension, return [ModuleType::Custom(ext)] if the extension is not internally supported.
/// Panic if the id do not has a extension.
pub fn module_type_from_id(id: &str) -> Option<ModuleType> {
  let path = PathBuf::from(id);

  path.extension().map(|ext| ext.to_str().unwrap().into())
}

/// return [None] if module type is not script
pub fn syntax_from_module_type(
  module_type: &ModuleType,
  config: ScriptParserConfig,
) -> Option<Syntax> {
  match module_type {
    ModuleType::Js => Some(Syntax::Es(EsSyntax {
      jsx: false,
      import_attributes: true,
      ..config.es_config
    })),
    ModuleType::Jsx => Some(Syntax::Es(EsSyntax {
      jsx: true,
      import_attributes: true,
      ..config.es_config
    })),
    ModuleType::Ts => Some(Syntax::Typescript(TsSyntax {
      tsx: false,
      ..config.ts_config
    })),
    ModuleType::Tsx => Some(Syntax::Typescript(TsSyntax {
      tsx: true,
      ..config.ts_config
    })),
    _ => None,
  }
}

/// Whether the call expr is commonjs require.
/// A call expr is commonjs require if:
/// * callee is an identifier named `require`
/// * arguments is a single string literal
/// * require is a global variable
pub fn is_commonjs_require(
  unresolved_mark: Mark,
  top_level_mark: Mark,
  call_expr: &CallExpr,
) -> bool {
  if let Callee::Expr(box Expr::Ident(Ident { ctxt, sym, .. })) = &call_expr.callee {
    sym == "require" && (ctxt.outer() == unresolved_mark || ctxt.outer() == top_level_mark)
  } else {
    false
  }
}

/// Whether the call expr is dynamic import.
pub fn is_dynamic_import(call_expr: &CallExpr) -> bool {
  matches!(&call_expr.callee, Callee::Import(Import { .. }))
}

pub fn create_top_level_ident(id: &str, top_level_mark: Mark) -> Ident {
  Ident::new(
    id.into(),
    DUMMY_SP,
    SyntaxContext::empty().apply_mark(top_level_mark),
  )
}

pub fn create_var_declarator(id: Ident, expr: Expr) -> farmfe_core::swc_ecma_ast::VarDeclarator {
  farmfe_core::swc_ecma_ast::VarDeclarator {
    span: DUMMY_SP,
    name: farmfe_core::swc_ecma_ast::Pat::Ident(farmfe_core::swc_ecma_ast::BindingIdent {
      id,
      type_ann: None,
    }),
    init: Some(Box::new(expr)),
    definite: false,
  }
}
pub fn create_var_decl_item(decls: Vec<farmfe_core::swc_ecma_ast::VarDeclarator>) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: Default::default(),
    decls,
    ctxt: SyntaxContext::empty(),
    declare: false,
  }))))
}

pub fn create_call_expr(callee: Expr, args: Vec<ExprOrSpread>) -> Expr {
  Expr::Call(CallExpr {
    span: DUMMY_SP,
    ctxt: SyntaxContext::empty(),
    callee: Callee::Expr(Box::new(callee)),
    args,
    type_args: None,
  })
}
