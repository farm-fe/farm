use farmfe_core::{
  config::FARM_REQUIRE,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    ArrayLit, ArrayPat, AwaitExpr, BindingIdent, CallExpr, Callee, Decl, Expr, ExprOrSpread,
    ExprStmt, Ident, Lit, MemberExpr, MemberProp, Module, ModuleItem, Pat, Stmt, Str, VarDecl,
    VarDeclKind, VarDeclarator,
  },
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

/// Transform asy module to meet the requirements of farm runtime
/// Example, transform:
/// ```js
/// const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
/// const _sync = _interop_require_default._(farmRequire("475776c7"));
/// const _dep2 = _interop_require_default._(farmRequire("ea236e3d"));
/// console.log(_dep2.default)
/// ```
/// To:
/// ```js
/// let [_interop_require_default__f, _sync__f, _dep2__f] = await Promise.all([
///   farmRequire("@swc/helpers/_/_interop_require_default"),
///   farmRequire("475776c7"),
///   farmRequire("ea236e3d")
/// ]);
/// _interop_require_default = _interop_require_default;
/// const _sync = _interop_require_default._(_sync__f);
/// const _dep2 = _interop_require_default._(_dep2__f);
/// console.log(_dep2.default)
/// ```
pub fn transform_async_module(ast: &mut Module) {
  let mut await_all = vec![];
  let mut stmt_to_remove = vec![];
  let mut first_require_index = usize::MAX;

  // 1. collect top level farmRequire pattern
  for (i, item) in ast.body.iter_mut().enumerate() {
    if let ModuleItem::Stmt(stmt) = item {
      match stmt {
        // const _sync = _interop_require_default._(farmRequire("475776c7"))
        Stmt::Decl(Decl::Var(box VarDecl { decls, .. })) => {
          for decl in decls {
            if let Pat::Ident(BindingIdent {
              id: Ident { sym, .. },
              ..
            }) = &decl.name
            {
              let name = sym.to_string();

              if let Some(box expr) = &mut decl.init {
                let mut visitor = FarmRequireVisitor::new(name.clone());
                expr.visit_mut_with(&mut visitor);
                if visitor.requires.len() == 1 {
                  await_all.push((Some(name), visitor.requires.remove(0)));
                  first_require_index = first_require_index.min(i);
                }
              }
            }
          }
        }
        // farmRequire("ea236e3d")
        Stmt::Expr(ExprStmt { box expr, .. }) => {
          if let Some(id) = try_get_farm_require_id(expr) {
            await_all.push((None, id));
            stmt_to_remove.push(i);
            first_require_index = first_require_index.min(i);
          }
        }
        _ => { /* ignore other stmts */ }
      }
    }
  }

  // remove farmRequire stmt
  stmt_to_remove.reverse();
  for i in stmt_to_remove {
    ast.body.remove(i);
  }

  // 2. transform the patterns, example
  // let [_interop_require_default__f, _sync__f, _dep2__f] = await Promise.all([
  //   farmRequire("@swc/helpers/_/_interop_require_default"),
  //   farmRequire("475776c7"),
  //   farmRequire("ea236e3d")
  // ]);
  if !await_all.is_empty() {
    let await_all_stmt = Stmt::Decl(Decl::Var(Box::new(VarDecl {
      span: DUMMY_SP,
      kind: VarDeclKind::Const,
      declare: false,
      decls: vec![VarDeclarator {
        span: DUMMY_SP,
        name: Pat::Array(ArrayPat {
          span: DUMMY_SP,
          elems: await_all
            .iter()
            .map(|(id, _)| {
              id.as_ref().map(|id| {
                Pat::Ident(BindingIdent {
                  id: Ident {
                    span: DUMMY_SP,
                    sym: rename_ident(id).into(),
                    optional: false,
                  },
                  type_ann: None,
                })
              })
            })
            .collect(),
          optional: false,
          type_ann: None,
        }),
        init: Some(Box::new(create_promise_all(&await_all))),
        definite: false,
      }],
    })));
    ast
      .body
      .insert(first_require_index, ModuleItem::Stmt(await_all_stmt));
  }
}

