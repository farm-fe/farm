#![deny(clippy::all)]

use farmfe_core::{config::Config, plugin::Plugin};

use farmfe_macro_plugin::farm_plugin;

#[farm_plugin]
pub struct <FARM-RUST-PLUGIN-STRUCT-NAME> {}

impl <FARM-RUST-PLUGIN-STRUCT-NAME> {
  fn new(config: &Config, options: String) -> Self {
    Self {}
  }
}

impl Plugin for <FARM-RUST-PLUGIN-STRUCT-NAME> {
  fn name(&self) -> &str {
    "<FARM-RUST-PLUGIN-STRUCT-NAME>"
  }
}
