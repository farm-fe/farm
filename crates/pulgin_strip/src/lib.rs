#![deny(clippy::all)]

use farmfe_core::{
  config::{config_regex::ConfigRegex, Config},
  context::CompilationContext,
  error::Result as HookResult,
  plugin::{Plugin, PluginTransformHookParam, PluginTransformHookResult},
  swc_common::DUMMY_SP,
  swc_ecma_ast::*,
  swc_ecma_parser::Syntax,
};

use farmfe_toolkit::{
  common::{
    build_source_map, create_swc_source_map, PathFilter, Source,
  },
  script::{codegen_module, parse_module, CodeGenCommentsConfig, ParseScriptModuleResult},
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

use regex::Regex;
use std::{error::Error, path::PathBuf, sync::Arc};

#[derive(serde::Deserialize, Clone)]
pub struct Options {
  pub include: Option<Vec<ConfigRegex>>,
  pub exclude: Option<Vec<ConfigRegex>>,
  pub labels: Option<Vec<String>>,
  pub functions: Option<Vec<String>>,
  pub source_map: Option<bool>,
  pub debugger: Option<bool>,
}

const PLUGIN_NAME: &str = "FarmPulginStrip";

pub struct FarmPulginStrip {
  options: Options,
}

impl FarmPulginStrip {
  pub fn new(_: &Config, options: Options) -> Self {
    Self { options }
  }
}

impl Plugin for FarmPulginStrip {
  fn name(&self) -> &str {
    PLUGIN_NAME
  }

  fn transform(
    &self,
    param: &PluginTransformHookParam,
    context: &std::sync::Arc<CompilationContext>,
  ) -> HookResult<Option<PluginTransformHookResult>> {
    let options = self.options.clone();
    let include = options.include.unwrap_or(vec![]);
    let exclude = options.exclude.unwrap_or(vec![]);
    let filter = PathFilter::new(&include, &exclude);
    if !filter.execute(param.resolved_path) {
      return Ok(None);
    }

    let source_map = match options.source_map {
      Some(s) => s != false,
      None => false,
    };

    let remove_debugger_statements = match options.debugger {
      Some(s) => s != false,
      None => false,
    };

    let labels = options.labels.unwrap_or(vec![]);

    let functions: Vec<String> = options.functions.unwrap_or(vec![]);

    let mut labels_patterns: Vec<String> = labels.iter().map(|l| format!(r"{}\s*:", l)).collect();
    let mut first_pass = functions.clone();
    first_pass.append(&mut labels_patterns);
    if remove_debugger_statements {
      first_pass.push("debugger\\b".to_string())
    }

    let first_pass_filter = if !first_pass.is_empty() {
      Box::new(move |code: &str| create_first_pass_regex(&first_pass).unwrap().is_match(code))
        as Box<dyn Fn(&str) -> bool>
    } else {
      Box::new(|_: &str| false) as Box<dyn Fn(&str) -> bool>
    };

    if !first_pass_filter(&param.content) {
      return Ok(None);
    }

    let (cm, _) = create_swc_source_map(Source {
      path: PathBuf::from(param.resolved_path),
      content: Arc::new(param.content.clone()),
    });
    let ParseScriptModuleResult { mut ast, comments } = match parse_module(
      &param.module_id,
      &param.content,
      Syntax::Es(Default::default()),
      EsVersion::EsNext,
    ) {
      Ok(res) => res,
      Err(err) => {
        println!("{}", err.to_string());
        panic!("Parse {} failed. See error details above.", param.module_id);
      }
    };
    let re_functions_regex = create_regex_from_list(&functions).unwrap();
    let mut remover = StripCode::new(labels, re_functions_regex, remove_debugger_statements);
    ast.visit_mut_with(&mut remover);
    let mut src_map = vec![];
    let transformed_content = codegen_module(
      &ast,
      context.config.script.target.clone(),
      cm.clone(),
      if source_map { Some(&mut src_map) } else { None },
      context.config.minify.enabled(),
      Some(CodeGenCommentsConfig {
        comments: &comments,
        config: &context.config.comments,
      }),
    )
    .unwrap();

    let output_code = String::from_utf8(transformed_content).unwrap();

    let map = if source_map {
      let map = build_source_map(cm, &src_map);
      let mut buf = vec![];
      map.to_writer(&mut buf).expect("failed to write sourcemap");
      Some(String::from_utf8(buf).unwrap())
    } else {
      None
    };

    Ok(Some(PluginTransformHookResult {
      content: output_code,
      source_map: map,
      module_type: Some(param.module_type.clone()),
      ignore_previous_source_map: false,
    }))
  }
}

struct StripCode {
  labels: Vec<String>,
  re_functions_regex: Regex,
  should_re_debugger: bool,
}

impl StripCode {
  fn new(labels: Vec<String>, re_functions_regex: Regex, should_re_debugger: bool) -> Self {
    StripCode {
      labels,
      re_functions_regex,
      should_re_debugger,
    }
  }
}

impl VisitMut for StripCode {
  fn visit_mut_module(&mut self, module: &mut Module) {
    let should_remove_debugger = self.should_re_debugger;
    let labels_to_remove = &self.labels;
    module.body.retain_mut(|item| match item {
      ModuleItem::Stmt(Stmt::Debugger(..)) => !should_remove_debugger,
      ModuleItem::Stmt(Stmt::Labeled(LabeledStmt { label, .. })) => {
        !labels_to_remove.contains(&label.sym.to_string())
      }
      _ => true,
    });
    module.visit_mut_children_with(self);
  }

  fn visit_mut_stmts(&mut self, stmts: &mut Vec<Stmt>) {
    let should_remove_debugger = self.should_re_debugger;
    let labels_to_remove = &self.labels;
    stmts.retain_mut(|stmt| match stmt {
      Stmt::Debugger(_) => !should_remove_debugger,
      Stmt::Labeled(LabeledStmt { label, .. }) => {
        !labels_to_remove.contains(&label.sym.to_string())
      }
      _ => true,
    });
    for stmt in stmts.iter_mut() {
      stmt.visit_mut_with(self);
    }
  }

  fn visit_mut_expr(&mut self, e: &mut Expr) {
    if let Expr::Call(CallExpr { callee, .. }) = e {
      if let Callee::Expr(callee_expr) = callee {
        if let Some(flatten_callee) = flatten(&callee_expr) {
          if self.re_functions_regex.is_match(&flatten_callee) {
            *e = *void_expr();
            return;
          }
        }
      }
    }
    e.visit_mut_children_with(self);
  }
}

fn void_expr() -> Box<Expr> {
  Box::new(Expr::Ident(Ident {
    sym: "(void 0)".into(),
    span: DUMMY_SP,
    optional: false,
  }))
}

fn flatten(expr: &Expr) -> Option<String> {
  match expr {
    Expr::Member(MemberExpr { obj, prop, .. }) => {
      let mut parts = vec![];
      if let MemberProp::Ident(ident) = &prop {
        parts.push(ident.sym.to_string());
      } else {
        return None;
      }

      let mut current_obj = obj;
      while let Expr::Member(nested_member) = &**current_obj {
        if let MemberProp::Ident(ident) = &nested_member.prop {
          parts.push(ident.sym.to_string());
        }
        current_obj = &nested_member.obj;
      }

      if let Expr::Ident(ident) = &**current_obj {
        parts.push(ident.sym.to_string());
      } else {
        return None;
      }

      parts.reverse();
      Some(parts.join("."))
    }
    _ => None,
  }
}

fn create_regex_from_list(functions: &[String]) -> Result<Regex, Box<dyn Error>> {
  let patterns: Vec<&str> = functions.iter().map(AsRef::as_ref).collect();
  let joined_patterns = patterns.join("|");
  let regex = Regex::new(&format!("^(?:{})$", joined_patterns))?;
  Ok(regex)
}

fn create_first_pass_regex(first_pass: &[String]) -> Result<Regex, Box<dyn Error>> {
  let patterns: Vec<&str> = first_pass.iter().map(AsRef::as_ref).collect();
  let joined_patterns = patterns.join("|");
  let regex = Regex::new(&format!(r"\b(?:{})", joined_patterns))?;
  Ok(regex)
}