fn create_promise_all(await_all: &Vec<(Option<String>, String)>) -> Expr {
  Expr::Await(AwaitExpr {
    span: DUMMY_SP,
    arg: Box::new(Expr::Call(CallExpr {
      span: DUMMY_SP,
      // Promise.all
      callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(Ident {
          span: DUMMY_SP,
          sym: "Promise".into(),
          optional: false,
        })),
        prop: MemberProp::Ident(Ident {
          span: DUMMY_SP,
          sym: "all".into(),
          optional: false,
        }),
      }))),
      args: vec![ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Array(ArrayLit {
          span: DUMMY_SP,
          elems: await_all
            .iter()
            .map(|(_, id)| {
              Some(ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Call(CallExpr {
                  span: DUMMY_SP,
                  callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                    FARM_REQUIRE.into(),
                    DUMMY_SP,
                  )))),
                  args: vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str(Str {
                      span: DUMMY_SP,
                      value: id.as_str().into(),
                      raw: None,
                    }))),
                  }],
                  type_args: None,
                })),
              })
            })
            .collect(),
        })),
      }],
      type_args: None,
    })),
  })
}

/// collect id and transform farmRequire("475776c7") to name__f
struct FarmRequireVisitor {
  pub name: String,
  /// id list, example: vec!["475776c7"]
  pub requires: Vec<String>,
}

impl FarmRequireVisitor {
  pub fn new(name: String) -> Self {
    Self {
      name,
      requires: vec![],
    }
  }
}

impl VisitMut for FarmRequireVisitor {
  fn visit_mut_expr(&mut self, expr: &mut farmfe_core::swc_ecma_ast::Expr) {
    if let Some(id) = try_get_farm_require_id(expr) {
      self.requires.push(id);
      *expr = Expr::Ident(Ident::new(rename_ident(&self.name).into(), DUMMY_SP));
    } else {
      expr.visit_mut_children_with(self);
    }
  }
}

fn rename_ident(name: &str) -> String {
  format!("{name}__f")
}

fn try_get_farm_require_id(expr: &Expr) -> Option<String> {
  if let Expr::Call(call_expr) = expr {
    if let Callee::Expr(box Expr::Ident(Ident { sym, .. })) = &call_expr.callee {
      if sym.to_string() == FARM_REQUIRE.to_string() && call_expr.args.len() == 1 {
        if let ExprOrSpread {
          expr: box Expr::Lit(Lit::Str(id)),
          ..
        } = &call_expr.args[0]
        {
          return Some(id.value.to_string());
        }
      }
    }
  }

  None
}
#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use farmfe_core::{
    swc_common::{FilePathMapping, SourceMap},
    swc_ecma_ast::{EsVersion, Module},
    swc_ecma_parser::Syntax,
  };
  use farmfe_toolkit::script::parse_module;

  fn parse(code: &str) -> Module {
    parse_module(
      "id",
      code,
      Syntax::Es(Default::default()),
      EsVersion::EsNext,
    )
    .unwrap()
    .ast
  }

  fn codegen(ast: &Module) -> String {
    String::from_utf8(
      farmfe_toolkit::script::codegen_module(
        ast,
        Arc::new(SourceMap::new(FilePathMapping::empty())),
        None,
        Default::default(),
        None,
      )
      .unwrap(),
    )
    .unwrap()
  }

  #[test]
  fn test_transform_async_module() {
    let input = r#"
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _sync = _interop_require_default._(farmRequire("475776c7"));
const _dep2 = _interop_require_default._(farmRequire("ea236e3d"));
console.log(_dep2.default);
    "#;

    let mut ast = parse(input);
    super::transform_async_module(&mut ast);
    let code = codegen(&ast);

    let output = r#"
const [_interop_require_default__f, _sync__f, _dep2__f] = await Promise.all([
    farmRequire("@swc/helpers/_/_interop_require_default"),
    farmRequire("475776c7"),
    farmRequire("ea236e3d")
]);
const _interop_require_default = _interop_require_default__f;
const _sync = _interop_require_default._(_sync__f);
const _dep2 = _interop_require_default._(_dep2__f);
console.log(_dep2.default);"#;

    assert_eq!(code.trim(), output.trim());
  }

  #[test]
  fn test_transform_async_module_pure() {
    let input = r#"
farmRequire("475776c7");
const b = farmRequire("12345678");
farmRequire("ea236e3d");
console.log(b);
    "#;

    let mut ast = parse(input);
    super::transform_async_module(&mut ast);
    let code = codegen(&ast);

    let output = r#"
const [, b__f, ] = await Promise.all([
    farmRequire("475776c7"),
    farmRequire("12345678"),
    farmRequire("ea236e3d")
]);
const b = b__f;
console.log(b);
    "#;

    assert_eq!(code.trim(), output.trim());
  }
}
