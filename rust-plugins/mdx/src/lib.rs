#![deny(clippy::all)]

use farmfe_core::{config::Config, plugin::Plugin};

use farmfe_macro_plugin::farm_plugin;

#[farm_plugin]
pub struct FarmPluginMdx {}

impl FarmPluginMdx {
  fn new(config: &Config, options: String) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginMdx {
  fn name(&self) -> &str {
    "FarmPluginMdx"
  }
}
