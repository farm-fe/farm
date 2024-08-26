use std::collections::HashMap;

use farmfe_core::{error::Result, plugin::PluginHookContext};

use crate::{
  generate::{
    finalize_resources::finalize_resources, partial_bundling::partial_bundling,
    render_resource_pots::render_resource_pots_and_generate_resources,
  },
  Compiler,
};

pub(crate) mod finalize_resources;
pub(crate) mod partial_bundling;
pub(crate) mod render_resource_pots;
pub(crate) mod resource_cache;

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

    self.process_resource_pot_map()?;

    self.render_and_generate_resources(&hook_context)?;

    finalize_resources(&self.context)?;

    self.context.plugin_driver.generate_end(&self.context)
  }

  fn optimize_module_graph(&self) -> Result<()> {
    #[cfg(feature = "profile")]
    farmfe_core::puffin::profile_function!();

    let mut module_graph = self.context.module_graph.write();

    self
      .context
      .plugin_driver
      .optimize_module_graph(&mut module_graph, &self.context)?;

    if self.context.config.record {
      self
        .context
        .record_manager
        .set_optimized_module_graph_stats(&module_graph);
    }

    Ok(())
  }

  fn process_resource_pot_map(&self) -> Result<()> {
    let mut resource_pot_map = self.context.resource_pot_map.write();

    self
      .context
      .plugin_driver
      .process_resource_pots(&mut resource_pot_map.resource_pots_mut(), &self.context)?;

    Ok(())
  }

  fn render_and_generate_resources(&self, hook_context: &PluginHookContext) -> Result<()> {
    let mut resource_pot_map = self.context.resource_pot_map.write();
    let resource_pots = resource_pot_map.resource_pots_mut();
    render_resource_pots_and_generate_resources(resource_pots, &self.context, hook_context)?;

    Ok(())
  }
}
