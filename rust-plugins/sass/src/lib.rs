#![deny(clippy::all)]
#![allow(clippy::result_large_err)]
use std::{collections::HashMap, str::FromStr};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::{ModuleId, ModuleType},
  plugin::{Plugin, PluginHookContext, PluginResolveHookParam, ResolveKind},
  relative_path::RelativePath,
  serde_json::{self, Value},
};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{
  fs::{self, read_file_utf8},
  regex::Regex,
};
use farmfe_utils::relative;
use rebase_urls::rebase_urls;
use sass_embedded::{
  Exception, Importer, OutputStyle, Sass, StringOptions, StringOptionsBuilder, Syntax, Url,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fmt::Debug};
use std::{
  env::consts::{ARCH, OS},
  fmt::Formatter,
};

const PKG_NAME: &str = "@farmfe/plugin-sass";

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

  fn get_sass_options(
    &self,
    resolved_path_with_query: String,
    sourcemap_enabled: bool,
  ) -> (StringOptions, HashMap<String, String>) {
    let options = serde_json::from_str(&self.sass_options).unwrap_or_default();
    get_options(options, resolved_path_with_query, sourcemap_enabled)
  }
}

struct ImporterCollection {
  root_importer: ModuleId,
  context: Arc<CompilationContext>,
}

fn extension_from_path(path: &str) -> Syntax {
  match PathBuf::from(path).extension().and_then(|ext| ext.to_str()) {
    Some("sass") => Syntax::Indented,
    Some("css") => Syntax::Css,
    Some("scss") => Syntax::Scss,
    _ => Syntax::Scss,
  }
}

fn resolve_importer_with_prefix(
  mut url: PathBuf,
  prefix: &str,
  root_importer: &ModuleId,
  context: &Arc<CompilationContext>,
) -> Result<Option<String>, Exception> {
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
    .map_err(|e| {
      Exception::new(format!(
        "can not resolve {:?} from {:?}: Error: {:?}",
        url.to_string_lossy().to_string(),
        prefix,
        e
      ))
    })
    .map(|item| item.map(|item| item.resolved_path))
}

fn resolve_importer(
  url: String,
  root_importer: &ModuleId,
  context: &Arc<CompilationContext>,
) -> Result<Option<String>, Exception> {
  if let Ok(file_path) = PathBuf::from_str(&url) {
    let try_prefix_list = ["_"];

    let default_import_result =
      resolve_importer_with_prefix(file_path.clone(), "", root_importer, context);

    if matches!(default_import_result, Ok(Some(_))) {
      return default_import_result;
    }

    for prefix in try_prefix_list {
      let resolved_path =
        resolve_importer_with_prefix(file_path.clone(), prefix, root_importer, context);

      if matches!(resolved_path, Ok(Some(_))) {
        return resolved_path;
      }
    }

    return default_import_result;
  };

  Ok(None)
}

impl ImporterCollection {
  fn load(&self, resolved_path: &str) -> Result<Option<String>, Box<Exception>> {
    let context = &self.context;

    if let Ok(file_content) = read_file_utf8(resolved_path) {
      let root_file = self.root_importer.resolved_path(&context.config.root);

      return Ok(Some(rebase_urls(
        resolved_path,
        &root_file,
        file_content,
        context,
      )?));
    }

    Ok(None)
  }
}

impl Importer for ImporterCollection {
  fn canonicalize(
    &self,
    url: &str,
    _options: &sass_embedded::ImporterOptions,
  ) -> sass_embedded::Result<Option<Url>> {
    let url = if url.strip_prefix("file:").is_some()
      || url.starts_with('/')
      || PathBuf::from(url).is_absolute()
    {
      if let Ok(url) = Url::from_str(url) {
        url
      } else {
        Url::from_file_path(url)
          .map_err(|_| Exception::new(format!("parse raw {url:?} to Url failed.")))?
      }
    } else {
      let resolved_path = RelativePath::new(url).to_logical_path(&self.context.config.root);
      Url::from_file_path(&resolved_path)
        .map_err(|_| Exception::new(format!("parse {resolved_path:?} to Url failed.")))?
    };

    Ok(Some(url))
  }

  fn load(
    &self,
    canonical_url: &Url,
  ) -> sass_embedded::Result<Option<sass_embedded::ImporterResult>> {
    let url = relative(
      &self.context.config.root,
      &canonical_url.to_file_path().unwrap().to_string_lossy(),
    );

    if let Some(resolve_result) = resolve_importer(url, &self.root_importer, &self.context)? {
      let content = self.load(&resolve_result)?;

      if let Some(file_content) = content {
        return Ok(Some(sass_embedded::ImporterResult {
          contents: file_content,
          source_map_url: None,
          syntax: extension_from_path(&resolve_result),
        }));
      }
    };

    Ok(None)
  }
}

