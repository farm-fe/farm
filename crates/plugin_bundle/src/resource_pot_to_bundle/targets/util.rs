use farmfe_core::{
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{ArrayLit, CallExpr, Callee, Expr, ExprOrSpread, Ident, ObjectLit, PropOrSpread},
};
use farmfe_toolkit::script::module2cjs::RuntimeCalleeAllocator;

use crate::resource_pot_to_bundle::{Polyfill, ShareBundleContext, SimplePolyfill};

pub fn wrap_require_default(
  expr: Box<Expr>,
  polyfill: &mut SimplePolyfill,
  ctx: &ShareBundleContext,
) -> Box<Expr> {
  polyfill.add(Polyfill::InteropRequireDefault);
  Box::new(Expr::Call(farmfe_core::swc_ecma_ast::CallExpr {
    span: DUMMY_SP,
    callee: farmfe_core::swc_ecma_ast::Callee::Expr(ctx.allocator.import_default_callee()),
    args: vec![farmfe_core::swc_ecma_ast::ExprOrSpread { spread: None, expr }],
    type_args: None,
    ctxt: SyntaxContext::empty(),
  }))
}

///
/// ```ts
/// import * as bar from "foo";
/// ```
///
pub fn wrap_require_wildcard(
  expr: Box<Expr>,
  polyfill: &mut SimplePolyfill,
  ctx: &ShareBundleContext,
) -> Box<Expr> {
  polyfill.add(Polyfill::Wildcard);
  Box::new(Expr::Call(farmfe_core::swc_ecma_ast::CallExpr {
    span: DUMMY_SP,
    callee: farmfe_core::swc_ecma_ast::Callee::Expr(ctx.allocator.import_namespace_callee()),
    args: vec![farmfe_core::swc_ecma_ast::ExprOrSpread { spread: None, expr }],
    type_args: None,
    ctxt: SyntaxContext::empty(),
  }))
}

///
/// ```ts
/// export * from "x";
/// ```
///
pub fn wrap_export_star(
  args: Vec<ExprOrSpread>,
  polyfill: &mut SimplePolyfill,
  ctx: &ShareBundleContext,
) -> Box<Expr> {
  polyfill.add(Polyfill::ExportStar);
  Box::new(Expr::Call(CallExpr {
    span: DUMMY_SP,
    callee: Callee::Expr(ctx.allocator.export_star_callee()),
    args,
    type_args: None,
    ctxt: SyntaxContext::empty(),
  }))
}

pub fn wrap_commonjs(args: Vec<ExprOrSpread>, polyfill: &mut SimplePolyfill) -> Box<Expr> {
  polyfill.add(Polyfill::WrapCommonJs);
  Box::new(Expr::Call(CallExpr {
    span: DUMMY_SP,
    callee: Callee::Expr(Box::new(Expr::Ident(("__commonJs").into()))),
    args,
    type_args: None,
    ctxt: SyntaxContext::empty(),
  }))
}

pub fn create_merge_namespace(
  props: Vec<PropOrSpread>,
  commonjs_fns: Vec<Ident>,
  reexport_namespace: Vec<Ident>,
  polyfill: &mut SimplePolyfill,
) -> Box<Expr> {
  polyfill.add(Polyfill::MergeNamespace);
  Box::new(Expr::Call(CallExpr {
    ctxt: SyntaxContext::empty(),
    span: DUMMY_SP,
    callee: Callee::Expr(Box::new(Expr::Ident(Ident::from("_mergeNamespaces")))),
    args: vec![
      // static
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Object(ObjectLit {
          span: DUMMY_SP,
          props,
        })),
      },
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Array(ArrayLit {
          span: DUMMY_SP,
          elems: commonjs_fns
            .into_iter()
            .map(|ident| {
              Some(ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Call(CallExpr {
                  ctxt: SyntaxContext::empty(),
                  span: DUMMY_SP,
                  callee: Callee::Expr(Box::new(Expr::Ident(ident))),
                  args: vec![],
                  type_args: None,
                })),
              })
            })
            .chain(reexport_namespace.into_iter().map(|ns| {
              Some(ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Ident(ns)),
              })
            }))
            .collect(),
        })),
      },
    ],
    type_args: None,
  }))
}

pub struct ShareBundleRuntimeCalleeAllocator {}

impl ShareBundleRuntimeCalleeAllocator {
  pub fn new() -> Self {
    Self {}
  }
}

impl RuntimeCalleeAllocator for ShareBundleRuntimeCalleeAllocator {
  fn import_default_callee(&self) -> Box<Expr> {
    Box::new(Expr::Ident("_interop_require_default".into()))
  }

  fn import_namespace_callee(&self) -> Box<Expr> {
    Box::new(Expr::Ident("_interop_require_wildcard".into()))
  }

  fn export_star_callee(&self) -> Box<Expr> {
    Box::new(Expr::Ident("_export_star".into()))
  }
}
