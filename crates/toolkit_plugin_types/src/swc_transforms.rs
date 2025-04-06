use std::sync::Arc;

use farmfe_core::{
  config::Mode,
  error::Result,
  swc_common::{comments::SingleThreadedComments, Globals, SourceMap},
  swc_ecma_ast,
};

pub struct FarmSwcTransformReactOptions<'a> {
  pub top_level_mark: u32,
  pub unresolved_mark: u32,
  pub inject_helpers: bool,
  pub mode: Mode,
  pub cm: Arc<SourceMap>,
  pub comments: SingleThreadedComments,
  pub globals: &'a Globals,
  pub options: String,
}

pub fn swc_transform_react(
  lib: &libloading::Library,
  ast: &mut swc_ecma_ast::Module,
  options: FarmSwcTransformReactOptions,
) -> Result<()> {
  unsafe {
    let func: libloading::Symbol<
      unsafe fn(&mut swc_ecma_ast::Module, FarmSwcTransformReactOptions) -> Result<()>,
    > = lib.get(b"farm_swc_transform_react").unwrap();
    func(ast, options)
  }
}
