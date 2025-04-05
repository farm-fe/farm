#![allow(clippy::needless_update)]

use std::{
  fs,
  path::{Path, PathBuf},
  sync::Arc,
};

use farmfe_compiler::Compiler;
use farmfe_core::{
  config::{
    bool_or_obj::BoolOrObj, config_regex::ConfigRegex, persistent_cache::PersistentCacheConfig,
    preset_env::PresetEnvConfig, Config, CssConfig, Mode, RuntimeConfig, SourcemapConfig,
  },
  plugin::Plugin,
  serde_json::{self, Value},
  HashMap,
};
use farmfe_testing_helpers::is_update_snapshot_from_env;
use farmfe_toolkit::fs::read_file_utf8;

pub fn generate_runtime(crate_path: PathBuf) -> Box<RuntimeConfig> {
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
    .to_string_lossy()
    .to_string();

  Box::new(RuntimeConfig {
    path: runtime_path,
    plugins: vec![],
    swc_helpers_path,
    ..Default::default()
  })
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
      output: Box::new(farmfe_core::config::OutputConfig {
        filename: "[resourceName].[ext]".to_string(),
        ..Default::default()
      }),
      mode: Mode::Production,
      external: vec![
        ConfigRegex::new("^react-refresh$"),
        ConfigRegex::new("^module$"),
      ],
      sourcemap: Box::new(SourcemapConfig::Bool(false)),
      css: Box::new(css_config),
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
pub fn merge_config(v1: Value, v2: Value) -> Value {
  match (v1, v2) {
    (Value::Array(mut a1), Value::Array(a2)) => {
      for item in a2 {
        a1.push(item);
      }

      Value::Array(a1)
    }

    (Value::Object(mut o1), Value::Object(o2)) => {
      for (key, val) in o2 {
        if let Some(left_value) = o1.remove(&key) {
          o1.insert(key, merge_config(left_value, val));
        } else {
          o1.insert(key, val);
        }
      }

      Value::Object(o1)
    }

    (_, v2) => v2,
  }
}

#[allow(dead_code)]
pub fn try_merge_config_file(origin: Config, file: PathBuf) -> Config {
  let config_from_file = try_read_config_from_json(file);

  if let Some(config_from_file) = config_from_file {
    let origin: Value = serde_json::from_str(&serde_json::to_string(&origin).unwrap()).unwrap();

    let origin = merge_config(origin, config_from_file);

    return serde_json::from_value(origin).unwrap();
  }

  origin
}

#[allow(dead_code)]
pub fn create_config(cwd: PathBuf, crate_path: PathBuf) -> Config {
  Config {
    input: HashMap::default(),
    root: cwd.to_string_lossy().to_string(),
    runtime: generate_runtime(crate_path),
    output: Default::default(),
    mode: Mode::Production,
    external: Default::default(),
    sourcemap: Box::new(SourcemapConfig::Bool(false)),
    lazy_compilation: false,
    progress: false,
    minify: Box::new(BoolOrObj::Bool(false)),
    preset_env: Box::new(PresetEnvConfig::Bool(false)),
    persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
    ..Default::default()
  }
}

#[allow(dead_code)]
pub fn try_read_config_from_json(path: PathBuf) -> Option<Value> {
  if !path.exists() {
    return None;
  }

  let Ok(content) = read_file_utf8(path.to_string_lossy().to_string().as_str()) else {
    return None;
  };

  farmfe_core::serde_json::from_str(&content).unwrap()
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
      output: Box::new(farmfe_core::config::OutputConfig {
        filename: "[resourceName].[ext]".to_string(),
        ..Default::default()
      }),
      mode: Mode::Production,
      external: vec![
        ConfigRegex::new("^react-refresh$"),
        ConfigRegex::new("^module$"),
        ConfigRegex::new("^vue$"),
        ConfigRegex::new("^fake-module$"),
      ],
      sourcemap: Box::new(SourcemapConfig::Bool(false)),
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
    .to_string_lossy()
    .to_string();

  let compiler = Compiler::new(
    Config {
      input,
      root: cwd.to_string_lossy().to_string(),
      runtime: Box::new(RuntimeConfig {
        path: runtime_path,
        plugins: vec![],
        swc_helpers_path,
        ..Default::default()
      }),
      external: vec![
        ConfigRegex::new("^react-refresh$"),
        ConfigRegex::new("^module$"),
      ],
      sourcemap: Box::new(SourcemapConfig::Bool(false)),
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

pub fn get_compiler_result(compiler: &Compiler, config: &AssertCompilerResultConfig) -> String {
  let resources_map = compiler.context().resources_map.lock();
  let mut result = vec![];

  for (name, resource) in resources_map.iter() {
    if !config.ignore_emitted_field && resource.emitted {
      continue;
    }

    result.push(match config.entry_name.as_ref() {
      Some(entry_name) if name == entry_name => (
        "1".into(),
        format!("//{}.{}:\n ", entry_name, resource.resource_type.to_ext()),
        String::from_utf8_lossy(&resource.bytes),
      ),
      _ => (
        format!("1{name}"),
        format!("//{name}:\n "),
        String::from_utf8_lossy(&resource.bytes),
      ),
    })
  }

  result.sort_by_key(|(raw_name, _, _)| raw_name.clone());

  let result_file_str = result
    .iter()
    .map(|(_, name, content)| format!("{name}{content}"))
    .collect::<Vec<String>>()
    .join("\n\n");

  result_file_str
}

#[allow(dead_code)]
pub fn load_expected_result(cwd: PathBuf, output_file: &String) -> String {
  std::fs::read_to_string(cwd.join(output_file)).unwrap_or("".to_string())
}

#[derive(Debug)]
pub struct AssertCompilerResultConfig {
  pub entry_name: Option<String>,
  pub ignore_emitted_field: bool,
  pub output_file: Option<String>,
}

impl Default for AssertCompilerResultConfig {
  fn default() -> Self {
    Self {
      entry_name: None,
      ignore_emitted_field: false,
      output_file: Some("output.js".to_string()),
    }
  }
}
impl AssertCompilerResultConfig {
  pub fn output_file(&self) -> String {
    self
      .output_file
      .clone()
      .unwrap_or_else(|| "output.js".to_string())
  }
}

#[allow(dead_code)]
pub fn assert_compiler_result_with_config(compiler: &Compiler, config: AssertCompilerResultConfig) {
  let output_path = config.output_file();
  let expected_result = load_expected_result(
    PathBuf::from(compiler.context().config.root.clone()),
    &output_path,
  );
  let result = get_compiler_result(compiler, &config);
  let output_path = PathBuf::from(compiler.context().config.root.clone()).join(output_path);
  if is_update_snapshot_from_env() || !output_path.exists() {
    std::fs::write(output_path, result).unwrap();
  } else {
    // assert lines are the same
    let expected_lines = expected_result.trim().lines().collect::<Vec<&str>>();
    let result_lines = result.trim().lines().collect::<Vec<&str>>();

    for (expected, result) in expected_lines.iter().zip(result_lines.iter()) {
      assert_eq!(expected.trim(), result.trim()); // ignore whitespace
    }

    assert_eq!(
      expected_lines.len(),
      result_lines.len(),
      "expect: \n{} result: \n{}",
      expected_result,
      result
    );
  }
}

#[allow(dead_code)]
pub fn assert_compiler_result(compiler: &Compiler, entry_name: Option<&String>) {
  assert_compiler_result_with_config(
    compiler,
    AssertCompilerResultConfig {
      entry_name: entry_name.cloned(),
      ..Default::default()
    },
  );
}

#[allow(dead_code)]
pub fn get_dir_config_files(cwd: &Path) -> Vec<(String, PathBuf)> {
  // println!("fs::read_dir(cwd): {:#?}", fs::read(format!("{}/", cwd.to_string_lossy().to_string())));
  let mut files = fs::read_dir(cwd)
    .map(|item| {
      item
        .into_iter()
        .filter_map(|file| match file {
          Ok(v) => Some(v),
          Err(_) => None,
        })
        .map(|v| v.path())
        .filter(|v| v.is_file())
        .filter(|v| {
          let m = v.file_name().unwrap().to_string_lossy().to_string();
          m.starts_with("config") && m.ends_with(".json")
        })
        .map(|v| {
          let m = v.file_name().unwrap().to_string_lossy().to_string();

          (
            m.trim_start_matches("config")
              .trim_start_matches('.')
              .trim_end_matches("json")
              .trim_end_matches('.')
              .to_string(),
            v,
          )
        })
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();

  if !files.iter().any(|(name, _)| name.is_empty()) {
    files.push(("".to_string(), cwd.to_path_buf().join("config.json")));
  }

  files
}

#[allow(dead_code)]
pub fn format_output_name(name: String) -> String {
  if name.is_empty() {
    return "output.js".to_string();
  }

  format!("output.{}.js", name)
}

#[allow(dead_code)]
pub struct TestBuilderOptions {
  pub cwd: PathBuf,
  pub file: String,
  pub plugins: Vec<Arc<dyn Plugin>>,
  pub config: Option<Config>,
  pub entry_name: String,
  pub crate_path: PathBuf,
}

impl TestBuilderOptions {
  #[allow(dead_code)]
  pub fn new(file: String, crate_path: PathBuf) -> Self {
    let filename = PathBuf::from(&file);
    let cwd = filename.parent().unwrap();
    Self {
      cwd: cwd.to_path_buf(),
      file,
      plugins: vec![],
      config: None,
      crate_path,
      entry_name: "index".to_string(),
    }
  }

  #[allow(dead_code)]
  pub fn with_plugins(mut self, plugins: Vec<Arc<dyn Plugin>>) -> Self {
    self.plugins = plugins;
    self
  }

  #[allow(dead_code)]
  pub fn with_config(mut self, config: Config) -> Self {
    self.config = Some(config);
    self
  }
}

#[allow(dead_code)]
pub fn test_builder(options: TestBuilderOptions) {
  let TestBuilderOptions {
    cwd,
    plugins: _plugins,
    config: _config,
    entry_name,
    crate_path,
    file,
  } = options;

  let files = get_dir_config_files(cwd.as_path());

  for (name, config_entry) in files {
    let compiler = create_compiler_with_args(
      cwd.clone(),
      crate_path.clone(),
      |mut config, mut plugins| {
        config.input = HashMap::from_iter([(entry_name.clone(), file.clone())]);

        if let Some(_config) = _config.clone() {
          let v1 = serde_json::to_value(config).expect("cannot convert config to value");
          let v2 = serde_json::to_value(_config).expect("cannot convert config to value");

          let v = merge_config(v1, v2);

          config = serde_json::from_value(v).expect("cannot convert value to config");
        }

        config = try_merge_config_file(config, config_entry);

        for plugin in _plugins.clone() {
          plugins.push(plugin);
        }

        (config, plugins)
      },
    );

    compiler.compile().unwrap();

    assert_compiler_result_with_config(
      &compiler,
      AssertCompilerResultConfig {
        entry_name: Some(entry_name.clone()),
        output_file: Some(format_output_name(name)),
        ignore_emitted_field: false,
        ..Default::default()
      },
    );
  }
}
