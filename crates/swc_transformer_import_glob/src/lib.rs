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

use std::collections::HashMap;
use std::path::Component;
use std::path::PathBuf;

use farmfe_core::config::AliasItem;
use farmfe_core::config::StringOrRegex;
use farmfe_core::regex;
use farmfe_core::relative_path::RelativePath;
use farmfe_core::swc_common::DUMMY_SP;
use farmfe_core::swc_ecma_ast::{
  self, ArrayLit, ArrowExpr, BindingIdent, BlockStmtOrExpr, CallExpr, Callee, Expr, ExprOrSpread,
  Ident, Import, KeyValueProp, Lit, MemberExpr, MemberProp, MetaPropExpr, MetaPropKind,
  Module as SwcModule, ModuleItem, ObjectLit, Pat, Prop, PropOrSpread,
};
use farmfe_core::wax::Glob;

use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};
use farmfe_utils::relative;

const REGEX_PREFIX: &str = "$__farm_regex:";

pub fn transform_import_meta_glob(
  ast: &mut SwcModule,
  root: String,
  cur_dir: String,
  alias: &Vec<AliasItem>,
) -> farmfe_core::error::Result<()> {
  let mut visitor = ImportGlobVisitor::new(cur_dir, root, alias);
  ast.visit_mut_with(&mut visitor);

  if visitor.errors.len() > 0 {
    return Err(farmfe_core::error::CompilationError::GenericError(
      visitor.errors.join("\n"),
    ));
  }

  // insert import statements
  for (index, meta_info) in visitor.import_globs.into_iter().enumerate() {
    for (glob_index, globed_source) in meta_info.globed_sources.into_iter().enumerate() {
      let globed_source = if let Some(query) = &meta_info.query {
        format!("{globed_source}?{query}")
      } else {
        globed_source
      };
      if let Some(import_glob_as) = &meta_info.glob_import_as {
        // variable is &String and can be automatically deref as &str
        // so remove `&` and `to_string()` can be quicker and more readable
        if import_glob_as == "raw" {
          // do nothing
        } else if import_glob_as == "url" {
          ast.body.insert(
            0,
            farmfe_core::swc_ecma_ast::ModuleItem::ModuleDecl(
              farmfe_core::swc_ecma_ast::ModuleDecl::Import(
                farmfe_core::swc_ecma_ast::ImportDecl {
                  span: DUMMY_SP,
                  // import __glob__0_0 from './dir/foo.js?url'
                  specifiers: vec![farmfe_core::swc_ecma_ast::ImportSpecifier::Default(
                    farmfe_core::swc_ecma_ast::ImportDefaultSpecifier {
                      span: DUMMY_SP,
                      local: farmfe_core::swc_ecma_ast::Ident::new(
                        format!("__glob__{index}_{glob_index}").into(),
                        DUMMY_SP,
                      ),
                    },
                  )],
                  src: Box::new(farmfe_core::swc_ecma_ast::Str {
                    span: DUMMY_SP,
                    value: format!("{globed_source}?url").into(),
                    raw: None,
                  }),
                  type_only: false,
                  with: None,
                  phase: Default::default(),
                },
              ),
            ),
          );
        } else {
          return Err(farmfe_core::error::CompilationError::GenericError(format!(
            "Error when glob {source:?}: unknown as: `{import_glob_as}`",
            source = meta_info.sources,
          )));
        }
      } else if meta_info.eager {
        if let Some(import) = &meta_info.import {
          if import == "default" {
            ast.body.insert(
              0,
              create_eager_default_import(index, glob_index, &globed_source),
            );
          } else {
            ast.body.insert(
              0,
              create_eager_named_import(index, glob_index, import, &globed_source),
            );
          }
        } else {
          ast.body.insert(
            0,
            create_eager_namespace_import(index, glob_index, &globed_source),
          );
        }
      }
    }
  }

  Ok(())
}

