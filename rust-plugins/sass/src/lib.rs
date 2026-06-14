#![deny(clippy::all)]
#![allow(clippy::result_large_err)]

use std::{
  collections::HashSet,
  fmt::{Debug, Formatter},
  io,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::{ModuleId, ModuleType},
  plugin::{Plugin, PluginHookContext, PluginResolveHookParam, ResolveKind},
  serde_json::{self, Value},
};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{
  fs::{self, read_file_utf8},
  regex::Regex,
};
use farmfe_utils::relative;
use grass::{Fs, InputSyntax, Options as GrassOptions, OutputStyle};
use rebase_urls::rebase_urls;

mod rebase_urls;

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
}

struct FarmFs {
  root_file: PathBuf,
  root_importer: ModuleId,
  root_content: String,
  additional_data: Option<String>,
  context: Arc<CompilationContext>,
  watched_files: Mutex<HashSet<PathBuf>>,
}

impl Debug for FarmFs {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("FarmFs")
      .field("root_file", &self.root_file)
      .field("root_importer", &self.root_importer)
      .finish()
  }
}

impl FarmFs {
  fn new(
    root_file: &str,
    root_importer: ModuleId,
    root_content: String,
    additional_data: Option<String>,
    context: Arc<CompilationContext>,
  ) -> Self {
    Self {
      root_file: PathBuf::from(root_file),
      root_importer,
      root_content,
      additional_data,
      context,
      watched_files: Mutex::new(HashSet::new()),
    }
  }

  fn read_root(&self) -> Vec<u8> {
    let content = if let Some(additional_data) = &self.additional_data {
      format!("{additional_data}\n{}", self.root_content)
    } else {
      self.root_content.clone()
    };

    content.into_bytes()
  }

  fn resolve_path(&self, path: &Path) -> Option<PathBuf> {
    if same_path(path, &self.root_file) {
      return Some(self.root_file.clone());
    }

    if path.is_file() {
      return Some(path.to_path_buf());
    }

    let source = path_to_resolve_source(path, &self.context.config.root);
    resolve_importer(source, &self.root_importer, &self.context).ok()?
  }

  fn watched_module_ids(&self) -> Vec<ModuleId> {
    self
      .watched_files
      .lock()
      .expect("cannot lock sass watched files")
      .iter()
      .filter(|path| !same_path(path, &self.root_file))
      .map(|path| ModuleId::new(&path.to_string_lossy(), "", &self.context.config.root))
      .collect()
  }

  fn watched_paths(&self) -> Vec<PathBuf> {
    self
      .watched_files
      .lock()
      .expect("cannot lock sass watched files")
      .iter()
      .filter(|path| !same_path(path, &self.root_file))
      .cloned()
      .collect()
  }
}

impl Fs for FarmFs {
  fn is_dir(&self, path: &Path) -> bool {
    path.is_dir()
  }

  fn is_file(&self, path: &Path) -> bool {
    self.resolve_path(path).is_some()
  }

  fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
    let resolved_path = self.resolve_path(path).ok_or_else(|| {
      io::Error::new(
        io::ErrorKind::NotFound,
        format!("can not resolve {}", path.to_string_lossy()),
      )
    })?;

    self
      .watched_files
      .lock()
      .expect("cannot lock sass watched files")
      .insert(resolved_path.clone());

    if same_path(&resolved_path, &self.root_file) {
      return Ok(self.read_root());
    }

    let resolved_path_string = resolved_path.to_string_lossy().to_string();
    let root_file_string = self.root_file.to_string_lossy().to_string();
    let content =
      read_file_utf8(&resolved_path_string).map_err(|error| io::Error::other(error.to_string()))?;

    rebase_urls(
      &resolved_path_string,
      &root_file_string,
      content,
      &self.context,
    )
    .map(String::into_bytes)
    .map_err(|error| io::Error::other(error.to_string()))
  }

  fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
    self
      .resolve_path(path)
      .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path.to_string_lossy().to_string()))
  }
}

impl Debug for FarmPluginSass {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("FarmPluginSass").finish()
  }
}

fn same_path(a: &Path, b: &Path) -> bool {
  if a == b {
    return true;
  }

  match (a.canonicalize(), b.canonicalize()) {
    (Ok(a), Ok(b)) => a == b,
    _ => false,
  }
}

fn path_to_resolve_source(path: &Path, root: &str) -> String {
  let path_string = path.to_string_lossy().replace('\\', "/");

  if let Some(index) = path_string.find("/@/") {
    return path_string[index + 1..].to_string();
  }

  if path_string.starts_with("@/") {
    return path_string;
  }

  if path.is_absolute() {
    return relative(root, &path_string);
  }

  path_string
}

