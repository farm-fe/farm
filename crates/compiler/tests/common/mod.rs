use std::{collections::HashMap, path::PathBuf};

use farmfe_compiler::Compiler;
use farmfe_core::{
  config::{Config, RuntimeConfig, SourcemapConfig},
  resource::ResourceType,
};

pub fn create_compiler(
  input: HashMap<String, String>,
  cwd: PathBuf,
  crate_path: PathBuf,
) -> Compiler {
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
      input,
      root: cwd.to_string_lossy().to_string(),
      runtime: RuntimeConfig {
        path: runtime_path,
        plugins: vec![],
        swc_helpers_path,
      },
      external: vec!["react-refresh".to_string(), "module".to_string()],
      sourcemap: SourcemapConfig::Bool(false),
      ..Default::default()
    },
    vec![],
  )
  .unwrap();

  compiler
}

pub fn get_compiler_result(compiler: &Compiler) -> String {
  let resources_map = compiler.context().resources_map.lock();
  let mut result = String::new();

  for (name, resource) in resources_map.iter() {
    if matches!(resource.resource_type, ResourceType::Runtime) {
      continue;
    }

    result.push_str(&format!(
      "//{}:\n {}\n\n",
      name,
      String::from_utf8_lossy(&resource.bytes)
    ));
  }

  result
}

pub fn load_expected_result(cwd: PathBuf) -> String {
  let expected_result = std::fs::read_to_string(cwd.join("output.js")).unwrap();
  expected_result
}

pub fn assert_compiler_result(compiler: &Compiler) {
  let expected_result = load_expected_result(PathBuf::from(compiler.context().config.root.clone()));
  let result = get_compiler_result(compiler);

  if std::env::var("FARM_UPDATE_SNAPSHOTS").is_ok() {
    std::fs::write(
      PathBuf::from(compiler.context().config.root.clone()).join("output.js"),
      result,
    )
    .unwrap();
  } else {
    // assert lines are the same
    let expected_lines: Vec<&str> = expected_result.trim().lines().collect();
    let lines: Vec<&str> = result.trim().lines().collect();

    assert_eq!(lines.len(), expected_lines.len());

    for (line, expected_line) in lines.iter().zip(expected_lines.iter()) {
      assert_eq!(line.trim(), expected_line.trim());
    }
  }
}