impl Debug for ImporterCollection {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ImporterCollection")
      .field("root_importer", &self.root_importer)
      .finish()
  }
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
    if param.module_type == ModuleType::Custom(String::from("sass")) {
      let exe_path: PathBuf = get_exe_path(context);
      let mut sass = Sass::new(exe_path).unwrap_or_else(|e| {
        panic!(
          "\n sass-embedded init error: {},\n Please try to install manually. eg: \n pnpm install sass-embedded-{}-{} \n",
          e.message(),
          get_os(),
          get_arch()
        )
      });

      let (mut string_options, additional_options) = self.get_sass_options(
        ModuleId::from(param.module_id.as_str()).resolved_path_with_query(&context.config.root),
        context.sourcemap_enabled(&param.module_id.to_string()),
      );

      let cloned_context = context.clone();

      let import_collection = Box::new(ImporterCollection {
        root_importer: param.module_id.clone().into(),
        context: cloned_context,
      });
      // TODO support source map for additionalData
      let content = if let Some(additional_data) = additional_options.get("additionalData") {
        format!("{}\n{}", additional_data, param.content)
      } else {
        param.content.clone()
      };

      string_options
        .common
        .importers
        .push(sass_embedded::SassImporter::Importer(import_collection));
      string_options.url = Some(Url::from_file_path(param.resolved_path).unwrap());

      let compile_result = sass.compile_string(&content, string_options).map_err(|e| {
        farmfe_core::error::CompilationError::TransformError {
          resolved_path: param.resolved_path.to_string(),
          msg: e.message().to_string(),
        }
      })?;

      let paths = compile_result
        .loaded_urls
        .iter()
        .map(|url| url.path())
        .filter(|p| p != &param.resolved_path)
        .map(|p| ModuleId::new(p, "", &context.config.root))
        .collect();

      context
        .add_watch_files(
          ModuleId::new(param.resolved_path, "", &context.config.root),
          paths,
        )
        .expect("cannot add file to watch graph");

      return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content: compile_result.css,
        source_map: compile_result.source_map,
        module_type: Some(farmfe_core::module::ModuleType::Css),
        ignore_previous_source_map: false,
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
    os => panic!("dart-sass-embed is not supported OS: {os}"),
  }
}

fn get_arch() -> &'static str {
  match ARCH {
    "x86" => "ia32",
    "x86_64" => "x64",
    "aarch64" => "arm64",
    arch => panic!("dart-sass-embed is not supported arch: {arch}"),
  }
}

fn get_exe_path(context: &Arc<CompilationContext>) -> PathBuf {
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

  if manual_installation_path.exists() {
    return manual_installation_path;
  }

  let pkg_dir = context
    .plugin_driver
    .resolve(
      &PluginResolveHookParam {
        source: PKG_NAME.to_string(),
        importer: None,
        kind: ResolveKind::Import,
      },
      context,
      &PluginHookContext::default(),
    )
    .unwrap()
    .unwrap_or_else(|| {
      panic!(
        "can not resolve @farmfe/plugin-sass start from {}",
        cur_dir.to_string_lossy()
      )
    })
    .resolved_path;

  // find closest node_modules start from pkg_dir
  let mut cur_dir = PathBuf::from(pkg_dir).parent().unwrap().to_path_buf();

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

  default_path
}

fn get_options(
  options: Value,
  resolved_path_with_query: String,
  sourcemap_enabled: bool,
) -> (StringOptions, HashMap<String, String>) {
  let mut builder = StringOptionsBuilder::new();
  builder = builder.url(
    Url::from_file_path(&resolved_path_with_query)
      .unwrap_or_else(|e| panic!("invalid path: {resolved_path_with_query}. Error: {e:?}")),
  );
  if let Some(source_map) = options.get("sourceMap") {
    builder = builder.source_map(source_map.as_bool().unwrap_or(sourcemap_enabled));
  } else {
    builder = builder.source_map(sourcemap_enabled);
  }
  if let Some(source_map_include_sources) = options.get("sourceMapIncludeSources") {
    builder =
      builder.source_map_include_sources(source_map_include_sources.as_bool().unwrap_or(true));
  } else {
    builder = builder.source_map_include_sources(true);
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

  let mut additional_data = HashMap::new();
  // TODO support sourcemap for additionalData
  if let Some(additional_date) = options.get("additionalData") {
    additional_data.insert(
      "additionalData".to_string(),
      additional_date.as_str().unwrap().to_string(),
    );
  }

  if resolved_path_with_query.ends_with(".sass") {
    builder = builder.syntax(Syntax::Indented);
  }

  // TODO support more options

  (builder.build(), additional_data)
}