/// import { <import> as __glob__0_0 } from './dir/foo.js'
fn create_eager_named_import(
  index: usize,
  glob_index: usize,
  import: &str,
  globed_source: &str,
) -> ModuleItem {
  farmfe_core::swc_ecma_ast::ModuleItem::ModuleDecl(farmfe_core::swc_ecma_ast::ModuleDecl::Import(
    farmfe_core::swc_ecma_ast::ImportDecl {
      span: DUMMY_SP,
      specifiers: vec![farmfe_core::swc_ecma_ast::ImportSpecifier::Named(
        farmfe_core::swc_ecma_ast::ImportNamedSpecifier {
          span: DUMMY_SP,
          local: farmfe_core::swc_ecma_ast::Ident::new(
            format!("__glob__{index}_{glob_index}").into(),
            DUMMY_SP,
          ),
          imported: Some(farmfe_core::swc_ecma_ast::ModuleExportName::Ident(
            Ident::new(import.into(), DUMMY_SP),
          )),
          is_type_only: false,
        },
      )],
      src: Box::new(farmfe_core::swc_ecma_ast::Str {
        span: DUMMY_SP,
        value: globed_source.into(),
        raw: None,
      }),
      type_only: false,
      with: None,
      phase: Default::default(),
    },
  ))
}

fn create_eager_namespace_import(
  index: usize,
  glob_index: usize,
  globed_source: &str,
) -> ModuleItem {
  farmfe_core::swc_ecma_ast::ModuleItem::ModuleDecl(farmfe_core::swc_ecma_ast::ModuleDecl::Import(
    farmfe_core::swc_ecma_ast::ImportDecl {
      span: DUMMY_SP,
      specifiers: vec![farmfe_core::swc_ecma_ast::ImportSpecifier::Namespace(
        farmfe_core::swc_ecma_ast::ImportStarAsSpecifier {
          span: DUMMY_SP,
          local: farmfe_core::swc_ecma_ast::Ident::new(
            format!("__glob__{index}_{glob_index}").into(),
            DUMMY_SP,
          ),
        },
      )],
      src: Box::new(farmfe_core::swc_ecma_ast::Str {
        span: DUMMY_SP,
        value: globed_source.into(),
        raw: None,
      }),
      type_only: false,
      with: None,
      phase: Default::default(),
    },
  ))
}

fn create_eager_default_import(index: usize, glob_index: usize, globed_source: &str) -> ModuleItem {
  farmfe_core::swc_ecma_ast::ModuleItem::ModuleDecl(farmfe_core::swc_ecma_ast::ModuleDecl::Import(
    farmfe_core::swc_ecma_ast::ImportDecl {
      span: DUMMY_SP,
      specifiers: vec![farmfe_core::swc_ecma_ast::ImportSpecifier::Default(
        farmfe_core::swc_ecma_ast::ImportDefaultSpecifier {
          span: DUMMY_SP,
          local: farmfe_core::swc_ecma_ast::Ident::new(
            format!("__glob__{index}_{glob_index}").into(),
            DUMMY_SP,
          ),
        },
      )],
      src: Box::new(farmfe_core::swc_ecma_ast::Str {
        span: DUMMY_SP,
        value: globed_source.into(),
        raw: None,
      }),
      type_only: false,
      with: None,
      phase: Default::default(),
    },
  ))
}

#[derive(Debug)]
pub struct ImportMetaGlobInfo {
  /// './dir/*.js' of `import.meta.glob('./dir/*.js', { as: 'raw', eager: true })`
  pub sources: Vec<String>,
  pub eager: bool,
  /// e.g. 'raw' of `import.meta.glob('./dir/*.js', { as: 'raw', eager: true })`
  pub glob_import_as: Option<String>,
  /// e.g. './dir/foo.js' of `import.meta.glob('./dir/*.js', { as: 'raw', eager: true })`
  pub globed_sources: Vec<String>,
  /// e.g. 'default' of `import.meta.glob('./dir/*.js', { import: 'default' })`
  pub import: Option<String>,
  /// e.g. 'foo' of `import.meta.glob('./dir/*.js', { query: { foo: 'bar', bar: true } })`
  pub query: Option<String>,
}

