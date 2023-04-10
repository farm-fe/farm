use std::sync::Arc;

use farmfe_core::{
  module::{Module, ModuleMetaData, ScriptModuleMetaData},
  swc_common::{FilePathMapping, Globals, Mark, SourceMap, GLOBALS},
  swc_ecma_ast::{EsVersion, Module as SwcModule},
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::{swc_ecma_transforms::resolver, swc_ecma_visit::VisitMutWith};

pub fn parse_module(code: &str) -> (SwcModule, Arc<SourceMap>) {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
  let mut swc_module = farmfe_toolkit::script::parse_module(
    "any",
    code,
    Syntax::Es(Default::default()),
    EsVersion::EsNext,
    cm.clone(),
  )
  .unwrap();

  swc_module.body.visit_mut_with(&mut resolver(
    Mark::fresh(Mark::root()),
    Mark::fresh(Mark::root()),
    false,
  ));

  (swc_module, cm)
}

pub fn create_module(code: &str) -> (Module, Arc<SourceMap>) {
  let mut module = Module::new("used_exports_idents_test".into());
  let (ast, cm) = parse_module(code);
  module.meta = ModuleMetaData::Script(ScriptModuleMetaData {
    ast,
    top_level_mark: 0,
    unresolved_mark: 0,
    module_system: farmfe_core::module::ModuleSystem::EsModule,
    hmr_accepted: false,
  });
  (module, cm)
}

pub fn create_module_with_globals(code: &str) -> Module {
  GLOBALS.set(&Globals::new(), || {
    let mut module = Module::new("used_exports_idents_test".into());
    let (ast, _) = parse_module(code);
    module.meta = ModuleMetaData::Script(ScriptModuleMetaData {
      ast,
      top_level_mark: 0,
      unresolved_mark: 0,
      module_system: farmfe_core::module::ModuleSystem::EsModule,
      hmr_accepted: false,
    });
    module
  })
}
