use std::sync::Arc;

use farmfe_core::{
  error::Result,
  swc_common::SourceMap,
  swc_ecma_ast::{EsVersion, Module as SwcModule},
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::script::parse_module;

#[no_mangle]
pub fn farm_swc_parse_module(
  id: &str,
  content: &str,
  syntax: Syntax,
  target: EsVersion,
  cm: Arc<SourceMap>,
) -> Result<SwcModule> {
  parse_module(id, content, syntax, target, cm)
}
