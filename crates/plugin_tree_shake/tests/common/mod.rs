use std::sync::Arc;

use farmfe_core::{
  module::{meta_data::script::ScriptModuleMetaData, Module, ModuleMetaData},
  swc_common::{comments::SingleThreadedComments, Globals, Mark, SourceMap, GLOBALS},
  swc_ecma_ast::{EsVersion, Id, Module as SwcModule},
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::{
  script::ParseScriptModuleResult, source_map::create_swc_source_map,
  swc_ecma_transforms::resolver, swc_ecma_visit::VisitMutWith,
};

pub fn parse_module_with_comments(code: &str) -> (SwcModule, SingleThreadedComments, Mark, Mark) {
  let ParseScriptModuleResult {
    ast: mut swc_module,
    comments,
  } = farmfe_toolkit::script::parse_module(
    &"any".into(),
    Arc::new(code.to_string()),
    Syntax::Es(Default::default()),
    EsVersion::Es2022,
    // None,
  )
  .unwrap();
  let top_level_mark = Mark::new();
  let unresolved_mark = Mark::new();

  swc_module.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));

  (swc_module, comments, unresolved_mark, top_level_mark)
}

pub fn parse_module(code: &str) -> (SwcModule, Arc<SourceMap>) {
  let (cm, _) = create_swc_source_map(&"any".into(), Arc::new(code.to_string()));
  let ParseScriptModuleResult {
    ast: mut swc_module,
    ..
  } = farmfe_toolkit::script::parse_module(
    &"any".into(),
    Arc::new(code.to_string()),
    Syntax::Es(Default::default()),
    EsVersion::Es2022,
    // None,
  )
  .unwrap();
  let top_level_mark = Mark::new();
  let unresoled_mark = Mark::new();

  swc_module.visit_mut_with(&mut resolver(unresoled_mark, top_level_mark, false));

  (swc_module, cm)
}

#[allow(dead_code)]
pub fn create_module_with_comments(code: &str) -> Module {
  let mut module = Module::new("used_exports_idents_test".into());
  let (ast, comments, unresolved_mark, top_level_mark) = parse_module_with_comments(code);
  module.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
    ast,
    top_level_mark: top_level_mark.as_u32(),
    unresolved_mark: unresolved_mark.as_u32(),
    module_system: farmfe_core::module::ModuleSystem::EsModule,
    hmr_self_accepted: false,
    hmr_accepted_deps: Default::default(),
    comments: comments.into(),
    custom: Default::default(),
    ..Default::default()
  }));
  module
}

#[allow(dead_code)]
pub fn create_module(code: &str) -> (Module, Arc<SourceMap>) {
  let mut module = Module::new("used_exports_idents_test".into());
  let (ast, cm) = parse_module(code);
  module.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
    ast,
    top_level_mark: 0,
    unresolved_mark: 0,
    module_system: farmfe_core::module::ModuleSystem::EsModule,
    hmr_self_accepted: false,
    hmr_accepted_deps: Default::default(),
    comments: Default::default(),
    custom: Default::default(),
    ..Default::default()
  }));
  (module, cm)
}

#[allow(dead_code)]
pub fn create_module_with_globals(code: &str) -> Module {
  GLOBALS.set(&Globals::new(), || {
    let mut module = Module::new("used_exports_idents_test".into());
    let (ast, _) = parse_module(code);
    module.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      ast,
      top_level_mark: 0,
      unresolved_mark: 0,
      module_system: farmfe_core::module::ModuleSystem::EsModule,
      hmr_self_accepted: false,
      hmr_accepted_deps: Default::default(),
      comments: Default::default(),
      custom: Default::default(),
      ..Default::default()
    }));
    module
  })
}

pub fn print_id(id: &Id) -> String {
  format!("{}{:?}", id.0, id.1)
}
