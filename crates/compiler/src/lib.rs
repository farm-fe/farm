#![deny(clippy::all)]
#![feature(box_patterns)]

use std::sync::Arc;

use farmfe_core::{
  config::Config, context::CompilationContext, error::Result, plugin::Plugin, stats::Stats,
};
use update::{UpdateOutput, UpdateType};

pub mod build;
pub mod generate;
pub mod update;

pub struct Compiler {
  context: Arc<CompilationContext>,
}

impl Compiler {
  /// The params are [farmfe_core::config::Config] and dynamic load rust plugins and js plugins [farmfe_core::plugin::Plugin]
  pub fn new(config: Config, mut plugin_adapters: Vec<Arc<dyn Plugin>>) -> Result<Self> {
    let mut plugins = vec![
      Arc::new(farmfe_plugin_runtime::FarmPluginRuntime::new(&config)) as _,
      // register internal core plugins
      Arc::new(farmfe_plugin_resolve::FarmPluginResolve::new(&config)) as _,
      Arc::new(farmfe_plugin_script::FarmPluginScript::new(&config)) as _,
      Arc::new(farmfe_plugin_merge_modules::FarmPluginMergeModules::new(
        &config,
      )) as _,
      Arc::new(farmfe_plugin_html::FarmPluginHtml::new(&config)) as _,
      Arc::new(farmfe_plugin_css::FarmPluginCss::new(&config)) as _,
    ];

    plugins.append(&mut plugin_adapters);
    // sort plugins by priority
    plugins.sort_by_key(|a| a.priority());

    Ok(Self {
      context: Arc::new(CompilationContext::new(config, plugins)?),
    })
  }

  /// Compile the project using the configuration
  pub fn compile(&self) -> Result<()> {
    // triggering build stage
    self.build()?;

    self.generate()?;

    self.context.plugin_driver.finish(&Stats {}, &self.context)
  }

  /// Recompile the project based on the changed files
  pub fn re_compile(&self, paths: Vec<(String, UpdateType)>) -> Result<UpdateOutput> {
    Ok(UpdateOutput::default())
  }

  pub fn context(&self) -> &Arc<CompilationContext> {
    &self.context
  }
}
