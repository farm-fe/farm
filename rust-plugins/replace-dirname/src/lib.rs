// 2024-8-21 farmfe_core: 0.6.4
#![deny(clippy::all)]

use farmfe_core::{
  config::{config_regex::ConfigRegex, Config},
  context::CompilationContext,
  error::CompilationError,
  plugin::{Plugin, PluginProcessModuleHookParam},
  swc_common::DUMMY_SP,
  swc_ecma_ast::{self, Expr, Lit, MemberExpr, MemberProp, Module, Str},
};
use std::{env, path::Path, sync::Arc};
use url::Url;

use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{
  common::PathFilter,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};
use serde;

#[farm_plugin]
pub struct FarmPluginReplaceDirname {
  options: ReplaceDirnameOptions,
}

#[derive(serde::Deserialize)]
pub struct ReplaceDirnameOptions {
  exclude: Vec<ConfigRegex>,
  include: Vec<ConfigRegex>,
}

impl Default for ReplaceDirnameOptions {
  fn default() -> Self {
    Self {
      exclude: vec![ConfigRegex::new("node_modules/")],
      include: vec![],
    }
  }
}

impl FarmPluginReplaceDirname {
  fn new(_: &Config, options: String) -> Self {
    let options: ReplaceDirnameOptions = serde_json::from_str(&options).unwrap_or_default();
    Self { options }
  }
}

impl Plugin for FarmPluginReplaceDirname {
  fn name(&self) -> &str {
    "FarmPluginReplaceDirname"
  }

  fn process_module(
    &self,
    param: &mut PluginProcessModuleHookParam,
    _: &Arc<CompilationContext>,
  ) -> Result<Option<()>, CompilationError> {
    let filter = PathFilter::new(&self.options.include, &self.options.exclude);
    if !filter.execute(&param.module_id.relative_path()) {
      return Ok(None);
    }

    let file_path = env::current_dir()
      .unwrap()
      .join(param.module_id.relative_path());

    let dir_path: &str = Path::new(&file_path)
      .parent()
      .map_or("", |p| p.to_str().unwrap_or(""));

    let ast = &mut param.meta.as_script_mut().ast;
    replace_dirname_with_ast(ast, dir_path, file_path.to_str().unwrap());
    Ok(Some(()))
  }
}

pub fn replace_dirname_with_ast(ast: &mut Module, dir_path: &str, file_path: &str) {
  struct ReplaceLibVisitor<'a> {
    dir_path: &'a str,
    file_path: &'a str,
  }

  impl<'a> VisitMut for ReplaceLibVisitor<'a> {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
      match expr {
        Expr::Ident(ident) => match &*ident.sym {
          "__dirname" => {
            *expr = Expr::Lit(Lit::Str(Str {
              value: self.dir_path.into(),
              span: DUMMY_SP,
              raw: None,
            }));
          }
          "__filename" => {
            *expr = Expr::Lit(Lit::Str(Str {
              value: self.file_path.into(),
              span: DUMMY_SP,
              raw: None,
            }));
          }
          _ => {}
        },
        Expr::Member(MemberExpr { obj, prop, .. }) => {
          if let Expr::MetaProp(meta_prop) = &**obj {
            if meta_prop.kind == swc_ecma_ast::MetaPropKind::ImportMeta {
              if let MemberProp::Ident(ident) = &prop {
                if ident.sym == "url" {
                  if let Ok(file_path) = Url::from_file_path(&self.file_path) {
                    *expr = Expr::Lit(Lit::Str(Str {
                      value: file_path.to_string().into(),
                      span: DUMMY_SP,
                      raw: None,
                    }));
                  }
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

  let mut visitor = ReplaceLibVisitor {
    dir_path,
    file_path,
  };

  ast.visit_mut_with(&mut visitor);
}
