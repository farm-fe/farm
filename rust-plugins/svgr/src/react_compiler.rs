pub use svgr_rs::{transform as _react_compiler, Config, JSXRuntime};

pub struct CompilerParams {
  pub svg: String,
  pub svg_name: Option<String>,
  pub root_path: Option<String>,
}

// TODO custom react component compiler
pub fn react_compiler(param: CompilerParams) -> String {
  let CompilerParams { svg, .. } = param;
  let code = _react_compiler(
    svg,
    Config {
      jsx_runtime: JSXRuntime::Classic,
      ..Default::default()
    },
    Default::default(),
  )
  .unwrap();
  code
}