fn resolve_importer_with_prefix(
  mut url: PathBuf,
  prefix: &str,
  root_importer: &ModuleId,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<Option<PathBuf>> {
  if let Some(filename) = url.file_name() {
    url.set_file_name(format!("{}{}", prefix, filename.to_string_lossy()));
  }

  context
    .plugin_driver
    .resolve(
      &PluginResolveHookParam {
        source: url.to_string_lossy().to_string().replace('\\', "/"),
        importer: Some(root_importer.clone()),
        kind: ResolveKind::CssAtImport,
      },
      context,
      &PluginHookContext::default(),
    )
    .map(|item| item.map(|item| PathBuf::from(item.resolved_path)))
}

fn resolve_importer(
  url: String,
  root_importer: &ModuleId,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<Option<PathBuf>> {
  let file_path = PathBuf::from(&url);
  let default_import_result =
    resolve_importer_with_prefix(file_path.clone(), "", root_importer, context);

  if matches!(default_import_result, Ok(Some(_))) {
    return default_import_result;
  }

  let resolved_path = resolve_importer_with_prefix(file_path, "_", root_importer, context);

  if matches!(resolved_path, Ok(Some(_))) {
    return resolved_path;
  }

  default_import_result
}

fn is_sass_module_type(module_type: &ModuleType) -> bool {
  matches!(module_type, ModuleType::Custom(t) if t == "sass" || t == "scss")
}

impl Plugin for FarmPluginSass {
  fn name(&self) -> &str {
    "FarmPluginSass"
  }

  // this plugin should be executed before internal plugins
  fn priority(&self) -> i32 {
    101
  }

  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    let extra_extensions = vec!["sass", "scss"];

    for ext in extra_extensions {
      if config.resolve.extensions.iter().all(|e| e != ext) {
        config.resolve.extensions.push(ext.to_string());
      }
    }

    Ok(Some(()))
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if param.query.is_empty() && self.regex.is_match(param.resolved_path) {
      let content = fs::read_file_utf8(param.resolved_path);

      if let Ok(content) = content {
        return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
          content,
          module_type: ModuleType::Custom(String::from("sass")),
          source_map: None,
        }));
      }
    }

    Ok(None)
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if is_sass_module_type(&param.module_type) {
      let options: Value = serde_json::from_str(&self.sass_options).unwrap_or_default();
      let additional_data = options
        .get("additionalData")
        .and_then(Value::as_str)
        .map(ToString::to_string);
      let fs = FarmFs::new(
        param.resolved_path,
        param.module_id.clone().into(),
        param.content.clone(),
        additional_data,
        context.clone(),
      );
      let grass_options = get_options(options, param.resolved_path, &fs);
      let mut compile_result =
        grass::from_path(param.resolved_path, &grass_options).map_err(|e| {
          farmfe_core::error::CompilationError::TransformError {
            resolved_path: param.resolved_path.to_string(),
            msg: e.to_string(),
          }
        })?;
      for path in fs.watched_paths() {
        compile_result = rebase_urls(
          &path.to_string_lossy(),
          param.resolved_path,
          compile_result,
          context,
        )?;
      }
      let watched_files = fs.watched_module_ids();

      context
        .add_watch_files(
          ModuleId::new(param.resolved_path, "", &context.config.root),
          watched_files,
        )
        .expect("cannot add file to watch graph");

      return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content: compile_result,
        source_map: None,
        module_type: Some(farmfe_core::module::ModuleType::Css),
        ignore_previous_source_map: false,
      }));
    }
    Ok(None)
  }
}

fn get_options<'a>(options: Value, resolved_path: &str, fs: &'a FarmFs) -> GrassOptions<'a> {
  let mut grass_options = GrassOptions::default().fs(fs);

  if let Some(charset) = options.get("charset") {
    grass_options = grass_options.allows_charset(charset.as_bool().unwrap_or(true));
  }

  if let Some(quiet_deps) = options.get("quietDeps") {
    grass_options = grass_options.quiet(quiet_deps.as_bool().unwrap_or(false));
  }

  if let Some(style) = options.get("style") {
    let output_style = match style.as_str().unwrap_or("expanded") {
      "expanded" => OutputStyle::Expanded,
      "compressed" => OutputStyle::Compressed,
      _ => panic!("sass stringOptions does not support this style configuration"),
    };
    grass_options = grass_options.style(output_style);
  }

  if resolved_path.ends_with(".sass") {
    grass_options = grass_options.input_syntax(InputSyntax::Sass);
  }

  grass_options
}
