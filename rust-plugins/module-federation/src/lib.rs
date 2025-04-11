#![deny(clippy::all)]

use farmfe_core::{
  config::Config, context::CompilationContext, error::CompilationError, plugin::Plugin,
  stats::Stats,
};

use std::sync::Arc;

use farmfe_macro_plugin::farm_plugin;

#[farm_plugin]
pub struct FarmfeModuleFederation {}

impl FarmfeModuleFederation {
  fn new(config: &Config, options: String) -> Self {
    Self {}
  }
}

impl Plugin for FarmfeModuleFederation {
  fn name(&self) -> &str {
    "FarmfeModuleFederation"
  }

  fn finish(
    &self,
    _stat: &Stats,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>, CompilationError> {
    println!("Module Federation Plugin Finish");
    Ok(None)
  }
}
