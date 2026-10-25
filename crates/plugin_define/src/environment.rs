//! Platform-injected environment values are expression reads, not text macros.
use std::collections::{BTreeMap, HashSet};

use farmfe_core::{
  config::Config,
  error::Result,
  swc_common::{Mark, Span},
  swc_ecma_ast::{Expr, MemberExpr, MemberProp, Stmt, UnaryOp},
};
use farmfe_toolkit::{
  script::parse_stmt,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

use super::{DEFAULT_DEFINE_PROCESS_ENV, REGEX_PREFIX};

pub type EnvironmentDefines = BTreeMap<String, Box<Expr>>;

pub fn prepare(config: &Config) -> Result<(EnvironmentDefines, HashSet<String>)> {
  let mut defines = BTreeMap::new();
  let mut automatic_keys = HashSet::new();
  if let Some(values) = config
    .define
    .get(DEFAULT_DEFINE_PROCESS_ENV)
    .and_then(|value| value.as_object())
  {
    for (name, value) in values {
      let key = format!("{REGEX_PREFIX}(global(This)?\\.)?process\\.env\\.{name}");
      let encoded = value.to_string();
      // Only claim the exact pair injected by Core. Explicit user text/regex
      // substitutions, and node targets without this pair, keep their contract.
      if config.define.get(&key).and_then(|value| value.as_str()) != Some(encoded.as_str()) {
        continue;
      }
      if let Stmt::Expr(statement) = parse_stmt("farm:environment-value", &format!("({encoded});"))?
      {
        defines.insert(name.clone(), statement.expr);
        automatic_keys.insert(key);
      }
    }
  }
  Ok((defines, automatic_keys))
}

pub struct EnvironmentVisitor<'a> {
  defines: &'a EnvironmentDefines,
  unresolved_mark: Mark,
}

impl<'a> EnvironmentVisitor<'a> {
  pub fn new(defines: &'a EnvironmentDefines, unresolved_mark: Mark) -> Self {
    Self {
      defines,
      unresolved_mark,
    }
  }

  fn global(&self, expression: &Expr, name: &str) -> bool {
    matches!(expression, Expr::Ident(ident) if ident.sym == name && ident.ctxt.outer() == self.unresolved_mark)
  }

  fn process(&self, expression: &Expr) -> bool {
    self.global(expression, "process")
      || matches!(expression, Expr::Member(member)
      if property(member) == Some("process") && (self.global(&member.obj, "globalThis") || self.global(&member.obj, "global")))
  }

  fn environment_name<'b>(&self, expression: &'b Expr) -> Option<&'b str> {
    let Expr::Member(member) = expression else {
      return None;
    };
    let Expr::Member(environment) = member.obj.as_ref() else {
      return None;
    };
    if property(environment) != Some("env") || !self.process(&environment.obj) {
      return None;
    }
    property(member)
  }

  fn visit_reference(&mut self, expression: &mut Expr) {
    if let Expr::Paren(paren) = expression {
      self.visit_reference(&mut paren.expr);
    } else {
      // The outer member is a location. Its object and computed key may still
      // contain ordinary reads (e.g. process.env[process.env.NODE_ENV]).
      expression.visit_mut_children_with(self);
    }
  }
}

fn property(member: &MemberExpr) -> Option<&str> {
  match &member.prop {
    MemberProp::Ident(ident) => Some(ident.sym.as_ref()),
    MemberProp::Computed(computed) => match computed.expr.as_ref() {
      Expr::Lit(farmfe_core::swc_ecma_ast::Lit::Str(value)) => value.value.as_str(),
      _ => None,
    },
    _ => None,
  }
}

struct ReplacementSpan(Span);
impl VisitMut for ReplacementSpan {
  fn visit_mut_span(&mut self, span: &mut Span) {
    *span = self.0;
  }
}

impl VisitMut for EnvironmentVisitor<'_> {
  fn visit_mut_expr(&mut self, expression: &mut Expr) {
    if let Some(value) = self
      .environment_name(expression)
      .and_then(|name| self.defines.get(name))
    {
      use farmfe_core::swc_common::Spanned;
      let span = expression.span();
      *expression = *value.clone();
      expression.visit_mut_with(&mut ReplacementSpan(span));
      return;
    }
    expression.visit_mut_children_with(self);
  }

  fn visit_mut_update_expr(&mut self, update: &mut farmfe_core::swc_ecma_ast::UpdateExpr) {
    self.visit_reference(&mut update.arg);
  }

  fn visit_mut_unary_expr(&mut self, unary: &mut farmfe_core::swc_ecma_ast::UnaryExpr) {
    if unary.op == UnaryOp::Delete {
      self.visit_reference(&mut unary.arg);
    } else {
      unary.visit_mut_children_with(self);
    }
  }

  fn visit_mut_pat(&mut self, pattern: &mut farmfe_core::swc_ecma_ast::Pat) {
    if let farmfe_core::swc_ecma_ast::Pat::Expr(expression) = pattern {
      self.visit_reference(expression);
    } else {
      pattern.visit_mut_children_with(self);
    }
  }

  fn visit_mut_simple_assign_target(
    &mut self,
    target: &mut farmfe_core::swc_ecma_ast::SimpleAssignTarget,
  ) {
    if let farmfe_core::swc_ecma_ast::SimpleAssignTarget::Paren(paren) = target {
      self.visit_reference(&mut paren.expr);
    } else {
      target.visit_mut_children_with(self);
    }
  }
}
