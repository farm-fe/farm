#![deny(clippy::all)]

use farmfe_core::{config::Config, module::ModuleType, plugin::Plugin};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{fs, regex::Regex};
use sass_embedded::{Sass, StringOptions};
use std::env;
use std::env::consts::{ARCH, OS};
use std::path::PathBuf;

const PKG_NAME: &str = "@farmfe/plugin-sass";

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
  let cur_dir = env::current_dir().unwrap().to_string_lossy().to_string();
  // user manually installs related dependencies
  let manual_installation_path = PathBuf::from(&cur_dir)
    .join(format!("node_modules/sass-embedded-{os}-{arch}"))
    .join(format!("dart-sass-embedded/{entry_file}"));

  let default_path = PathBuf::from(&cur_dir)
    .join(format!(
      "node_modules/{PKG_NAME}/node_modules/sass-embedded-{os}-{arch}"
    ))
    .join(format!("dart-sass-embedded/{entry_file}"));

  if manual_installation_path.exists() {
    manual_installation_path
  } else {
    default_path
  }
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
      let exe_path = get_exe_path();
      let mut sass = Sass::new(exe_path).unwrap_or_else(|e| {
        panic!(
          "\n sass-embedded init error: {},\n Please try to install manually. eg: {} \n ",
          e.message(),
          format!("pnpm install sass-embedded-{}-{}", get_os(), get_arch())
        )
      });

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
