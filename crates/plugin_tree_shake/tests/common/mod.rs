use std::sync::Arc;

use farmfe_core::{
  module::{Module, ModuleMetaData, ScriptModuleMetaData},
  swc_common::{FilePathMapping, Globals, Mark, SourceMap, GLOBALS},
  swc_ecma_ast::{EsVersion, Module as SwcModule},
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::{swc_ecma_transforms::resolver, swc_ecma_visit::VisitMutWith};

pub fn parse_module(code: &str) -> SwcModule {
  let mut swc_module = farmfe_toolkit::script::parse_module(
    "any",
    code,
    Syntax::Es(Default::default()),
    EsVersion::EsNext,
    Arc::new(SourceMap::new(FilePathMapping::empty())),
  )
  .unwrap();

  swc_module.body.visit_mut_with(&mut resolver(
    Mark::fresh(Mark::root()),
    Mark::fresh(Mark::root()),
    false,
  ));

  swc_module
}

pub fn create_module(code: &str) -> Module {
  let mut module = Module::new("used_exports_idents_test".into());
  module.meta = ModuleMetaData::Script(ScriptModuleMetaData {
    ast: parse_module(code),
    top_level_mark: 0,
    unresolved_mark: 0,
    module_system: farmfe_core::module::ModuleSystem::EsModule,
    hmr_accepted: false,
  });
  module
}

pub fn create_module_with_globals(code: &str) -> Module {
  GLOBALS.set(&Globals::new(), || {
    let mut module = Module::new("used_exports_idents_test".into());
    module.meta = ModuleMetaData::Script(ScriptModuleMetaData {
      ast: parse_module(code),
      top_level_mark: 0,
      unresolved_mark: 0,
      module_system: farmfe_core::module::ModuleSystem::EsModule,
      hmr_accepted: false,
    });
    module
  })
}
