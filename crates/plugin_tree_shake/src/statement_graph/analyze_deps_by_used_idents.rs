use std::collections::HashMap;

use farmfe_core::swc_ecma_ast::{Id, ModuleExportName, ModuleItem};
use farmfe_toolkit::swc_ecma_visit::{Visit, VisitWith};

use super::{defined_idents_collector::DefinedIdentsCollector, StatementGraphEdge, StatementId};

pub struct AnalyzeUsedIdentsParams<'a> {
  pub stmt: &'a ModuleItem,
  pub reverse_defined_idents_map: &'a HashMap<Id, StatementId>,
}

pub fn analyze_deps_by_used_idents(
  params: AnalyzeUsedIdentsParams,
) -> HashMap<StatementId, StatementGraphEdge> {
  let mut deps = HashMap::new();

  let AnalyzeUsedIdentsParams {
    stmt,
    reverse_defined_idents_map,
    ..
  } = params;

  let mut visitor = UsedIdentsVisitor::new(&mut deps, reverse_defined_idents_map);
  stmt.visit_with(&mut visitor);

  deps
}

struct UsedIdentsVisitor<'a> {
  deps: &'a mut HashMap<StatementId, StatementGraphEdge>,
  reverse_defined_idents_map: &'a HashMap<Id, StatementId>,
  current_defined_ident: Option<Vec<Id>>,
  in_top_level: bool,
}

impl<'a> UsedIdentsVisitor<'a> {
  pub fn new(
    deps: &'a mut HashMap<StatementId, StatementGraphEdge>,
    reverse_defined_idents_map: &'a HashMap<Id, StatementId>,
  ) -> Self {
    Self {
      deps,
      reverse_defined_idents_map,
      current_defined_ident: None,
      in_top_level: true,
    }
  }

  pub fn with_ident(&mut self, idents: Vec<Id>, f: impl FnOnce(&mut Self)) {
    if self.current_defined_ident.is_some() {
      f(self);
      return;
    }
    let pre = self.current_defined_ident.take();
    self.current_defined_ident = Some(idents);
    f(self);
    self.current_defined_ident = pre;
  }

  pub fn into_scope(&mut self, f: impl FnOnce(&mut Self)) {
    let pre = self.in_top_level;
    self.in_top_level = false;
    f(self);
    self.in_top_level = pre;
  }

  pub fn handled_used_ident(&mut self, stmt_id: &StatementId, ident: Id) {
    let edge = self.deps.entry(*stmt_id).or_default();

    if let Some(current_defined_idents) = &self.current_defined_ident {
      let mut found = false;

      for current_defined_ident in current_defined_idents {
        if self
          .reverse_defined_idents_map
          .contains_key(current_defined_ident)
        {
          edge
            .used_idents_map
            .entry(current_defined_ident.clone())
            .or_default()
            .insert(ident.clone());
          found = true;
        }
      }

      if found {
        return;
      }
    }

    edge.used_idents.insert(ident);
  }
}

