use std::collections::HashSet;

use farmfe_core::swc_common::Mark;
use farmfe_toolkit::{
  script::defined_idents_collector::DefinedIdentsCollector,
  swc_ecma_visit::{VisitMut, VisitWith},
};

/// Collect top level idents and unresolved ident to avoid duplicate ident declaration
/// TODO: Support top level idents
pub struct TopLevelIdentsCollector {
  pub top_level_idents: HashSet<String>,
}

impl TopLevelIdentsCollector {
  pub fn new() -> Self {
    Self {
      top_level_idents: HashSet::new(),
    }
  }
}

impl VisitMut for TopLevelIdentsCollector {
  /// ignore body of function
  fn visit_mut_function(&mut self, _: &mut farmfe_core::swc_ecma_ast::Function) {}
  /// ignore body of arrow function
  fn visit_mut_arrow_expr(&mut self, _: &mut farmfe_core::swc_ecma_ast::ArrowExpr) {}
  /// ignore body of class constructor
  fn visit_mut_class(&mut self, _: &mut farmfe_core::swc_ecma_ast::Class) {}

  fn visit_mut_decl(&mut self, n: &mut farmfe_core::swc_ecma_ast::Decl) {
    match n {
      farmfe_core::swc_ecma_ast::Decl::Class(class) => {
        self.top_level_idents.insert(class.ident.sym.to_string());
      }
      farmfe_core::swc_ecma_ast::Decl::Fn(func) => {
        self.top_level_idents.insert(func.ident.sym.to_string());
      }
      farmfe_core::swc_ecma_ast::Decl::Var(var_decls) => {
        for var_decl in &mut var_decls.decls {
          let mut defined_idents_collector = DefinedIdentsCollector::new();
          var_decl.name.visit_with(&mut defined_idents_collector);

          for defined_ident in defined_idents_collector.defined_idents {
            self.top_level_idents.insert(defined_ident.0.to_string());
          }
        }
      }
      _ => { /* do nothing */ }
    }
  }

  fn visit_mut_default_decl(&mut self, n: &mut farmfe_core::swc_ecma_ast::DefaultDecl) {
    match n {
      farmfe_core::swc_ecma_ast::DefaultDecl::Class(class) => {
        if let Some(ident) = &class.ident {
          self.top_level_idents.insert(ident.sym.to_string());
        }
      }
      farmfe_core::swc_ecma_ast::DefaultDecl::Fn(func) => {
        if let Some(ident) = &func.ident {
          self.top_level_idents.insert(ident.sym.to_string());
        }
      }
      farmfe_core::swc_ecma_ast::DefaultDecl::TsInterfaceDecl(_) => { /* do nothing */ }
    }
  }

  fn visit_mut_import_decl(&mut self, n: &mut farmfe_core::swc_ecma_ast::ImportDecl) {
    for sp in &n.specifiers {
      match sp {
        farmfe_core::swc_ecma_ast::ImportSpecifier::Named(name) => {
          self.top_level_idents.insert(name.local.sym.to_string());
        }
        farmfe_core::swc_ecma_ast::ImportSpecifier::Default(default) => {
          self.top_level_idents.insert(default.local.sym.to_string());
        }
        farmfe_core::swc_ecma_ast::ImportSpecifier::Namespace(ns) => {
          self.top_level_idents.insert(ns.local.sym.to_string());
        }
      }
    }
  }
}

pub struct UnresolvedIdentCollector {
  pub unresolved_idents: HashSet<String>,
  unresolved_mark: Mark,
}

impl UnresolvedIdentCollector {
  pub fn new(unresolved_mark: Mark) -> Self {
    Self {
      unresolved_idents: HashSet::new(),
      unresolved_mark,
    }
  }
}

impl VisitMut for UnresolvedIdentCollector {
  fn visit_mut_ident(&mut self, n: &mut farmfe_core::swc_ecma_ast::Ident) {
    if n.ctxt.outer() == self.unresolved_mark {
      self.unresolved_idents.insert(n.sym.to_string());
    }
  }
}
