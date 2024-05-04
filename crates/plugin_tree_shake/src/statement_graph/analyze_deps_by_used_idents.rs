use std::collections::{HashMap, HashSet};

use farmfe_core::swc_ecma_ast::{Id, Ident, ModuleExportName, ModuleItem};
use farmfe_toolkit::swc_ecma_visit::{Visit, VisitWith};

use super::{defined_idents_collector::DefinedIdentsCollector, StatementGraphEdge, StatementId};

pub struct AnalyzeUsedIdentsParams<'a> {
  pub id: &'a StatementId,
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
  current_defined_ident: Option<Id>,
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
    }
  }

  pub fn with_ident(&mut self, ident: Id, f: impl FnOnce(&mut Self)) {
    if self.current_defined_ident.is_some() {
      f(self);
      return;
    }

    self.current_defined_ident = Some(ident);
    f(self);
    self.current_defined_ident = None;
  }
}

impl Visit for UsedIdentsVisitor<'_> {
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
            self.current_defined_ident = Some(ident.to_id());
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
          self.with_ident(ident.to_id(), |v| {
            class_expr.class.visit_children_with(v);
          });
        } else {
          class_expr.class.visit_children_with(self);
        }
      }
      farmfe_core::swc_ecma_ast::DefaultDecl::Fn(fn_expr) => {
        if let Some(ident) = &fn_expr.ident {
          self.with_ident(ident.to_id(), |v| {
            fn_expr.function.visit_children_with(v);
          });
        } else {
          fn_expr.function.visit_children_with(self);
        }
      }
      farmfe_core::swc_ecma_ast::DefaultDecl::TsInterfaceDecl(_) => {}
    }
  }

  fn visit_decl(&mut self, n: &farmfe_core::swc_ecma_ast::Decl) {
    match n {
      farmfe_core::swc_ecma_ast::Decl::Fn(n) => {
        self.with_ident(n.ident.to_id(), |v| {
          n.function.visit_children_with(v);
        });
      }
      farmfe_core::swc_ecma_ast::Decl::Var(n) => {
        for decl in &n.decls {
          if let Some(init) = &decl.init {
            let mut defined_idents_collector = DefinedIdentsCollector::new();
            decl.name.visit_with(&mut defined_idents_collector);

            let mut used_ident_visitor = IdentCollector::new();
            init.visit_with(&mut used_ident_visitor);

            for defined_ident in defined_idents_collector.defined_idents {
              self.with_ident(defined_ident, |v| {
                for used_ident in &used_ident_visitor.idents {
                  v.visit_ident(used_ident);
                }
              });
            }
          }
        }
      }
      farmfe_core::swc_ecma_ast::Decl::Class(n) => {
        self.with_ident(n.ident.to_id(), |v| {
          n.class.visit_children_with(v);
        });
      }
      _ => {}
    }
  }

  fn visit_ident(&mut self, n: &farmfe_core::swc_ecma_ast::Ident) {
    let ident = n.to_id();

    if let Some(stmt_id) = self.reverse_defined_idents_map.get(&ident) {
      let entry = self.deps.entry(*stmt_id).or_insert(StatementGraphEdge {
        used_idents_map: HashMap::new(),
        used_idents: HashSet::new(),
      });

      if let Some(current_defined_ident) = &self.current_defined_ident {
        if self
          .reverse_defined_idents_map
          .contains_key(current_defined_ident)
        {
          entry
            .used_idents_map
            .entry(current_defined_ident.clone())
            .or_insert(HashSet::new())
            .insert(ident);
          return;
        }
      }

      entry.used_idents.insert(ident);
    }
  }
}

struct IdentCollector {
  pub idents: HashSet<Ident>,
}

impl IdentCollector {
  pub fn new() -> Self {
    Self {
      idents: HashSet::new(),
    }
  }
}

impl Visit for IdentCollector {
  fn visit_ident(&mut self, n: &farmfe_core::swc_ecma_ast::Ident) {
    self.idents.insert(n.clone());
  }
}
