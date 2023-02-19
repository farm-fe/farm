#![deny(clippy::all)]
#![feature(box_patterns)]

use std::sync::Arc;

use farmfe_core::{
  config::Config, context::CompilationContext, error::Result, plugin::Plugin, stats::Stats,
};
use update::{UpdateResult, UpdateType};

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
      Arc::new(farmfe_plugin_partial_bundling::FarmPluginPartialBundling::new(&config)) as _,
      Arc::new(farmfe_plugin_html::FarmPluginHtml::new(&config)) as _,
      Arc::new(farmfe_plugin_css::FarmPluginCss::new(&config)) as _,
      Arc::new(farmfe_plugin_react::FarmPluginReact::new(&config)) as _,
    ];

    plugins.append(&mut plugin_adapters);
    // sort plugins by priority
    plugins.sort_by_key(|a| a.priority());

    let mut context = CompilationContext::new(config, plugins)?;
    context.plugin_driver.config(&mut context.config)?;
    Ok(Self {
      context: Arc::new(context),
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
  pub fn re_compile(&self, paths: Vec<(String, UpdateType)>) -> Result<UpdateResult> {
    // self.context.plugin_driver.update_start(&self.context)?;
    self.update(paths)
    // self.context.plugin_driver.update_end(&self.context)?;
  }

  pub fn context(&self) -> &Arc<CompilationContext> {
    &self.context
  }
}
