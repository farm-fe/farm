pub use farmfe_svgr_rs::{Config, JSXRuntime, transform as react_compiler};

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
