#![deny(clippy::all)]

use farmfe_core::{config::Config, module::ModuleType, plugin::Plugin};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{fs, regex::Regex};
use sass_embedded::{Sass, StringOptions};
use std::env::consts::OS;

fn exe_path() -> std::path::PathBuf {
  let file_name = if OS == "windows" {
    "dart-sass-embedded.bat"
  } else {
    "dart-sass-embedded"
  };
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("ext/sass/sass-embedded")
    .join(file_name)
}

#[farm_plugin]
pub struct FarmPluginSass {}

impl FarmPluginSass {
  pub fn new(_config: &Config, _options: String) -> Self {
    Self {}
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
    let reg = Regex::new(r#"\.scss$"#).unwrap();
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
      let mut sass = Sass::new(exe_path()).unwrap();
      let res = sass
        .compile_string(&param.content, StringOptions::default())
        .unwrap();
      return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content: res.css,
        source_map: None,
        module_type: Some(farmfe_core::module::ModuleType::Css),
      }));
    }
    Ok(None)
  }
}
