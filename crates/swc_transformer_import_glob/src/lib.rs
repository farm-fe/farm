//! Transform `import.meta.glob` like Vite. See https://vitejs.dev/guide/features.html#glob-import
//! for example:
//! ```js
//! const modules = import.meta.glob('./dir/*.js', { eager: true })
//! ```
//! will be transformed to:
//! ```js
//！import * as __glob__0_0 from './dir/foo.js'
//！import * as __glob__0_1 from './dir/bar.js'
//！const modules = {
//！  './dir/foo.js': __glob__0_0,
//！  './dir/bar.js': __glob__0_1,
// }
//! ````
#![feature(box_patterns)]

use std::{collections::HashMap, path::PathBuf};

use relative_path::RelativePath;
use swc_ecma_ast::{
  CallExpr, Callee, Expr, ExprOrSpread, Ident, KeyValueProp, Lit, MemberExpr, MemberProp,
  MetaPropExpr, MetaPropKind, ObjectLit, Prop,
};

use swc_ecma_visit::{VisitMut, VisitMutWith};

pub struct ImportGlobVisitor {
  // './dir/*.js' of `import.meta.glob('./dir/*.js', { as: 'raw', eager: true })`
  source: Option<String>,
  eager: Option<bool>,
  // e.g. 'raw' of `import.meta.glob('./dir/*.js', { as: 'raw', eager: true })`
  glob_import_as: Option<String>,
  cur_dir: String,
  pub errors: Vec<String>,
}

impl ImportGlobVisitor {
  pub fn new(cur_dir: String) -> Self {
    Self {
      source: None,
      eager: None,
      glob_import_as: None,
      cur_dir,
      errors: vec![],
    }
  }
}

impl VisitMut for ImportGlobVisitor {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    match expr {
      Expr::Call(CallExpr {
        callee:
          Callee::Expr(box Expr::Member(MemberExpr {
            obj:
              box Expr::MetaProp(MetaPropExpr {
                kind: MetaPropKind::ImportMeta,
                ..
              }),
            prop: MemberProp::Ident(Ident { sym, .. }),
            ..
          })),
        args,
        ..
      }) => {
        if sym.to_string() == "glob".to_string() && args.len() > 0 {
          if let Some(source) = get_string_literal(&args[0]) {
            self.source = Some(source);

            // get arguments from args[1]
            if args.len() > 1 {
              if let Some(mut options) = get_object_literal(&args[2]) {
                if options.contains_key("as") {
                  self.glob_import_as = Some(options.remove("as").unwrap());
                }
                if options.contains_key("eager") {
                  let eager = if options.remove("eager").unwrap() == "true".to_string() {
                    true
                  } else {
                    false
                  };
                  self.eager = Some(eager);
                }
              }
            }

            // search source using glob
            let source = self.source.as_ref().unwrap();

            let paths = if PathBuf::from(source).is_absolute() {
              glob::glob(source)
            } else {
              let rel_source = RelativePath::new(source);
              let abs_source = rel_source
                .to_logical_path(&self.cur_dir)
                .to_string_lossy()
                .to_string();
              glob::glob(&abs_source)
            };
            match paths {
              Ok(paths) => {
                for entry in paths {
                  match entry {
                    Ok(file) => {
                      // deal with eager
                      if self.eager.unwrap_or(false) {
                      } else {
                      }

                      // deal with as
                      if let Some(glob_import_as) = &self.glob_import_as {
                        if glob_import_as == &"raw".to_string() {}
                      }
                    }
                    Err(err) => {
                      self
                        .errors
                        .push(format!("Error when glob {source}: {err:?}"));
                    }
                  }
                }
              }
              Err(err) => {
                self
                  .errors
                  .push(format!("Error when glob {source}: {err:?}"));
              }
            }
          }
        }
      }
      _ => {
        expr.visit_mut_children_with(self);
      }
    }
  }
}

fn get_string_literal(expr: &ExprOrSpread) -> Option<String> {
  match &expr.expr {
    box Expr::Lit(Lit::Str(str)) => Some(str.value.to_string()),
    _ => None,
  }
}

fn get_object_literal(expr: &ExprOrSpread) -> Option<HashMap<String, String>> {
  match &expr.expr {
    box Expr::Object(ObjectLit { props, .. }) => {
      let mut result = HashMap::new();

      for prop in props {
        match prop {
          swc_ecma_ast::PropOrSpread::Spread(_) => {}
          swc_ecma_ast::PropOrSpread::Prop(box Prop::KeyValue(KeyValueProp { key, value })) => {
            let k = match key {
              swc_ecma_ast::PropName::Ident(i) => Some(i.sym.to_string()),
              swc_ecma_ast::PropName::Str(str) => Some(str.value.to_string()),
              swc_ecma_ast::PropName::Num(_)
              | swc_ecma_ast::PropName::Computed(_)
              | swc_ecma_ast::PropName::BigInt(_) => None,
            };

            let v = match value {
              box Expr::Lit(Lit::Str(str)) => Some(str.value.to_string()),
              box Expr::Lit(Lit::Bool(b)) => Some(if b.value {
                "true".to_string()
              } else {
                "false".to_string()
              }),
              _ => None,
            };

            if k.is_some() && v.is_some() {
              result.insert(k.unwrap(), v.unwrap());
            }
          }
          _ => {}
        }
      }

      if result.len() > 0 {
        Some(result)
      } else {
        None
      }
    }
    _ => None,
  }
}
