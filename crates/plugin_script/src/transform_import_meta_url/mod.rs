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

fn normalized_glob_pattern(pattern: String) -> String {
  let mut pattern_builder: Vec<String> = vec![];
  let stack = pattern
    .split("/")
    .into_iter()
    .filter(|str| str != &"")
    .collect::<Vec<_>>();

  for item in stack {
    let mut file_pattern = String::with_capacity(item.len());

    for ch in item.chars() {
      if file_pattern.is_empty() {
        file_pattern.push(ch);
        continue;
      }

      match ch {
        '*' => {
          // x* push * => x*
          if file_pattern.ends_with("*") && file_pattern.len() > 1 {
            continue;
          }

          file_pattern.push(ch);
        }

        _ => {
          // ** push - => *-
          if ch != '*' && file_pattern.ends_with("**") {
            file_pattern.pop();
          }

          file_pattern.push(ch);
        }
      }
    }

    // ./** push **
    if file_pattern == "**" && pattern_builder.last().is_some_and(|s| s == "**") {
      continue;
    }

    if file_pattern.is_empty() {
      continue;
    }

    pattern_builder.push(file_pattern);
  }

  if pattern.starts_with("/") {
    pattern_builder.insert(0, "".to_string());
  }

  return pattern_builder.join("/");
}

// transform `new URL("url", import.meta.url)` to `import.meta.glob('url', { eager: true, import: 'default', query: 'url' })`
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
                  let mut pattern_builder = String::new();
                  // maybe quasis is continuous
                  let mut index = 0;

                  for item in tpl.quasis.iter() {
                    if index >= tpl.exprs.len() {
                      if item.raw.is_empty() {
                        continue;
                      }

                      pattern_builder.push_str(&item.raw);
                    } else {
                      for exp in tpl.exprs.iter().skip(index) {
                        if item.span_hi() < exp.span_lo() {
                          pattern_builder.push_str(&item.raw);
                          break;
                        }
                      }

                      pattern_builder.push_str(&"**");
                      index += 1;
                    }
                  }

                  pattern_builder = normalized_glob_pattern(pattern_builder);

                  pattern_builder
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

mod tests {

  #[test]
  fn test_normalized_glob_pattern() {
    use super::normalized_glob_pattern;
    assert_eq!(
      normalized_glob_pattern("./**/*/**".to_string()),
      "./**/*/**"
    );
    assert_eq!(normalized_glob_pattern("./**/**/**".to_string()), "./**");
    assert_eq!(
      normalized_glob_pattern("./foo/**-**".to_string()),
      "./foo/*-*"
    );
    assert_eq!(
      normalized_glob_pattern("./foo/*-**".to_string()),
      "./foo/*-*"
    );
    assert_eq!(
      normalized_glob_pattern("./foo/*-*".to_string()),
      "./foo/*-*"
    );
    assert_eq!(
      normalized_glob_pattern("./foo/***/*****".to_string()),
      "./foo/**"
    );

    assert_eq!(
      normalized_glob_pattern("/foo/bar/**".to_string()),
      "/foo/bar/**"
    );
  }
}
