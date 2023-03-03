use std::collections::HashMap;

use farmfe_core::{error::Result, plugin::PluginHookContext};
use farmfe_toolkit::tracing;

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

impl Compiler {
  /// the generate stage
  #[tracing::instrument(skip_all)]
  pub(crate) fn generate(&self) -> Result<()> {
    tracing::trace!("Starting generating...");
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

    tracing::trace!("Generating finished.");
    self.context.plugin_driver.generate_end(&self.context)
  }

  #[tracing::instrument(skip_all)]
  fn optimize_module_graph(&self) -> Result<()> {
    tracing::trace!("Optimizing module graph...");
    let mut module_graph = self.context.module_graph.write();

    self
      .context
      .plugin_driver
      .optimize_module_graph(&mut *module_graph, &self.context)?;

    tracing::trace!("Optimized module graph.");
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  fn process_resource_pot_map(&self) -> Result<()> {
    tracing::trace!("Processing resource pot graph...");
    let mut resource_pot_map = self.context.resource_pot_map.write();

    self
      .context
      .plugin_driver
      .process_resource_pot_map(&mut *resource_pot_map, &self.context)?;

    tracing::trace!("Processed resource pot graph.");

    Ok(())
  }

  #[tracing::instrument(skip_all)]
  fn render_and_generate_resources(&self, hook_context: &PluginHookContext) -> Result<()> {
    tracing::trace!("Rendering and generating resources...");

    let mut resource_pot_map = self.context.resource_pot_map.write();
    let resource_pots = resource_pot_map.resource_pots_mut();
    render_resource_pots_and_generate_resources(resource_pots, &self.context, hook_context)?;

    tracing::trace!("Rendered and generated resources.");

    Ok(())
  }
}