pub struct ImportGlobVisitor<'a> {
  import_globs: Vec<ImportMetaGlobInfo>,
  cur_dir: String,
  root: String,
  alias: &'a Vec<AliasItem>,
  pub errors: Vec<String>,
}

impl<'a> ImportGlobVisitor<'a> {
  pub fn new(cur_dir: String, root: String, alias: &'a Vec<AliasItem>) -> Self {
    Self {
      import_globs: vec![],
      cur_dir,
      root,
      errors: vec![],
      alias,
    }
  }

  fn create_import_glob_info(sources: Vec<String>, args: &Vec<ExprOrSpread>) -> ImportMetaGlobInfo {
    let mut import_glob_info = ImportMetaGlobInfo {
      sources,
      eager: false,
      glob_import_as: None,
      globed_sources: vec![],
      import: None,
      query: None,
    };

    // get arguments from args[1]
    if args.len() > 1 {
      if let Some(mut options) = get_object_literal(&args[1]) {
        if options.contains_key("as") {
          import_glob_info.glob_import_as = Some(options.remove("as").unwrap());
        }
        if options.contains_key("eager") {
          let eager = if options.remove("eager").unwrap() == "true".to_string() {
            true
          } else {
            false
          };
          import_glob_info.eager = eager;
        }
        if options.contains_key("import") {
          let import = options.remove("import").unwrap();
          import_glob_info.import = Some(import);
        }
        if options.contains_key("query") {
          let query = options.remove("query").unwrap();
          import_glob_info.query = Some(query);
        }
      }
    }

    import_glob_info
  }

  // fn try_alias(&self, source: &str) -> String {
  //   let (source, negative) = if source.starts_with('!') {
  //     (&source[1..], true)
  //   } else {
  //     (source, false)
  //   };
  //   let mut result = source.to_string();
  //   // sort the alias by length, so that the longest alias will be matched first
  //   let mut alias_list: Vec<_> = self.alias.keys().collect();
  //   alias_list.sort_by(|a, b| b.len().cmp(&a.len()));

  //   for alias in alias_list {
  //     let replaced = self.alias.get(alias).unwrap();

  //     // try regex alias first
  //     if let Some(alias) = alias.strip_prefix(REGEX_PREFIX) {
  //       let regex = regex::Regex::new(alias).unwrap();
  //       if regex.is_match(source) {
  //         let replaced = regex.replace(source, replaced.as_str()).to_string();
  //         result = replaced;
  //         break;
  //       }
  //     }

  //     if alias.ends_with('$') && source == alias.trim_end_matches('$') {
  //       result = replaced.to_string();
  //       break;
  //     } else if !alias.ends_with('$') && source.starts_with(alias) {
  //       let source_left = RelativePath::new(source.trim_start_matches(alias));
  //       let new_source = source_left.to_logical_path(replaced);

  //       result = if new_source.is_absolute() {
  //         format!(
  //           "/{}",
  //           relative(&self.root, &new_source.to_string_lossy().to_string())
  //         )
  //       } else {
  //         new_source.to_string_lossy().to_string()
  //       };
  //       break;
  //     }
  //   }

  //   if negative {
  //     format!("!{}", result)
  //   } else {
  //     result
  //   }
  // }

