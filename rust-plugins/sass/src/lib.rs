#![deny(clippy::all)]

use farmfe_core::serde_json::{self, Value};
use farmfe_core::{config::Config, module::ModuleType, plugin::Plugin};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{fs, regex::Regex, resolve::follow_symlinks};
use sass_embedded::{OutputStyle, Sass, StringOptions, StringOptionsBuilder};
use serde::{Deserialize, Serialize};
use std::env;
use std::env::consts::{ARCH, OS};
use std::path::PathBuf;

const PKG_NAME: &str = "@farmfe/plugin-sass";

#[farm_plugin]
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FarmPluginSass {
  sass_options: String,
}

impl FarmPluginSass {
  pub fn new(_config: &Config, options: String) -> Self {
    Self {
      sass_options: options,
    }
  }

  pub fn get_sass_options(&self) -> StringOptions {
    let options = serde_json::from_str(&self.sass_options).unwrap_or_default();
    get_options(options)
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
    let reg = Regex::new(r#"\.(sass|scss)$"#).unwrap();
    if reg.is_match(param.resolved_path) {
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
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if param.module_type == ModuleType::Custom(String::from("sass")) {
      let exe_path = get_exe_path();
      let mut sass = Sass::new(exe_path).unwrap_or_else(|e| {
        panic!(
          "\n sass-embedded init error: {},\n Please try to install manually. eg: \n pnpm install sass-embedded-{}-{} \n",
          e.message(),
          get_os(),
          get_arch()
        )
      });
      let string_options = self.get_sass_options();
      let compile_result = sass.compile_string(&param.content, string_options).unwrap();
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
  let manual_installation_path = cur_dir
    .join(format!("node_modules/sass-embedded-{os}-{arch}"))
    .join(format!("dart-sass-embedded/{entry_file}"));

  let default_path = follow_symlinks(PathBuf::from(cur_dir).join("node_modules").join(PKG_NAME))
    .join("../..")
    .join(format!("sass-embedded-{os}-{arch}"))
    .join(format!("dart-sass-embedded/{entry_file}"));

  if manual_installation_path.exists() {
    manual_installation_path
  } else {
    default_path
  }
}

fn get_options(options: Value) -> StringOptions {
  let mut builder = StringOptionsBuilder::new();

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
