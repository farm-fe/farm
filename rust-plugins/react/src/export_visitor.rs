use farmfe_core::swc_ecma_ast::Decl;
use farmfe_toolkit::swc_ecma_visit::VisitAll;

pub struct AllExportsReactComponentsVisitor {
  pub are_all_exports_react_component: bool,
}

impl AllExportsReactComponentsVisitor {
  pub fn new() -> Self {
    Self {
      are_all_exports_react_component: true,
    }
  }
}

impl VisitAll for AllExportsReactComponentsVisitor {
  fn visit_export_decl(&mut self, expr: &farmfe_core::swc_ecma_ast::ExportDecl) {
    if let Decl::Fn(fn_decl) = &expr.decl {
      let name = fn_decl.ident.sym.to_string();
      if !name.starts_with(|a: char| a.is_uppercase()) {
        self.are_all_exports_react_component = false;
      }
    } else {
      self.are_all_exports_react_component = false;
    }
  }
}
