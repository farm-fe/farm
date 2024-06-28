use farmfe_core::{
  swc_common::{util::take::Take, Spanned, DUMMY_SP},
  swc_ecma_ast::{
    CallExpr, Callee, ComputedPropName, Expr, ExprOrSpread, Ident, KeyValueProp, Lit, MemberExpr,
    MemberProp, MetaPropExpr, MetaPropKind, Module, ObjectLit, Prop, PropName, PropOrSpread,
  },
};
use farmfe_toolkit::{
  swc_ecma_utils::ExprExt,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

struct ImportMetaURLVisitor {}

impl ImportMetaURLVisitor {
  fn is_import_meta_url(expr: &Expr) -> bool {
    if let Expr::Member(MemberExpr {
      obj:
        box Expr::MetaProp(MetaPropExpr {
          kind: MetaPropKind::ImportMeta,
          ..
        }),
      prop: MemberProp::Ident(Ident { sym, .. }),
      ..
    }) = expr
    {
      return sym == "url";
    }

    false
  }
}

impl VisitMut for ImportMetaURLVisitor {
  fn visit_mut_expr(&mut self, node: &mut Expr) {
    let mut is_replace = false;
    match node {
      Expr::New(new) => {
        if let box Expr::Ident(ident) = &new.callee {
          if ident.sym != "URL" {
            return;
          }

          if !new.args.as_ref().is_some_and(|a| {
            a.len() == 2
              && a[0].expr.is_str()
              && ImportMetaURLVisitor::is_import_meta_url(&a[1].expr)
          }) {
            return;
          }

          if let Some(args) = &mut new.args {
            let url = {
              match &args[0].expr {
                box Expr::Lit(Lit::Str(str)) => str.value.to_string(),
                box Expr::Tpl(tpl) => {
                  let mut pattern = String::from("");
                  // maybe quasis is continuous
                  let mut index = 0;

                  for item in tpl.quasis.iter() {
                    if index >= tpl.exprs.len() {
                      if item.raw.is_empty() {
                        continue;
                      }

                      if !item.raw.starts_with("/") && pattern.ends_with("**") {
                        pattern.pop();
                      }

                      pattern.push_str(&item.raw);
                    } else {
                      for exp in tpl.exprs.iter().skip(index) {
                        if item.span_hi() < exp.span_lo() {
                          if !item.raw.starts_with("/") && pattern.ends_with("**") {
                            pattern.pop();
                          }

                          pattern.push_str(&item.raw);
                          break;
                        }
                      }

                      pattern.push_str("**");
                      index += 1;
                    }
                  }

                  pattern
                }
                _ => return,
              }
            };

            is_replace = true;

            args[0] = ExprOrSpread {
              spread: None,
              expr: Box::new(Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Call(CallExpr {
                  span: DUMMY_SP,
                  callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(Expr::MetaProp(MetaPropExpr {
                      span: DUMMY_SP,
                      kind: MetaPropKind::ImportMeta,
                    })),
                    prop: MemberProp::Ident("glob".into()),
                  }))),
                  args: vec![
                    ExprOrSpread {
                      spread: None,
                      expr: Box::new(Expr::Lit(Lit::Str(url.into()))),
                    },
                    ExprOrSpread {
                      spread: None,
                      expr: Box::new(Expr::Object(ObjectLit {
                        span: DUMMY_SP,
                        props: vec![
                          PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident("eager".into()),
                            value: Box::new(Expr::Lit(Lit::Bool(true.into()))),
                          }))),
                          PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident("import".into()),
                            value: Box::new(Expr::Lit(Lit::Str("default".into()))),
                          }))),
                          PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident("query".into()),
                            value: Box::new(Expr::Lit(Lit::Str("url".into()))),
                          }))),
                        ],
                      })),
                    },
                  ],
                  type_args: None,
                })),
                prop: MemberProp::Computed(ComputedPropName {
                  span: DUMMY_SP,
                  expr: args[0].expr.take(),
                }),
              })),
            }
          }
        }
      }
      _ => {}
    }

    if !is_replace {
      node.visit_mut_children_with(self);
    }
  }
}

pub fn transform_url_with_import_meta_url(ast: &mut Module) {
  ast.visit_mut_with(&mut ImportMetaURLVisitor {});
}
