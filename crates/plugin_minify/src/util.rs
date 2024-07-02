use farmfe_core::swc_ecma_ast::{Ident, ModuleExportName};

pub fn get_module_export_name(n: ModuleExportName) -> Ident {
  match n {
    ModuleExportName::Ident(ident) => ident,
    ModuleExportName::Str(_) => unreachable!(),
  }
}
