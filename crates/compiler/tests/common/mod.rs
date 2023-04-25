use std::{collections::HashMap, path::PathBuf, sync::Arc};

use farmfe_compiler::Compiler;
use farmfe_core::{
  config::{Config, CssConfig, Mode, RuntimeConfig, SourcemapConfig},
  plugin::Plugin,
  resource::ResourceType,
};

pub fn generate_runtime(crate_path: PathBuf) -> RuntimeConfig {
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

  RuntimeConfig {
    path: runtime_path,
    plugins: vec![],
    swc_helpers_path,
  }
}

pub fn create_css_modules_compiler(
  input: HashMap<String, String>,
  cwd: PathBuf,
  crate_path: PathBuf,
  mode: Mode,
) -> Compiler {
  let compiler = Compiler::new(
    Config {
      input,
      root: cwd.to_string_lossy().to_string(),
      runtime: generate_runtime(crate_path),
      output: farmfe_core::config::OutputConfig {
        filename: "[resourceName].[ext]".to_string(),
        ..Default::default()
      },
      mode,
      external: vec!["react-refresh".to_string(), "module".to_string()],
      sourcemap: SourcemapConfig::Bool(false),
      css: CssConfig {
        modules: true,
        ..Default::default()
      },
      lazy_compilation: false,
      minify: false,
      ..Default::default()
    },
    vec![],
  )
  .unwrap();

  compiler
}

pub fn create_compiler(
  input: HashMap<String, String>,
  cwd: PathBuf,
  crate_path: PathBuf,
  minify: bool,
) -> Compiler {
  let compiler = Compiler::new(
    Config {
      input,
      root: cwd.to_string_lossy().to_string(),
      runtime: generate_runtime(crate_path),
      output: farmfe_core::config::OutputConfig {
        filename: "[resourceName].[ext]".to_string(),
        ..Default::default()
      },
      mode: Mode::Production,
      external: vec!["react-refresh".to_string(), "module".to_string()],
      sourcemap: SourcemapConfig::Bool(false),
      lazy_compilation: false,
      minify,
      preset_env: false,
      ..Default::default()
    },
    vec![],
  )
  .unwrap();

  compiler
}

pub fn create_compiler_with_plugins(
  input: HashMap<String, String>,
  cwd: PathBuf,
  crate_path: PathBuf,
  minify: bool,
  plugins: Vec<Arc<(dyn Plugin + 'static)>>,
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
      lazy_compilation: false,
      minify,
      ..Default::default()
    },
    plugins,
  )
  .unwrap();

  compiler
}

pub fn get_compiler_result(compiler: &Compiler, entry_name: Option<&String>) -> String {
  let resources_map = compiler.context().resources_map.lock();
  let mut result = vec![];

  for (name, resource) in resources_map.iter() {
    if matches!(resource.resource_type, ResourceType::Runtime) {
      continue;
    }

    result.push(match entry_name {
      Some(entry_name) if name.starts_with(entry_name) => (
        "1".into(),
        format!("//{}.{}:\n ", entry_name, resource.resource_type.to_ext()),
        String::from_utf8_lossy(&resource.bytes),
      ),
      _ => (
        format!("1{}", name),
        format!("//{}:\n ", name),
        String::from_utf8_lossy(&resource.bytes),
      ),
    })
  }

  result.sort_by_key(|(raw_name, _, _)| raw_name.clone());

  let result_file_str = result
    .iter()
    .map(|(_, name, content)| format!("{}{}", name, content))
    .collect::<Vec<String>>()
    .join("\n\n");

  result_file_str
}

pub fn load_expected_result(cwd: PathBuf) -> String {
  let expected_result = std::fs::read_to_string(cwd.join("output.js")).unwrap_or("".to_string());
  expected_result
}

pub fn assert_compiler_result(compiler: &Compiler, entry_name: Option<&String>) {
  let expected_result = load_expected_result(PathBuf::from(compiler.context().config.root.clone()));
  let result = get_compiler_result(compiler, entry_name);

  if std::env::var("FARM_UPDATE_SNAPSHOTS").is_ok() {
    std::fs::write(
      PathBuf::from(compiler.context().config.root.clone()).join("output.js"),
      result,
    )
    .unwrap();
  } else {
    // assert lines are the same
    let expected_lines = expected_result.trim().lines().collect::<Vec<&str>>();
    let result_lines = result.trim().lines().collect::<Vec<&str>>();

    assert_eq!(expected_lines.len(), result_lines.len());

    for (expected, result) in expected_lines.iter().zip(result_lines.iter()) {
      assert_eq!(expected.trim(), result.trim()); // ignore whitespace
    }
  }
}
