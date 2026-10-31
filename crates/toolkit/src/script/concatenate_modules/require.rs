use farmfe_core::{
  HashMap,
  module::{ModuleId, module_graph::ModuleGraph},
  plugin::ResolveKind,
  swc_common::Mark,
  swc_ecma_ast::{CallExpr, Callee, Expr, Lit},
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

/// Retain the resolved target before the originating module is concatenated.
/// Source strings alone are ambiguous across directories (and package scopes).
pub struct RequireVisitor<'a> {
  pub module_id: &'a ModuleId,
  pub module_graph: &'a ModuleGraph,
  pub unresolved_mark: Mark,
  pub external_modules: HashMap<(String, ResolveKind), ModuleId>,
}

impl VisitMut for RequireVisitor<'_> {
  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);
    if !matches!(&call.callee, Callee::Expr(box Expr::Ident(ident))
      if ident.sym == "require" && ident.ctxt.outer() == self.unresolved_mark)
      || call.args.len() != 1
      || call.args[0].spread.is_some()
    {
      return;
    }
    let Expr::Lit(Lit::Str(source)) = &mut *call.args[0].expr else {
      return;
    };
    let Some(target) = self.module_graph.get_dep_by_source_optional(
      self.module_id,
      &source.value.to_string_lossy(),
      Some(ResolveKind::Require),
    ) else {
      return;
    };
    // This internal key is consumed by SourceReplacer, never emitted as a URL.
    let key = format!("\0farm:concatenated-require:{}", target.to_string());
    source.value = key.clone().into();
    source.raw = None;
    self
      .external_modules
      .insert((key, ResolveKind::Require), target);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::script::{parse_module, swc_try_with::resolve_module_mark};
  use farmfe_core::{
    module::{
      Module,
      module_graph::{ModuleGraphEdge, ModuleGraphEdgeDataItem},
    },
    swc_common::{GLOBALS, Globals},
    swc_ecma_ast::EsVersion,
    swc_ecma_parser::Syntax,
  };
  use std::sync::Arc;

  fn rewrite(
    origin: &str,
    target: &str,
    source: &str,
  ) -> (
    farmfe_core::swc_ecma_ast::Module,
    HashMap<(String, ResolveKind), ModuleId>,
  ) {
    let id: ModuleId = origin.into();
    let target: ModuleId = target.into();
    let mut graph = ModuleGraph::new();
    graph.add_module(Module::new(id.clone()));
    graph.add_module(Module::new(target.clone()));
    graph
      .add_edge(
        &id,
        &target,
        ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
          source: "./dep.cjs".into(),
          kind: ResolveKind::Require,
          order: 0,
        }]),
      )
      .unwrap();
    let mut ast = parse_module(
      &id,
      Arc::new(source.to_owned()),
      Syntax::Es(Default::default()),
      EsVersion::latest(),
    )
    .unwrap()
    .ast;
    let globals = Globals::new();
    let (unresolved_mark, _) = resolve_module_mark(&mut ast, false, &globals);
    let mut visitor = RequireVisitor {
      module_id: &id,
      module_graph: &graph,
      unresolved_mark,
      external_modules: HashMap::default(),
    };
    GLOBALS.set(&globals, || ast.visit_mut_with(&mut visitor));
    (ast, visitor.external_modules)
  }

  #[test]
  fn hoisted_require_keeps_distinct_directory_targets() {
    let (first, a) = rewrite("a/entry.js", "a/dep.cjs", "require('./dep.cjs')");
    let (second, b) = rewrite("b/entry.js", "b/dep.cjs", "require('./dep.cjs')");
    assert_ne!(a.keys().next(), b.keys().next());
    assert_eq!(a.values().next(), Some(&ModuleId::from("a/dep.cjs")));
    assert_eq!(b.values().next(), Some(&ModuleId::from("b/dep.cjs")));
    assert_ne!(format!("{first:?}"), format!("{second:?}"));
  }

  #[test]
  fn hoisted_require_preserves_local_and_dynamic_calls() {
    for source in [
      "function f(require){return require('./dep.cjs')}",
      "const require = x => x; require('./dep.cjs')",
      "require(name)",
      "require(...['./dep.cjs'])",
      "require('./dep.cjs', true)",
    ] {
      let (_, mappings) = rewrite("entry.js", "dep.cjs", source);
      assert!(mappings.is_empty(), "{source}");
    }
  }
}
