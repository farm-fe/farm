use std::collections::HashMap;

use farmfe_core::{error::Result, plugin::PluginHookContext};

use crate::{
  generate::{
    partial_bundling::partial_bundling,
    render_resource_pots::render_resource_pots_and_generate_resources,
    write_resources::write_resources,
  },
  Compiler,
};

pub(crate) mod partial_bundling;
pub(crate) mod render_resource_pots;
pub(crate) mod write_resources;

impl Compiler {
  /// the generate stage
  pub(crate) fn generate(&self) -> Result<()> {
    self.context.plugin_driver.generate_start(&self.context)?;

    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::new(),
    };

    self.optimize_module_graph()?;

    partial_bundling(&self.context, &hook_context)?;

    self.process_resource_pot_graph()?;

    self.render_and_generate_resources(&hook_context)?;

    write_resources(&self.context)?;

    self.context.plugin_driver.generate_end(&self.context)
  }

  fn optimize_module_graph(&self) -> Result<()> {
    println!("optimize module graph start");
    let mut module_graph = self.context.module_graph.write();

    self
      .context
      .plugin_driver
      .optimize_module_graph(&mut *module_graph, &self.context)
  }

  fn process_resource_pot_graph(&self) -> Result<()> {
    let mut resource_pot_graph = self.context.resource_pot_graph.write();
    println!("process resource pot start");
    self
      .context
      .plugin_driver
      .process_resource_pot_graph(&mut *resource_pot_graph, &self.context)
  }

  fn render_and_generate_resources(&self, hook_context: &PluginHookContext) -> Result<()> {
    let mut resource_pot_graph = self.context.resource_pot_graph.write();
    let resource_pots = resource_pot_graph.resource_pots_mut();
    render_resource_pots_and_generate_resources(resource_pots, &self.context, &hook_context)
  }
}
