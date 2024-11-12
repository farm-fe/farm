#![deny(clippy::all)]

use farmfe_core::{config::Config, plugin::Plugin};

use farmfe_macro_plugin::farm_plugin;

#[derive(serde::Deserialize)]
pub struct Options {
  pub my_option: Option<String>,
}

#[farm_plugin]
pub struct FarmPluginExample {}

impl FarmPluginExample {
  fn new(config: &Config, options: String) -> Self {
    let opts: Options = serde_json::from_str(&options).unwrap();
    Self {}
  }
}

impl Plugin for FarmPluginExample {
  fn name(&self) -> &str {
    "FarmPluginExample"
  }

  fn priority(&self) -> i32 {
    101
  }

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    println!("resolve {:?} from {:?}", param.source, param.importer);
    Ok(None)
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    println!(
      "load path: {:?}, id: {:?}",
      param.resolved_path, param.module_id
    );
    Ok(None)
  }
}
