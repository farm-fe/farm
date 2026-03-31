pub use svgr_rs::{transform as _react_compiler, Config, JSXRuntime};

use super::CompilerParams;

pub fn react_compiler(param: CompilerParams) -> String {
  let CompilerParams { svg, .. } = param;
  _react_compiler(
    svg,
    Config {
      jsx_runtime: JSXRuntime::Classic,
      ..Default::default()
    },
    Default::default(),
  )
  .unwrap()
}
