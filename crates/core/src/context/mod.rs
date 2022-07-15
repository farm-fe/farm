use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
  config::Config,
  module::module_graph::ModuleGraph,
  plugin::{plugin_driver::PluginDriver, Plugin},
};

pub struct CompilationContext {
  pub config: Config,
  pub module_graph: RwLock<ModuleGraph>,
  pub plugin_driver: PluginDriver,
}

impl CompilationContext {
  pub fn new(config: Config, plugins: Vec<Arc<dyn Plugin + Sync + Send>>) -> Self {
    Self {
      module_graph: RwLock::new(ModuleGraph::new()),
      config,
      plugin_driver: PluginDriver::new(plugins),
    }
  }
}
