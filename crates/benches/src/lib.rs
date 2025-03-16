use std::path::Path;

use farmfe_core::config::RuntimeConfig;

pub fn get_runtime_config(crate_path: &Path) -> Box<RuntimeConfig> {
  let swc_helpers_path = crate_path
    .join("benches")
    .join("fixtures")
    .join("_internal")
    .join("swc_helpers")
    .to_string_lossy()
    .to_string();
  let runtime_path = crate_path
    .join("benches")
    .join("fixtures")
    .join("_internal")
    .join("runtime")
    .to_string_lossy()
    .to_string();

  Box::new(RuntimeConfig {
    path: runtime_path,
    plugins: vec![],
    swc_helpers_path,
    ..Default::default()
  })
}
