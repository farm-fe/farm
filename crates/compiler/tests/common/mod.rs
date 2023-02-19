use std::path::PathBuf;

use farmfe_compiler::Compiler;
use farmfe_core::config::{Config, RuntimeConfig};

pub fn create_compiler(cwd: PathBuf, crate_path: PathBuf) -> Compiler {
  let swc_helpers_path = crate_path
    .join("tests")
    .join("fixtures")
    .join("_internal")
    .join("swc_helpers")
    .to_string_lossy()
    .to_string();
  let runtime_path = crate_path
    .join("tests")
    .join("fixtures")
    .join("_internal")
    .join("runtime")
    .join("index.js")
    .to_string_lossy()
    .to_string();

  let compiler = Compiler::new(
    Config {
      root: cwd.to_string_lossy().to_string(),
      runtime: RuntimeConfig {
        path: runtime_path,
        plugins: vec![],
        swc_helpers_path,
      },
      external: vec!["react-refresh".to_string(), "@swc/helpers".to_string()],
      ..Default::default()
    },
    vec![],
  )
  .unwrap();

  compiler
}