  fn try_alias(&self, source: &str) -> String {
    let (source, negative) = if source.starts_with('!') {
      (&source[1..], true)
    } else {
      (source, false)
    };

    let result = self
      .alias
      .iter()
      .find_map(|alias_item| {
        match alias_item {
          AliasItem::Complex { find, replacement } => match find {
            StringOrRegex::String(alias) => {
              if let Some(regex_str) = alias.strip_prefix(REGEX_PREFIX) {
                if let Ok(regex) = regex::Regex::new(regex_str) {
                  if regex.is_match(source) {
                    return Some(regex.replace(source, replacement.as_str()).to_string());
                  }
                }
              } else if alias.ends_with('$') && source == alias.trim_end_matches('$') {
                return Some(replacement.to_string());
              } else if !alias.ends_with('$') && source.starts_with(alias) {
                let source_left = RelativePath::new(source.trim_start_matches(alias));
                let new_source = source_left.to_logical_path(replacement);

                return Some(if new_source.is_absolute() {
                  format!(
                    "/{}",
                    relative(&self.root, &new_source.to_string_lossy().to_string())
                  )
                } else {
                  new_source.to_string_lossy().to_string()
                });
              }
            }
            StringOrRegex::Regex(regex) => {
              if regex.is_match(source) {
                return Some(regex.replace(source, replacement.as_str()).to_string());
              }
            }
          }, // AliasItem::Simple(_) => {}
        }
        None
      })
      .unwrap_or_else(|| source.to_string());

    if negative {
      format!("!{result}")
    } else {
      result
    }
  }

  /// Glob the sources and filter negative sources, return globs relative paths
  fn glob_and_filter_sources(&mut self, sources: &Vec<String>) -> HashMap<String, String> {
    let mut paths = vec![];

    for source in sources {
      let mut negative = false;

      let source = if source.starts_with("!") {
        negative = true;
        &source[1..]
      } else {
        &source[..]
      };

      let source = if !source.starts_with('.') && !source.starts_with('/') {
        format!("./{source}")
      } else {
        source.to_string()
      };

      // relative to root when source starts with '/'.
      // and alias
      let rel_path = if source.starts_with('/') {
        let abs_path = RelativePath::new(&source[1..]).to_logical_path(&self.root);
        relative(&self.cur_dir, &abs_path.to_string_lossy())
      } else {
        source.clone()
      };
      let rel_source = relative(
        &self.root,
        &RelativePath::new(&rel_path)
          .to_logical_path(&self.cur_dir)
          .to_string_lossy(),
      );
      // wax::Glob does not support ../, so we need to convert it to relative path
      let (root, rel_source) = if rel_source.starts_with("../") {
        let mut root_path = PathBuf::from(&self.root);
        let rel_source_path = PathBuf::from(&rel_source);

        for comp in rel_source_path.components() {
          match comp {
            Component::ParentDir => {
              root_path.pop();
            }
            _ => {}
          }
        }

        (
          root_path.to_string_lossy().to_string(),
          rel_source.replace("../", ""),
        )
      } else {
        (self.root.clone(), rel_source.clone())
      };

      let glob = Glob::new(&rel_source);
      match glob {
        Ok(glob) => {
          let p = glob
            .walk(&root)
            // filter out directory
            .filter(|p| {
              if p.is_err() {
                true
              } else {
                p.as_ref().is_ok_and(|f| !f.path().is_dir())
              }
            })
            .map(|p| {
              (
                source.clone(),
                p.map(|p| p.path().to_string_lossy().to_string()),
              )
            })
            .collect::<Vec<_>>();
          paths.push((negative, p));
        }
        Err(err) => {
          self
            .errors
            .push(format!("Error when glob {source}: {err:?}"));
        }
      }
    }

    let mut filtered_paths = HashMap::new();

    for (negative, path) in paths {
      for (source, entry) in path {
        match entry {
          Ok(file) => {
            // if source starts with '/', we need make source relative to root, otherwise, relative to cur_dir
            let source: String = if source.starts_with('/') {
              let rel_source = relative(&self.root, &file);

              if !rel_source.starts_with('/') {
                format!("/{rel_source}")
              } else {
                rel_source
              }
            } else {
              let rel_source = relative(&self.cur_dir, &file);

              if !rel_source.starts_with('.') {
                format!("./{rel_source}")
              } else {
                rel_source
              }
            };

            let mut relative_file = relative(&self.cur_dir, &file);

            if !relative_file.starts_with('.') {
              relative_file = format!("./{relative_file}");
            }

            if negative && filtered_paths.contains_key(&relative_file) {
              filtered_paths.remove(&relative_file);
            } else if !negative {
              filtered_paths.insert(relative_file, source);
            }
          }
          Err(err) => {
            self
              .errors
              .push(format!("Error when glob {sources:?}: {err:?}"));
          }
        }
      }
    }

    filtered_paths
  }

