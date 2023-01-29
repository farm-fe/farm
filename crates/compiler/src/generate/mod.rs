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

mod partial_bundling;
mod render_resource_pots;
mod write_resources;

impl Compiler {
  pub(crate) fn generate(&self) -> Result<()> {
    self.context.plugin_driver.generate_start(&self.context)?;
    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::new(),
    };

    /* =============== Optimize Module Graph Start ================ */
    println!("optimize module graph start");
    {
      let mut module_graph = self.context.module_graph.write();

      self
        .context
        .plugin_driver
        .optimize_module_graph(&mut *module_graph, &self.context)?;
    }
    /* =============== Optimize Module Graph End ================ */

    /* =============== Merge Modules Start ================ */
    partial_bundling(&self.context, &hook_context)?;
    /* =============== Merge Modules End ================ */

    /* =============== Process Resource Pot Graph Start ================ */
    {
      let mut resource_pot_graph = self.context.resource_pot_graph.write();
      println!("process resource pot start");
      self
        .context
        .plugin_driver
        .process_resource_pot_graph(&mut *resource_pot_graph, &self.context)?;
    }
    /* =============== Process Resource Pot Graph End ================ */

    render_resource_pots_and_generate_resources(&self.context, &hook_context)?;

    write_resources(&self.context)?;

    self.context.plugin_driver.generate_end(&self.context)
  }
}
