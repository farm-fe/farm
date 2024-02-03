use console::style;
use farmfe_core::{config::Config, context::CompilationContext, error::Result, plugin::Plugin};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::{Arc, RwLock};

pub struct FarmPluginProgress {
  module_count: Arc<RwLock<u32>>,
  progress_bar: ProgressBar,
}

impl FarmPluginProgress {
  pub fn new(_config: &Config) -> Self {
    let spinner_style =
      ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.green} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let progress_bar = ProgressBar::new(1);
    progress_bar.set_style(spinner_style.clone());
    progress_bar.set_prefix("[ building ]");

    Self {
      module_count: Arc::new(RwLock::new(0)),
      progress_bar,
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
      .progress_bar
      .set_message(format!("transform ({count}) {module}"));
    self.progress_bar.inc(1);
    Ok(None)
  }

  fn render_resource_pot(
    &self,
    param: &farmfe_core::plugin::PluginRenderResourcePotHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<farmfe_core::plugin::PluginRenderResourcePotHookResult>> {
    let name: &String = &param.resource_pot_info.name.clone();
    self.progress_bar.set_message(format!("render {name}"));
    self.progress_bar.inc(1);
    Ok(None)
  }

  fn generate_end(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.progress_bar.finish_and_clear();
    println!(
      "{} compiled {} modules successfully",
      style("✔").green(),
      self.get_module_count()
    );
    Ok(None)
  }
}
