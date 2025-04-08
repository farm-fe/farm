use std::sync::Arc;

use farmfe_core::{
  context::create_swc_source_map,
  error::Result,
  module::ModuleId,
  swc_common::{Globals, SourceFile, SourceMap},
  swc_ecma_ast::EsVersion,
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::script::{parse_module, swc_try_with::try_with, ParseScriptModuleResult};

#[no_mangle]
pub fn farm_swc_parse_module(
  id: &ModuleId,
  content: Arc<String>,
  syntax: Syntax,
  target: EsVersion,
) -> Result<ParseScriptModuleResult> {
  parse_module(id, content, syntax, target)
}

#[no_mangle]
pub fn farm_create_swc_source_map(
  id: &ModuleId,
  content: Arc<String>,
) -> Result<(Arc<SourceMap>, Arc<SourceFile>)> {
  let (cm, source_file) = create_swc_source_map(id, content);
  Ok((cm, source_file))
}

#[no_mangle]
pub fn farm_swc_try_with(
  cm: Arc<SourceMap>,
  globals: &Globals,
  op: Box<dyn FnOnce()>,
) -> Result<()> {
  try_with(cm, globals, op)
}
