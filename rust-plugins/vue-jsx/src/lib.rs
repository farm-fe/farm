#![deny(clippy::all)]

use farmfe_core::relative_path::RelativePath;
use farmfe_core::serde_json::{self, Value};
use farmfe_core::{config::Config, module::ModuleType, plugin::Plugin};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{fs, regex::Regex, resolve::follow_symlinks};
use serde::{Deserialize, Serialize};
use std::env;
use std::env::consts::{ARCH, OS};
use std::path::PathBuf;

const PKG_NAME: &str = "@farmfe/plugin-sass";

#[farm_plugin]
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FarmPluginVueJsx {
  vue_jsx_options: String,
}

impl FarmPluginVueJsx {
  pub fn new(_config: &Config, options: String) -> Self {
    println!("FarmPluginVueJsx::new(options: {})", options);
    Self {
      vue_jsx_options: options,
    }
  }

  pub fn get_sass_options(&self) {
    let options = serde_json::from_str(&self.vue_jsx_options).unwrap_or_default();
    get_options(options)
  }
}

impl Plugin for FarmPluginVueJsx {
  fn name(&self) -> &str {
    "FarmPluginVueJsx"
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
    Ok(None)
  }
}

fn get_options(options: Value) {
  println!("{:?}", options);
}
