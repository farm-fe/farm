use farmfe_core::{config::Config, plugin::Plugin};

#[derive(Default)]
pub struct FarmPluginLibrary {}

impl FarmPluginLibrary {
  pub fn new(_: &Config) -> Self {
    Self::default()
  }
}

impl Plugin for FarmPluginLibrary {
  fn name(&self) -> &str {
    "FarmPluginLibrary"
  }
}
