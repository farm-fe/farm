#![deny(clippy::all)]

use std::sync::Arc;

use farmfe_core::{
  config::Config, context::CompilationContext, error::Result, plugin::Plugin, stats::Stats,
};
use update::UpdateOutput;

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
      // register internal core plugins
      Arc::new(farmfe_plugin_resolve::FarmPluginResolve::new(&config)) as _,
      Arc::new(farmfe_plugin_script::FarmPluginScript::new(&config)) as _,
    ];

    plugins.append(&mut plugin_adapters);
    // sort plugins by priority
    plugins.sort_by_key(|a| a.priority());

    Ok(Self {
      context: Arc::new(CompilationContext::new(config, plugins)?),
    })
  }

  pub fn compile(&self) -> Result<()> {
    // triggering build stage
    self.build()?;

    let module_graph = self.context.module_graph.read();
    let modules = module_graph.modules();
    println!("module graph nodes length: {}", modules.len());

    for m in modules {
      if m.id.path().contains("index.ts") {
        let dep = module_graph.get_dep_by_source(&m.id, "./src/main");
        println!("dep: {:?}", dep);
      }
    }
    self.generate()?;

    self.context.plugin_driver.finish(&Stats {}, &self.context)
  }

  pub fn update(&self, paths: Vec<String>) -> Result<UpdateOutput> {
    Ok(UpdateOutput {})
  }
}