impl Visit for UsedIdentsVisitor<'_> {
  fn visit_import_decl(&mut self, _: &farmfe_core::swc_ecma_ast::ImportDecl) {
    // do not visit children of import decl
  }

  fn visit_named_export(&mut self, n: &farmfe_core::swc_ecma_ast::NamedExport) {
    if n.src.is_none() {
      // make edge of `const a = 1; export { a }` be `a -> a`
      let mut visit_children = false;

      for specifier in &n.specifiers {
        match specifier {
          farmfe_core::swc_ecma_ast::ExportSpecifier::Named(named_export) => {
            let ident = match &named_export.orig {
              ModuleExportName::Ident(ident) => ident,
              _ => panic!("unexpected named export orig"),
            };
            self.current_defined_ident = Some(vec![ident.to_id()]);
            self.visit_ident(ident);
            self.current_defined_ident = None;
          }
          _ => {
            visit_children = true;
          }
        }
      }

      if visit_children {
        n.visit_children_with(self);
      }
    } else {
      n.visit_children_with(self);
    }
  }
  fn visit_default_decl(&mut self, n: &farmfe_core::swc_ecma_ast::DefaultDecl) {
    self.in_top_level = false;
    // For export default decl, the defined ident can be used in the module.
    // But for the var decl, the defined ident can't be used in the module. For example:
    // ```js
    // var a = class b {};
    // console.log(a); // a is defined
    // console.log(b); // b is not defined
    // ```
    match n {
      farmfe_core::swc_ecma_ast::DefaultDecl::Class(class_expr) => {
        if let Some(ident) = &class_expr.ident {
          self.with_ident(vec![ident.to_id()], |v| {
            class_expr.class.visit_with(v);
          });
        } else {
          class_expr.class.visit_with(self);
        }
      }
      farmfe_core::swc_ecma_ast::DefaultDecl::Fn(fn_expr) => {
        if let Some(ident) = &fn_expr.ident {
          self.with_ident(vec![ident.to_id()], |v| {
            fn_expr.function.visit_with(v);
          });
        } else {
          fn_expr.function.visit_with(self);
        }
      }
      farmfe_core::swc_ecma_ast::DefaultDecl::TsInterfaceDecl(_) => {}
    }
    self.in_top_level = true;
  }

  fn visit_arrow_expr(&mut self, n: &farmfe_core::swc_ecma_ast::ArrowExpr) {
    self.into_scope(|v| n.visit_children_with(v))
  }

  fn visit_constructor(&mut self, n: &farmfe_core::swc_ecma_ast::Constructor) {
    self.into_scope(|v| n.visit_children_with(v))
  }

  fn visit_function(&mut self, n: &farmfe_core::swc_ecma_ast::Function) {
    self.into_scope(|v| n.visit_children_with(v))
  }

  fn visit_decl(&mut self, n: &farmfe_core::swc_ecma_ast::Decl) {
    if !self.in_top_level {
      n.visit_children_with(self);
      return;
    }

    match n {
      farmfe_core::swc_ecma_ast::Decl::Fn(n) => {
        self.with_ident(vec![n.ident.to_id()], |v| {
          v.in_top_level = false;
          n.function.visit_with(v);
          v.in_top_level = true;
        });
      }
      farmfe_core::swc_ecma_ast::Decl::Var(n) => {
        for decl in &n.decls {
          if let Some(init) = &decl.init {
            let mut cb = |from, to| {
              if let Some(to_stmt_id) = self.reverse_defined_idents_map.get(&to) {
                if self.reverse_defined_idents_map.contains_key(&from) {
                  let entry = self.deps.entry(*to_stmt_id).or_default();
                  entry.used_idents_map.entry(from).or_default().insert(to);
                } else {
                  self.handled_used_ident(to_stmt_id, to);
                }
              }
            };
            let mut defined_idents_collector =
              DefinedIdentsCollector::from_callback(Box::new(&mut cb));
            decl.name.visit_with(&mut defined_idents_collector);

            let defined_idents = defined_idents_collector
              .defined_idents
              .into_iter()
              .collect::<Vec<_>>();

            self.with_ident(defined_idents, |v| {
              init.visit_with(v);
            });
          }
        }
      }
      farmfe_core::swc_ecma_ast::Decl::Class(n) => {
        self.with_ident(vec![n.ident.to_id()], |v| {
          v.in_top_level = false;
          n.class.visit_with(v);
          v.in_top_level = true;
        });
      }
      _ => {}
    }
  }

  fn visit_ident(&mut self, n: &farmfe_core::swc_ecma_ast::Ident) {
    let ident = n.to_id();

    if let Some(stmt_id) = self.reverse_defined_idents_map.get(&ident) {
      self.handled_used_ident(stmt_id, ident)
    }
  }
}
