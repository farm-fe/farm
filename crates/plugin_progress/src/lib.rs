use std::sync::{Arc, RwLock};

use farmfe_core::{config::Config, context::CompilationContext, error::Result, plugin::Plugin};
use indicatif::{ProgressBar, ProgressStyle};

pub struct FarmPluginProgress {
  module_count: Arc<RwLock<u32>>,
  transform_progress: ProgressBar,
  render_resource_pot_progress: ProgressBar,
}

impl FarmPluginProgress {
  pub fn new(_config: &Config) -> Self {
    let spinner_style =
      ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.green} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let transform_progress = ProgressBar::new(1);
    transform_progress.set_style(spinner_style.clone());
    transform_progress.set_prefix("[ transform ]");
    let render_resource_pot_progress = ProgressBar::new(1);
    Self {
      module_count: Arc::new(RwLock::new(0)),
      transform_progress,
      render_resource_pot_progress,
    }
  }

  pub fn increment_module_count(&self) {
    if let Ok(mut count) = self.module_count.write() {
      *count += 1;
    }
  }

  pub fn reset_module_count(&self) {
    if let Ok(mut count) = self.module_count.write() {
      *count = 0;
    }
  }

  pub fn get_module_count(&self) -> u32 {
    if let Ok(count) = self.module_count.read() {
      *count
    } else {
      0
    }
  }
}

impl Plugin for FarmPluginProgress {
  fn name(&self) -> &'static str {
    "FarmPluginProgress"
  }

  fn build_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.reset_module_count();
    Ok(None)
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    self.increment_module_count();
    let count = self.get_module_count();
    let module = &param.module_id;
    self
      .transform_progress
      .set_message(format!("({count}) {module}"));
    Ok(None)
  }

  fn build_end(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.transform_progress.finish_with_message("waiting...");
    Ok(None)
  }

  fn generate_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.transform_progress.finish_and_clear();
    Ok(None)
  }

  fn process_resource_pots(
    &self,
    resource_pots: &mut Vec<&mut farmfe_core::resource::resource_pot::ResourcePot>,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    let n = resource_pots.len();
    self
      .render_resource_pot_progress
      .set_length(n.try_into().unwrap());
    Ok(None)
  }

  fn render_resource_pot(
    &self,
    _resource_pot: &farmfe_core::plugin::PluginRenderResourcePotHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<farmfe_core::plugin::PluginRenderResourcePotHookResult>> {
    self.render_resource_pot_progress.inc(1);
    Ok(None)
  }

  fn generate_end(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.render_resource_pot_progress.finish_and_clear();
    Ok(None)
  }
}
