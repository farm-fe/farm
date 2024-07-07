use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  swc_ecma_ast::{Ident, ModuleDecl, ModuleExportName, ModuleItem},
};

pub fn get_module_export_name(n: ModuleExportName) -> Ident {
  match n {
    ModuleExportName::Ident(ident) => ident,
    ModuleExportName::Str(_) => unreachable!(),
  }
}

pub fn is_module_contains_export(module_id: &ModuleId, module_graph: &ModuleGraph) -> bool {
  let module = module_graph.module(module_id).unwrap();

  if !module.module_type.is_script() {
    return false;
  }

  let meta = module.meta.as_script();
  let ast = &meta.ast;

  ast.body.iter().any(|item| {
    matches!(
      item,
      ModuleItem::ModuleDecl(
        ModuleDecl::ExportAll(_)
          | ModuleDecl::ExportDecl(_)
          | ModuleDecl::ExportDefaultDecl(_)
          | ModuleDecl::ExportNamed(_)
      )
    )
  })
}