  fn deal_with_import_as(
    &mut self,
    glob_import_as: &str,
    relative_file: &str,
    source: &str,
    cur_index: usize,
    entry_index: usize,
    sources: &Vec<String>,
  ) -> Option<(String, Box<Expr>)> {
    if glob_import_as == "raw" {
      let file = RelativePath::new(&relative_file).to_logical_path(&self.cur_dir);
      let content = std::fs::read_to_string(file).unwrap();
      Some((
        source.to_string(),
        Box::new(Expr::Lit(Lit::Str(swc_ecma_ast::Str {
          span: DUMMY_SP,
          value: content.into(),
          raw: None,
        }))),
      ))
    } else if glob_import_as == "url" {
      // add "./dir/foo.js": __glob__0_0
      Some((
        source.to_string(),
        Box::new(Expr::Ident(Ident::new(
          format!("__glob__{cur_index}_{entry_index}").into(),
          DUMMY_SP,
        ))),
      ))
    } else {
      self.errors.push(format!(
        "Error when glob {sources:?}: unknown as: `{glob_import_as:?}`",
      ));
      None
    }
  }

  /// add "./dir/foo.js": () => import('./dir/foo.js')
  fn deal_with_non_eager(
    &self,
    relative_file: &str,
    source: &str,
    import: &Option<String>,
  ) -> (String, Box<Expr>) {
    let import_call_expr = Box::new(Expr::Call(CallExpr {
      span: DUMMY_SP,
      callee: Callee::Import(Import {
        span: DUMMY_SP,
        phase: Default::default(),
      }),
      args: vec![ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Lit(Lit::Str(swc_ecma_ast::Str {
          span: DUMMY_SP,
          value: relative_file.into(),
          raw: None,
        }))),
      }],
      type_args: None,
    }));

    if let Some(import) = import.as_ref() {
      // () => import('./dir/foo.js').then((m) => m.setup)
      (
        source.to_string(),
        Box::new(Expr::Arrow(ArrowExpr {
          span: DUMMY_SP,
          params: vec![],
          body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
              span: DUMMY_SP,
              obj: import_call_expr,
              prop: MemberProp::Ident(Ident::new("then".into(), DUMMY_SP)),
            }))),
            args: vec![ExprOrSpread {
              spread: None,
              expr: Box::new(Expr::Arrow(ArrowExpr {
                span: DUMMY_SP,
                params: vec![Pat::Ident(BindingIdent {
                  id: Ident::new("m".into(), DUMMY_SP),
                  type_ann: None,
                })],
                body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Member(MemberExpr {
                  span: DUMMY_SP,
                  obj: Box::new(Expr::Ident(Ident::new("m".into(), DUMMY_SP))),
                  prop: MemberProp::Ident(Ident::new(import.as_str().into(), DUMMY_SP)),
                })))),
                is_async: false,
                is_generator: false,
                type_params: None,
                return_type: None,
              })),
            }],
            type_args: None,
          })))),
          is_async: false,
          is_generator: false,
          type_params: None,
          return_type: None,
        })),
      )
    } else {
      (
        source.to_string(),
        Box::new(Expr::Arrow(ArrowExpr {
          span: DUMMY_SP,
          params: vec![],
          body: Box::new(BlockStmtOrExpr::Expr(import_call_expr)),
          is_async: false,
          is_generator: false,
          type_params: None,
          return_type: None,
        })),
      )
    }
  }
}

