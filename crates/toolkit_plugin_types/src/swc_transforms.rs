use std::sync::Arc;

use farmfe_core::{context::CompilationContext, error::Result, swc_common::Mark, swc_ecma_ast};

pub struct FarmSwcTransformReactOptions {
  pub top_level_mark: u32,
  pub unresolved_mark: u32,
  pub inject_helpers: bool,
}

impl Default for FarmSwcTransformReactOptions {
  fn default() -> Self {
    Self {
      top_level_mark: Mark::fresh(Mark::root()).as_u32(),
      unresolved_mark: Mark::fresh(Mark::root()).as_u32(),
      inject_helpers: true,
    }
  }
}

pub fn swc_transform_react(
  lib: &libloading::Library,
  context: &Arc<CompilationContext>,
  ast: &mut swc_ecma_ast::Module,
  options: FarmSwcTransformReactOptions,
) -> Result<()> {
  unsafe {
    let func: libloading::Symbol<
      unsafe fn(
        &Arc<CompilationContext>,
        &mut swc_ecma_ast::Module,
        FarmSwcTransformReactOptions,
      ) -> Result<()>,
    > = lib.get(b"farm_swc_transform_react").unwrap();
    func(context, ast, options)
  }
}
