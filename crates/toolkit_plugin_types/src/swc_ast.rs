use std::sync::Arc;

use farmfe_core::{
  error::Result,
  swc_common::{comments::SingleThreadedComments, SourceFile, SourceMap},
  swc_ecma_ast::{EsVersion, Module as SwcModule},
  swc_ecma_parser::Syntax,
};

pub struct ParseScriptModuleResult {
  pub ast: SwcModule,
  pub comments: SingleThreadedComments,
  pub source_map: Arc<SourceMap>,
}

#[no_mangle]
pub fn parse_module(
  lib: &libloading::Library,
  file_name: &str,
  src: &str,
  syntax: Syntax,
  target: EsVersion,
) -> Result<ParseScriptModuleResult> {
  unsafe {
    let farm_swc_parse_module: libloading::Symbol<
      unsafe fn(&str, &str, Syntax, EsVersion) -> Result<ParseScriptModuleResult>,
    > = lib.get(b"farm_swc_parse_module").unwrap();

    farm_swc_parse_module(file_name, src, syntax, target)
  }
}

#[no_mangle]
pub fn create_swc_source_map(
  lib: &libloading::Library,
  file_name: &str,
  content: Arc<String>,
) -> Result<(Arc<SourceMap>, Arc<SourceFile>)> {
  unsafe {
    let farm_create_swc_source_map: libloading::Symbol<
      unsafe fn(&str, Arc<String>) -> Result<(Arc<SourceMap>, Arc<SourceFile>)>,
    > = lib.get(b"farm_create_swc_source_map").unwrap();

    farm_create_swc_source_map(file_name, content)
  }
}
