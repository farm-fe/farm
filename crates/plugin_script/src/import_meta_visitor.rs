use std::{collections::HashSet, sync::Arc};

use farmfe_core::{
  config::FARM_MODULE,
  context::CompilationContext,
  module::ModuleId,
  plugin::{PluginResolveHookParam, ResolveKind},
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, MemberExpr, MemberProp, MetaPropKind, Str,
  },
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};
use farmfe_utils::stringify_query;

/// transform `import.meta.xxx` to `module.meta.xxx`
pub struct ImportMetaVisitor {}

impl ImportMetaVisitor {
  pub fn new() -> Self {
    Self {}
  }
}

impl VisitMut for ImportMetaVisitor {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    match expr {
      Expr::MetaProp(meta_prop) => {
        if matches!(meta_prop.kind, MetaPropKind::ImportMeta) {
          // check if it's hmr accepted
          *expr = Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(Ident::new(FARM_MODULE.into(), DUMMY_SP))),
            prop: MemberProp::Ident(Ident::new("meta".into(), DUMMY_SP)),
          });
        }
      }
      _ => {
        expr.visit_mut_children_with(self);
      }
    }
  }
}

pub struct HmrAcceptedVisitor {
  pub is_hmr_self_accepted: bool,
  pub hmr_accepted_deps: HashSet<ModuleId>,

  module_id: ModuleId,
  context: Arc<CompilationContext>,
}

impl HmrAcceptedVisitor {
  pub fn new(module_id: ModuleId, context: Arc<CompilationContext>) -> Self {
    Self {
      is_hmr_self_accepted: false,
      hmr_accepted_deps: HashSet::new(),
      module_id,
      context,
    }
  }
}

impl VisitMut for HmrAcceptedVisitor {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    // detect hmr based on `module.meta.hot.accept`
    if let Expr::Call(CallExpr {
      callee:
        Callee::Expr(box Expr::Member(MemberExpr {
          obj:
            box Expr::Member(MemberExpr {
              obj:
                box Expr::Member(MemberExpr {
                  obj: box Expr::Ident(Ident { sym: module, .. }),
                  prop: MemberProp::Ident(Ident { sym: meta, .. }),
                  ..
                }),
              prop: MemberProp::Ident(Ident { sym: hot, .. }),
              ..
            }),
          prop: MemberProp::Ident(Ident { sym: accept, .. }),
          ..
        })),
      args,
      ..
    }) = expr
    {
      if &module.to_string() == FARM_MODULE
        && &meta.to_string() == "meta"
        && &hot.to_string() == "hot"
        && &accept.to_string() == "accept"
      {
        // if args is empty or the first arg is a function expression, then it's hmr self accepted
        if args.is_empty()
          || matches!(args[0], ExprOrSpread {
            expr: box Expr::Fn(..) | box Expr::Arrow(..), ..
          })
        {
          self.is_hmr_self_accepted = true;
        } else if !args.is_empty() {
          let mut resolve_and_replace_deps = |s: &mut Str| {
            // string literal
            let resolve_result = self.context.plugin_driver.resolve(
              &PluginResolveHookParam {
                source: s.value.to_string(),
                importer: Some(self.module_id.clone()),
                kind: ResolveKind::Import,
              },
              &self.context,
              &Default::default(),
            );
            if let Ok(resolved) = resolve_result {
              if let Some(resolved) = resolved {
                let id = ModuleId::new(
                  &resolved.resolved_path,
                  &stringify_query(&resolved.query),
                  &self.context.config.root,
                );
                self.hmr_accepted_deps.insert(id.clone());
                *s = Str {
                  span: DUMMY_SP,
                  value: id.to_string().into(),
                  raw: None,
                }
              }
            }
          };
          // if args is not empty and the first arg is a literal, then it's hmr accepted deps
          if let ExprOrSpread {
            expr: box Expr::Lit(Lit::Str(s)),
            ..
          } = &mut args[0]
          {
            resolve_and_replace_deps(s);
          } else if let ExprOrSpread {
            expr: box Expr::Array(arr),
            ..
          } = &mut args[0]
          {
            // array literal
            for expr in arr.elems.iter_mut() {
              if let Some(ExprOrSpread {
                expr: box Expr::Lit(Lit::Str(s)),
                ..
              }) = expr
              {
                resolve_and_replace_deps(s);
              }
            }
          }
        }
      }
    } else {
      expr.visit_mut_children_with(self);
    }
  }
}
