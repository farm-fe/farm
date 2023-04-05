#![deny(clippy::all)]

use farmfe_core::{config::Config, plugin::Plugin};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::regex::Regex;
use sass_embedded::{Options, Sass};

fn exe_path() -> std::path::PathBuf {
  std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join("ext/sass/sass-embedded")
    .join("dart-sass-embedded")
}

#[farm_plugin]
pub struct FarmPluginSass {}

impl FarmPluginSass {
  fn new(_config: &Config, _options: String) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginSass {
  fn name(&self) -> &str {
    "FarmPluginReact"
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    let reg = Regex::new(r#"\.scss$"#).unwrap();
    if reg.is_match(param.resolved_path) {
      println!("【 exe_path() 】==> {:?}", exe_path());
      let mut sass = Sass::new(exe_path()).unwrap();
      let res = sass
        .compile(&param.resolved_path, Options::default())
        .unwrap();
      return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
        content: res.css,
        module_type: farmfe_core::module::ModuleType::Css,
      }));
    }
    Ok(None)
  }
}
