use std::sync::Arc;

use farmfe_core::{
  error::Result,
  swc_common::SourceMap,
  swc_ecma_ast::{EsVersion, Module as SwcModule},
  swc_ecma_parser::Syntax,
};

#[no_mangle]
pub fn parse_module(
  lib: &libloading::Library,
  file_name: &str,
  src: &str,
  syntax: Syntax,
  target: EsVersion,
  cm: Arc<SourceMap>,
) -> Result<SwcModule> {
  unsafe {
    let farm_swc_parse_module: libloading::Symbol<
      unsafe fn(&str, &str, Syntax, EsVersion, Arc<SourceMap>) -> Result<SwcModule>,
    > = lib.get(b"farm_swc_parse_module").unwrap();

    farm_swc_parse_module(file_name, src, syntax, target, cm)
  }
}
