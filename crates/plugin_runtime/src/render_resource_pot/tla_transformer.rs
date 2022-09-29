use farmfe_core::{
  swc_common::{DUMMY_SP, Mark},
  swc_ecma_ast::{
    CallExpr, Callee, Decl, Expr, ExprStmt, Ident, MemberExpr, MemberProp, Module, ModuleItem, Stmt, ExprOrSpread,
  },
};
use farmfe_toolkit::{swc_ecma_visit::{VisitMut, VisitMutWith}, script::is_commonjs_require};

/// Transform a esmodule to farm's top level await module, for example:
/// ```js
/// // a.js is originally a esm module
/// const { b } = require('./b');
/// const { c } = require('./c');
/// console.log(b, c);
/// // after transform
/// Promise.all([b, c]).then(([b, c]) => {
///  console.log(b, c);
/// });
/// ```
pub struct TLATransformer {
  unresolved_mark: Mark,
}

impl TLATransformer {
  pub fn new(unresolved_mark: Mark) -> Self {
    Self {
      unresolved_mark
    }
  }
}

impl VisitMut for TLATransformer {
  fn visit_mut_module(&mut self, module: &mut Module) {
    let mut require_assign_key_values = vec![];
    // the position of the first non require statement
    let mut non_require_pos = 0;

    for (index, item) in module.body.iter().enumerate() {
      if let ModuleItem::Stmt(Stmt::Decl(Decl::Var(var))) = item {
        for decl in &var.decls {
          if let Some(init) = &decl.init {
            // TODO recursively check if the init is a require call
            if let Expr::Call(call_expr) = &**init {
              if is_commonjs_require(self.unresolved_mark, &call_expr) {
                require_assign_key_values.push((decl.name.clone(), ExprOrSpread {
                  spread: None,
                  expr: call_expr.args[0].expr.clone(),
                }));
              }
            }
          }
        }
      } else {
        non_require_pos = index;
        // break if not a `var xx = require('xx')` declaration
        break;
      }
    }

    // remove all require statements
    for _ in 0..non_require_pos {
      module.body.remove(0);
    }

    let args = require_assign_key_values.iter().map(|(_, value)| value.clone()).collect();

    module.body.insert(
      0,
      ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        // TODO construct this complex Promse.all expression
        expr: Box::new(Expr::Call(CallExpr {
          span: DUMMY_SP,
          callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(Ident::new("Promise".into(), DUMMY_SP))),
            prop: MemberProp::Ident(Ident::new("all".into(), DUMMY_SP)),
          }))),
          args,
          type_args: None,
        })),
      })),
    );
  }
}
