use farmfe_core::{
  swc_common::{
    comments::{Comments, SingleThreadedComments},
    util::take::Take,
    Spanned, DUMMY_SP,
  },
  swc_ecma_ast::{
    CallExpr, Callee, ComputedPropName, Expr, ExprOrSpread, Ident, KeyValueProp, Lit, MemberExpr,
    MemberProp, MetaPropExpr, MetaPropKind, Module, ObjectLit, Prop, PropName, PropOrSpread,
  },
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};
use farmfe_utils::is_skip_action_by_comment;

fn normalized_glob_pattern(pattern: String) -> String {
  let mut pattern_builder: Vec<String> = vec![];
  let path_split = pattern
    .split('/')
    .filter(|str| str != &"")
    .collect::<Vec<_>>();

  for item in path_split {
    let mut file_pattern = String::with_capacity(item.len());

    for ch in item.chars() {
      if file_pattern.is_empty() {
        file_pattern.push(ch);
        continue;
      }

      match ch {
        '*' => {
          // x* push * => x*
          if file_pattern.ends_with('*') && file_pattern.len() > 1 {
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

  if pattern.starts_with('/') {
    pattern_builder.insert(0, "".to_string());
  }

  pattern_builder.join("/")
}

// transform `new URL("url", import.meta.url)` to `new URL(import.meta.glob('url', { eager: true, import: 'default', query: 'url' }), import.meta.url)`
struct ImportMetaURLVisitor<'a> {
  comments: &'a SingleThreadedComments,
}

impl<'a> ImportMetaURLVisitor<'a> {
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

  fn transform_url(&self, node: &mut Expr) -> Option<()> {
    let mut is_replace = false;

    let Expr::New(new) = node else { return None };

    let box Expr::Ident(ident) = &new.callee else {
      return None;
    };

    if ident.sym != "URL" {
      return None;
    }

    if !new.args.as_ref().is_some_and(|a| {
      a.len() == 2
        && matches!(a[0].expr.as_ref(), Expr::Lit(_) | Expr::Tpl(_))
        && ImportMetaURLVisitor::is_import_meta_url(&a[1].expr)
    }) {
      return None;
    }

    if let Some(args) = &mut new.args {
      let url = {
        // skip transform when contain skip comment
        if self
          .comments
          .get_leading(args[0].span_lo())
          .is_some_and(|c| {
            c.iter()
              .any(|item| is_skip_action_by_comment(item.text.as_str()))
          })
        {
          return None;
        };

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

                pattern_builder.push_str("**");
                index += 1;
              }
            }

            pattern_builder = normalized_glob_pattern(pattern_builder);

            pattern_builder
          }
          _ => return None,
        }
      };

      if url.is_empty() || url.starts_with('/') {
        return None;
      }

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

    if is_replace {
      return Some(());
    }

    None
  }
}

impl<'a> VisitMut for ImportMetaURLVisitor<'a> {
  fn visit_mut_expr(&mut self, node: &mut Expr) {
    if self.transform_url(node).is_none() {
      node.visit_mut_children_with(self);
    };
  }
}

pub fn transform_url_with_import_meta_url(ast: &mut Module, comments: &SingleThreadedComments) {
  ast.visit_mut_with(&mut ImportMetaURLVisitor { comments });
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
