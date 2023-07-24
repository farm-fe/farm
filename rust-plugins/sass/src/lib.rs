#![deny(clippy::all)]

use farmfe_core::{
  config::Config,
  module::ModuleType,
  parking_lot::RwLock,
  plugin::Plugin,
  relative_path::RelativePath,
  serde_json::{self, Value},
};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{fs, regex::Regex, resolve::follow_symlinks};
use sass_embedded::{FileImporter, OutputStyle, Sass, StringOptions, StringOptionsBuilder, Url};
use std::env::consts::{ARCH, OS};
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, path::Path};

const PKG_NAME: &str = "@farmfe/plugin-sass";

#[farm_plugin]
pub struct FarmPluginSass {
  sass_options: String,
  regex: Regex,
}

impl FarmPluginSass {
  pub fn new(_config: &Config, options: String) -> Self {
    Self {
      sass_options: options,
      regex: Regex::new(r#"\.(sass|scss)$"#).unwrap(),
    }
  }

  pub fn get_sass_options(&self, resolve_path: String) -> StringOptions {
    let options = serde_json::from_str(&self.sass_options).unwrap_or_default();
    get_options(options, resolve_path)
  }
}

#[derive(Debug)]
struct FileImporterCollection {
  paths: Arc<RwLock<Vec<String>>>,
  cwd: String,
}

impl FileImporter for FileImporterCollection {
  fn find_file_url(
    &self,
    url: &str,
    _options: &sass_embedded::ImporterOptions,
  ) -> sass_embedded::Result<Option<Url>> {
    let resolved_path: String = if !Path::new(url).is_absolute() {
      RelativePath::new(url)
        .to_logical_path(&self.cwd)
        .to_string_lossy()
        .to_string()
    } else {
      url.to_string()
    };

    self.paths.write().push(resolved_path.clone());

    sass_embedded::Result::Ok(Some(Url::from_file_path(resolved_path).unwrap()))
  }
}

impl Plugin for FarmPluginSass {
  fn name(&self) -> &str {
    "FarmPluginSass"
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if self.regex.is_match(param.resolved_path) {
      let content = fs::read_file_utf8(param.resolved_path).unwrap();
      return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
        content,
        module_type: ModuleType::Custom(String::from("sass")),
      }));
    }
    Ok(None)
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if param.module_type == ModuleType::Custom(String::from("sass")) {
      let exe_path: PathBuf = get_exe_path();
      let mut sass = Sass::new(exe_path).unwrap_or_else(|e| {
        panic!(
          "\n sass-embedded init error: {},\n Please try to install manually. eg: \n pnpm install sass-embedded-{}-{} \n",
          e.message(),
          get_os(),
          get_arch()
        )
      });

      let mut string_options = self.get_sass_options(param.resolved_path.to_string());

      let paths = Arc::new(RwLock::new(vec![]));

      let import_collection = Box::new(FileImporterCollection {
        paths: paths.clone(),
        cwd: Path::new(param.resolved_path)
          .parent()
          .unwrap()
          .to_string_lossy()
          .to_string(),
      });

      string_options
        .common
        .importers
        .push(sass_embedded::SassImporter::FileImporter(import_collection));
      string_options.url = None;

      let compile_result = sass
        .compile_string(&param.content, string_options)
        .map_err(|e| farmfe_core::error::CompilationError::TransformError {
          resolved_path: param.resolved_path.to_string(),
          msg: e.message().to_string(),
        })?;

      let paths = paths.read();

      context
        .add_watch_files(param.resolved_path.to_string(), paths.iter().collect())
        .expect("cannot add file to watch graph");

      return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content: compile_result.css,
        source_map: None,
        module_type: Some(farmfe_core::module::ModuleType::Css),
      }));
    }
    Ok(None)
  }
}

fn get_os() -> &'static str {
  match OS {
    "linux" => "linux",
    "macos" => "darwin",
    "windows" => "win32",
    os => panic!("dart-sass-embed is not supported OS: {}", os),
  }
}

fn get_arch() -> &'static str {
  match ARCH {
    "x86" => "ia32",
    "x86_64" => "x64",
    "aarch64" => "arm64",
    arch => panic!("dart-sass-embed is not supported arch: {}", arch),
  }
}

fn get_exe_path() -> PathBuf {
  let os = get_os();
  let arch = get_arch();
  let entry_file = if let "win32" = os {
    "dart-sass-embedded.bat"
  } else {
    "dart-sass-embedded"
  };

  let cur_dir = env::current_dir().unwrap();

  // user manually installs related dependencies
  let manual_installation_path = RelativePath::new(&format!(
    "node_modules/sass-embedded-{os}-{arch}/dart-sass-embedded/{entry_file}"
  ))
  .to_logical_path(&cur_dir);

  let pkg_dir = follow_symlinks(
    RelativePath::new("node_modules")
      .join(PKG_NAME)
      .to_logical_path(&cur_dir),
  );

  // find closest node_modules start from pkg_dir
  let mut cur_dir = pkg_dir;

  while !cur_dir.join("node_modules").exists() {
    if cur_dir.parent().is_none() {
      panic!("can not find node_modules in @farmfe/plugin-sass");
    }

    cur_dir = cur_dir.parent().unwrap().to_path_buf();
  }

  let default_path = RelativePath::new("node_modules")
    .join(format!("sass-embedded-{os}-{arch}"))
    .join(format!("dart-sass-embedded/{entry_file}"))
    .to_logical_path(&cur_dir);

  if manual_installation_path.exists() {
    manual_installation_path
  } else {
    default_path
  }
}

fn get_options(options: Value, resolve_path: String) -> StringOptions {
  let mut builder = StringOptionsBuilder::new();
  builder = builder.url(Url::from_file_path(resolve_path).unwrap());
  if let Some(source_map) = options.get("sourceMap") {
    builder = builder.source_map(source_map.as_bool().unwrap());
  }
  if let Some(source_map_include_sources) = options.get("sourceMapIncludeSources") {
    builder = builder.source_map(source_map_include_sources.as_bool().unwrap());
  }
  if let Some(alert_ascii) = options.get("alertAscii") {
    builder = builder.alert_ascii(alert_ascii.as_bool().unwrap());
  }
  if let Some(alert_color) = options.get("alertColor") {
    builder = builder.alert_color(alert_color.as_bool().unwrap())
  }
  if let Some(charset) = options.get("charset") {
    builder = builder.charset(charset.as_bool().unwrap());
  }
  if let Some(quiet_deps) = options.get("quietDeps") {
    builder = builder.quiet_deps(quiet_deps.as_bool().unwrap());
  }
  if let Some(verbose) = options.get("verbose") {
    builder = builder.verbose(verbose.as_bool().unwrap());
  }
  if let Some(style) = options.get("style") {
    let output_style = match style.as_str().unwrap() {
      "expanded" => OutputStyle::Expanded,
      "compressed" => OutputStyle::Compressed,
      _ => panic!("sass stringOptions does not support this style configuration"),
    };
    builder = builder.style(output_style);
  }

  // TODO support more options

  builder.build()
}
