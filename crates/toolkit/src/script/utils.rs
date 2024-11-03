use std::path::PathBuf;

use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};

use farmfe_core::{
  config::ScriptParserConfig,
  module::ModuleType,
  swc_common::Mark,
  swc_ecma_ast::{CallExpr, Callee, Expr, Ident, Import},
};

pub use farmfe_toolkit_plugin_types::swc_ast::ParseScriptModuleResult;

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
