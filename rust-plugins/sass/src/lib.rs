#![deny(clippy::all)]

use std::path::Path;

use farmfe_core::{config::Config, module::ModuleType, plugin::Plugin, serde_json};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{fs, regex::Regex};
use grass;
use serde_json::Value;

#[farm_plugin]
pub struct FarmPluginSass {
  options: String,
  root: String,
  regex: Regex,
}

impl FarmPluginSass {
  pub fn new(config: &Config, options: String) -> Self {
    Self {
      options: options,
      root: config.root.clone(),
      regex: Regex::new(r#"\.(sass|scss)$"#).unwrap(),
    }
  }

  pub fn get_sass_options(&self) -> grass::Options {
    let options: Value = serde_json::from_str(&self.options).unwrap_or_default();
    let mut sass_options = grass::Options::default();

    if let Value::Bool(quiet) = options.get("quiet").unwrap_or(&Value::Null) {
      sass_options = sass_options.quiet(*quiet);
    }

    if let Value::Bool(allows_charset) = options.get("allows_charset").unwrap_or(&Value::Null) {
      sass_options = sass_options.allows_charset(*allows_charset);
    }

    if let Value::Bool(unicode_error_messages) = options
      .get("unicode_error_messages")
      .unwrap_or(&Value::Null)
    {
      sass_options = sass_options.unicode_error_messages(*unicode_error_messages);
    }

    let mut paths = vec![Path::new(&self.root)];

    if let Value::Array(load_paths) = options.get("load_paths").unwrap_or(&Value::Null) {
      for path in load_paths {
        if let Value::String(path) = path {
          paths.push(Path::new(path));
        }
      }
    }

    sass_options = sass_options.load_paths(&paths);
    sass_options
  }
}

struct FileImporterCollection {
  paths: Arc<RwLock<Vec<String>>>,
  importer: ModuleId,
  context: Arc<CompilationContext>,
}

impl Debug for FileImporterCollection {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("FileImporterCollection")
      .field("paths", &self.paths)
      .field("importer", &self.importer)
      .finish()
  }
}

impl FileImporter for FileImporterCollection {
  fn find_file_url(
    &self,
    url: &str,
    _options: &sass_embedded::ImporterOptions,
  ) -> sass_embedded::Result<Option<Url>> {
    let context = &self.context;
    // try to resolve url using relative path first
    let importer_path = PathBuf::from(self.importer.resolved_path(&context.config.root));
    let importer_dir = importer_path.parent().unwrap();
    let relative_url = RelativePath::new(url);
    let resolved_url = relative_url.to_logical_path(importer_dir);

    if resolved_url.exists() {
      let mut paths = self.paths.write();
      paths.push(resolved_url.to_string_lossy().to_string());
      return Ok(Some(Url::from_file_path(resolved_url).unwrap()));
    }

    let url = if url.starts_with('~') {
      url.replacen('~', "", 1)
    } else {
      url.to_string()
    };

    let resolve_result = context
      .plugin_driver
      .resolve(
        &PluginResolveHookParam {
          source: url,
          importer: Some(self.importer.clone()),
          kind: ResolveKind::CssAtImport,
        },
        context,
        &PluginHookContext::default(),
      )
      .unwrap();

    if let Some(resolve_result) = resolve_result {
      let mut paths = self.paths.write();
      paths.push(resolve_result.resolved_path.clone());
      return Ok(Some(
        Url::from_file_path(resolve_result.resolved_path).unwrap(),
      ));
    }

    sass_embedded::Result::Ok(None)
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
      let sass_options = self.get_sass_options();
      let css = grass::from_string(&param.content.to_owned(), &sass_options).map_err(|e| {
        farmfe_core::error::CompilationError::TransformError {
          resolved_path: param.resolved_path.to_string(),
          msg: e.to_string(),
        }
      })?;
      return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content: css,
        source_map: None,
        module_type: Some(farmfe_core::module::ModuleType::Css),
      }));
    }
    Ok(None)
  }
}
