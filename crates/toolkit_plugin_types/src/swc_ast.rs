use std::sync::Arc;

use farmfe_core::{
  error::Result,
  module::ModuleId,
  swc_common::{comments::SingleThreadedComments, Globals, SourceFile, SourceMap},
  swc_ecma_ast::{EsVersion, Module as SwcModule},
  swc_ecma_parser::Syntax,
};

pub struct ParseScriptModuleResult {
  pub ast: SwcModule,
  pub comments: SingleThreadedComments,
}

pub fn farm_swc_parse_module(
  lib: &libloading::Library,
  id: &ModuleId,
  content: Arc<String>,
  syntax: Syntax,
  target: EsVersion,
) -> Result<ParseScriptModuleResult> {
  unsafe {
    let farm_swc_parse_module: libloading::Symbol<
      unsafe fn(&ModuleId, Arc<String>, Syntax, EsVersion) -> Result<ParseScriptModuleResult>,
    > = lib.get(b"farm_swc_parse_module").unwrap();
    farm_swc_parse_module(id, content, syntax, target)
  }
}

pub fn farm_create_swc_source_map(
  lib: &libloading::Library,
  id: &ModuleId,
  content: Arc<String>,
) -> Result<(Arc<SourceMap>, Arc<SourceFile>)> {
  unsafe {
    let farm_create_swc_source_map: libloading::Symbol<
      unsafe fn(&ModuleId, Arc<String>) -> Result<(Arc<SourceMap>, Arc<SourceFile>)>,
    > = lib.get(b"farm_create_swc_source_map").unwrap();
    farm_create_swc_source_map(id, content)
  }
}

pub fn farm_swc_try_with(
  lib: &libloading::Library,
  cm: Arc<SourceMap>,
  globals: &Globals,
  op: Box<dyn FnOnce() -> ()>,
) -> Result<()> {
  unsafe {
    let farm_swc_try_with: libloading::Symbol<
      unsafe fn(Arc<SourceMap>, &Globals, Box<dyn FnOnce() -> ()>) -> Result<()>,
    > = lib.get(b"farm_swc_try_with").unwrap();
    farm_swc_try_with(cm, globals, op)
  }
}
