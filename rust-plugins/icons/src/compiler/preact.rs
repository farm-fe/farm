pub use svgr_rs::{transform as react_compiler, Config, JSXRuntime};

use super::CompilerParams;

pub fn preact_compiler(param: CompilerParams) -> String {
  let CompilerParams { svg, .. } = param;
  react_compiler(
    svg,
    Config {
      jsx_runtime: JSXRuntime::ClassicPreact,
      ..Default::default()
    },
    Default::default(),
  )
  .unwrap()
}