impl<'a> VisitMut for ImportGlobVisitor<'a> {
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
        if *sym == *"glob" && !args.is_empty() {
          if let Some(original_sources) = get_string_literal(&args[0]) {
            let mut sources = vec![];

            for source in original_sources {
              let source = self.try_alias(&source);

              if !source.starts_with('.')
                && !source.starts_with('/')
                && !source.starts_with('!')
                && !source.starts_with('*')
              {
                self
                  .errors
                  .push(format!("Error when glob {source}: source must be relative path. e.g. './dir/*.js' or '/dir/*.js'(relative to root) or '!/dir/*.js'(exclude) or '!**/bar.js'(exclude) or '**/*.js'(relative to current dir)"));
                return;
              }

              sources.push(source);
            }

            let cur_index = self.import_globs.len();
            let mut import_glob_info = Self::create_import_glob_info(sources, args);
            // search source using glob
            let sources = &import_glob_info.sources;
            let filtered_paths = self.glob_and_filter_sources(sources);
            let mut filtered_paths = filtered_paths.into_iter().collect::<Vec<_>>();
            filtered_paths.sort();

            let mut props = vec![];

            for (entry_index, (relative_file, source)) in filtered_paths.into_iter().enumerate() {
              // deal with as
              if let Some(glob_import_as) = &import_glob_info.glob_import_as {
                if let Some(prop) = self.deal_with_import_as(
                  glob_import_as,
                  &relative_file,
                  &source,
                  cur_index,
                  entry_index,
                  sources,
                ) {
                  props.push(prop);
                }
              } else if import_glob_info.eager {
                // add "./dir/foo.js": __glob__0_0
                props.push((
                  source.clone(),
                  Box::new(Expr::Ident(Ident::new(
                    format!("__glob__{cur_index}_{entry_index}").into(),
                    DUMMY_SP,
                  ))),
                ));
              } else {
                // add "./dir/foo.js": () => import('./dir/foo.js')
                let (rel_file, source) = if let Some(query) = &import_glob_info.query {
                  (
                    format!("{relative_file}?{query}"),
                    format!("{source}?{query}"),
                  )
                } else {
                  (relative_file.clone(), source.clone())
                };

                props.push(self.deal_with_non_eager(&rel_file, &source, &import_glob_info.import));
              }

              import_glob_info.globed_sources.push(relative_file);
            }

            // props to object literal
            let mut object_lit_props = vec![];
            for (key, value) in props {
              object_lit_props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: swc_ecma_ast::PropName::Str(swc_ecma_ast::Str {
                  span: DUMMY_SP,
                  value: key.into(),
                  raw: None,
                }),
                value,
              }))));
            }
            // replace expr with object literal
            *expr = Expr::Object(ObjectLit {
              span: DUMMY_SP,
              props: object_lit_props,
            });

            self.import_globs.push(import_glob_info);
          }
        }
      }
      _ => {
        expr.visit_mut_children_with(self);
      }
    }
  }
}

fn get_string_literal(expr: &ExprOrSpread) -> Option<Vec<String>> {
  match &expr.expr {
    box Expr::Lit(Lit::Str(str)) => Some(vec![str.value.to_string()]),
    box Expr::Array(ArrayLit { elems, .. }) => {
      let mut result = vec![];

      for elem in elems {
        if let Some(ExprOrSpread {
          spread: None,
          expr: box Expr::Lit(Lit::Str(str)),
        }) = elem
        {
          result.push(str.value.to_string());
        }
      }

      if !result.is_empty() {
        Some(result)
      } else {
        None
      }
    }
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
              box Expr::Object(ObjectLit { props, .. }) => {
                let mut query_str = String::new();

                for prop in props {
                  match prop {
                    swc_ecma_ast::PropOrSpread::Spread(_) => {}
                    swc_ecma_ast::PropOrSpread::Prop(box Prop::KeyValue(KeyValueProp {
                      key,
                      value,
                    })) => {
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
                        query_str.push_str(&format!("{}={}&", k.unwrap(), v.unwrap()));
                      }
                    }
                    _ => {}
                  }
                }

                if !query_str.is_empty() {
                  query_str.pop();
                  Some(query_str)
                } else {
                  None
                }
              }
              _ => None,
            };

            if k.is_some() && v.is_some() {
              result.insert(k.unwrap(), v.unwrap());
            }
          }
          _ => {}
        }
      }

      if !result.is_empty() {
        Some(result)
      } else {
        None
      }
    }
    _ => None,
  }
}
