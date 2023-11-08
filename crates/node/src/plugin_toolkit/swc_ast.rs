use std::sync::Arc;

use farmfe_core::{
  error::Result,
  swc_common::{SourceFile, SourceMap},
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
) -> Result<SwcModule> {
  parse_module(id, content, syntax, target)
}

#[no_mangle]
pub fn farm_create_swc_source_map(
  id: &str,
  content: Arc<String>,
) -> Result<(Arc<SourceMap>, Arc<SourceFile>)> {
  let (cm, source_file) =
    farmfe_toolkit::common::create_swc_source_map(farmfe_toolkit::common::Source {
      path: std::path::PathBuf::from(id),
      content,
    });
  Ok((cm, source_file))
}
