use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::ResolveKind,
  swc_common::{Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{CallExpr, Callee, Expr, Ident, IdentName, MemberExpr, MemberProp},
  HashMap, HashSet,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use super::{unique_idents::TopLevelIdentsRenameHandler, utils::create_export_namespace_ident};

pub struct DynamicImportVisitor<'a> {
  module_id: &'a ModuleId,
  module_graph: &'a ModuleGraph,
  module_ids: &'a HashSet<ModuleId>,
  rename_handler: &'a TopLevelIdentsRenameHandler,

  unresolved_mark: Mark,
  pub external_modules: HashMap<(String, ResolveKind), ModuleId>,
}

impl<'a> DynamicImportVisitor<'a> {
  pub fn new(
    module_id: &'a ModuleId,
    module_graph: &'a ModuleGraph,
    module_ids: &'a HashSet<ModuleId>,
    rename_handler: &'a TopLevelIdentsRenameHandler,
  ) -> Self {
    let module = module_graph.module(module_id).unwrap();
    let unresolved_mark = module.meta.as_script().unresolved_mark;

    Self {
      module_id,
      module_graph,
      module_ids,
      rename_handler,
      unresolved_mark: Mark::from_u32(unresolved_mark),
      external_modules: HashMap::default(),
    }
  }
}

impl<'a> VisitMut for DynamicImportVisitor<'a> {
  fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
    call_expr.visit_mut_children_with(self);

    // Check if this is an import() call
    if !call_expr.callee.is_import() {
      return;
    }

    // Get the source string from the first argument
    let source = match &call_expr.args[0].expr.as_lit() {
      Some(lit) => match lit {
        farmfe_core::swc_ecma_ast::Lit::Str(s) => s.value.to_string(),
        _ => return,
      },
      None => return,
    };

    // Get the dependency module ID
    let dep_module_id = self.module_graph.get_dep_by_source(
      self.module_id,
      &source,
      Some(ResolveKind::DynamicImport),
    );

    // If the dependency is in module_ids, transform to Promise.resolve(namespace)
    if self.module_ids.contains(&dep_module_id) {
      let dep_module = self.module_graph.module(&dep_module_id).unwrap();

      if dep_module.external || !dep_module.module_type.is_script() {
        return;
      }

      let dep_module_meta = dep_module.meta.as_script();
      let top_level_mark = Mark::from_u32(dep_module_meta.top_level_mark);
      let namespace_ident = create_export_namespace_ident(&dep_module_id, top_level_mark)
        .to_id()
        .into();
      let namespace_ident = self
        .rename_handler
        .get_renamed_ident(&dep_module_id, &namespace_ident)
        .unwrap_or(namespace_ident);
      let ctxt = namespace_ident.ctxt();

      // Create Promise.resolve(namespace_ident)
      *call_expr = CallExpr {
        span: DUMMY_SP,
        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(Ident::new(
            "Promise".into(),
            DUMMY_SP,
            SyntaxContext::empty().apply_mark(self.unresolved_mark),
          ))),
          prop: MemberProp::Ident(IdentName {
            sym: "resolve".into(),
            span: DUMMY_SP,
          }),
        }))),
        args: vec![farmfe_core::swc_ecma_ast::ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Ident(Ident::new(namespace_ident.sym, DUMMY_SP, ctxt))),
        }],
        type_args: None,
        ctxt: SyntaxContext::empty(),
      };
    }
    // Otherwise keep as is - external modules will be handled by concatenate_modules_ast
    else {
      self
        .external_modules
        .insert((source, ResolveKind::DynamicImport), dep_module_id);
    }
  }
}
