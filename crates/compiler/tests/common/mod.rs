use std::{collections::HashMap, path::PathBuf, sync::Arc};

use farmfe_compiler::Compiler;
use farmfe_core::{
  config::{
    bool_or_obj::BoolOrObj,
    config_regex::ConfigRegex,
    external::{ExternalConfig, ExternalConfigItem},
    persistent_cache::PersistentCacheConfig,
    preset_env::PresetEnvConfig,
    Config, CssConfig, Mode, RuntimeConfig, SourcemapConfig,
  },
  plugin::Plugin,
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
    ..Default::default()
  }
}

#[allow(dead_code)]
pub fn create_css_compiler(
  input: HashMap<String, String>,
  cwd: PathBuf,
  crate_path: PathBuf,
  css_config: CssConfig,
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
      external: vec![
        ConfigRegex::new("^react-refresh$"),
        ConfigRegex::new("^module$"),
      ],
      sourcemap: SourcemapConfig::Bool(false),
      css: css_config,
      lazy_compilation: false,
      progress: false,
      minify: Box::new(BoolOrObj::Bool(false)),
      preset_env: Box::new(PresetEnvConfig::Bool(false)),
      persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
      ..Default::default()
    },
    vec![],
  )
  .unwrap();

  compiler
}

#[allow(dead_code)]
pub fn create_config(cwd: PathBuf, crate_path: PathBuf) -> Config {
  Config {
    input: HashMap::new(),
    root: cwd.to_string_lossy().to_string(),
    runtime: generate_runtime(crate_path),
    output: farmfe_core::config::OutputConfig::default(),
    mode: Mode::Production,
    external: Default::default(),
    sourcemap: SourcemapConfig::Bool(false),
    lazy_compilation: false,
    progress: false,
    minify: Box::new(BoolOrObj::Bool(false)),
    preset_env: Box::new(PresetEnvConfig::Bool(false)),
    persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
    ..Default::default()
  }
}

#[allow(dead_code)]
pub fn create_compiler_with_args<F>(cwd: PathBuf, crate_path: PathBuf, handle: F) -> Compiler
where
  F: FnOnce(Config, Vec<Arc<dyn Plugin>>) -> (Config, Vec<Arc<dyn Plugin>>),
{
  let config = create_config(cwd, crate_path);

  let plguins = vec![];

  let (config, plugins) = handle(config, plguins);
  Compiler::new(config, plugins).expect("faile to create compiler")
}

#[allow(dead_code)]
pub fn create_with_compiler(config: Config, plugin_adapters: Vec<Arc<dyn Plugin>>) -> Compiler {
  Compiler::new(config, plugin_adapters).expect("faile to create compiler")
}
#[allow(dead_code)]
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
      external: vec![
        ConfigRegex::new("^react-refresh$"),
        ConfigRegex::new("^module$"),
        ConfigRegex::new("^vue$"),
      ],
      sourcemap: SourcemapConfig::Bool(false),
      lazy_compilation: false,
      progress: false,
      minify: Box::new(BoolOrObj::from(minify)),
      preset_env: Box::new(PresetEnvConfig::Bool(false)),
      persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
      ..Default::default()
    },
    vec![],
  )
  .unwrap();

  compiler
}
#[allow(dead_code)]
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
        ..Default::default()
      },
      external: vec![
        ConfigRegex::new("^react-refresh$"),
        ConfigRegex::new("^module$"),
      ],
      sourcemap: SourcemapConfig::Bool(false),
      lazy_compilation: false,
      progress: false,
      minify: Box::new(BoolOrObj::from(minify)),
      preset_env: Box::new(PresetEnvConfig::Bool(false)),
      persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
      ..Default::default()
    },
    plugins,
  )
  .unwrap();

  compiler
}
#[allow(dead_code)]
pub fn get_compiler_result(compiler: &Compiler, entry_name: Option<&String>) -> String {
  let resources_map = compiler.context().resources_map.lock();
  let mut result = vec![];

  for (name, resource) in resources_map.iter() {
    if resource.emitted {
      continue;
    }

    result.push(match entry_name {
      Some(entry_name) if name == entry_name => (
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

#[allow(dead_code)]
pub fn load_expected_result(cwd: PathBuf) -> String {
  std::fs::read_to_string(cwd.join("output.js")).unwrap_or("".to_string())
}

#[allow(dead_code)]
pub fn assert_compiler_result(compiler: &Compiler, entry_name: Option<&String>) {
  let expected_result = load_expected_result(PathBuf::from(compiler.context().config.root.clone()));
  let result = get_compiler_result(compiler, entry_name);
  let output_path = PathBuf::from(compiler.context().config.root.clone()).join("output.js");
  if std::env::var("FARM_UPDATE_SNAPSHOTS").is_ok() || !output_path.exists() {
    std::fs::write(output_path, result).unwrap();
  } else {
    // assert lines are the same
    let expected_lines = expected_result.trim().lines().collect::<Vec<&str>>();
    let result_lines = result.trim().lines().collect::<Vec<&str>>();

    for (expected, result) in expected_lines.iter().zip(result_lines.iter()) {
      assert_eq!(expected.trim(), result.trim()); // ignore whitespace
    }

    assert_eq!(expected_lines.len(), result_lines.len());
  }
}
