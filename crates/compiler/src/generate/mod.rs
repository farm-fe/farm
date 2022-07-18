use farmfe_core::{
  error::Result,
  rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
  },
  resource::Resource,
};

use crate::Compiler;

impl Compiler {
  pub(crate) fn generate(&self) -> Result<()> {
    self.context.plugin_driver.generate_start(&self.context)?;

    self
      .context
      .plugin_driver
      .optimize_module_graph(&self.context.module_graph, &self.context)?;

    let module_group_map = self
      .context
      .plugin_driver
      .analyze_module_graph(&self.context.module_graph, &self.context)?
      .unwrap();

    let mut resource_graph = self
      .context
      .plugin_driver
      .merge_modules(&module_group_map, &self.context)?
      .unwrap();

    self
      .context
      .plugin_driver
      .process_resource_graph(&self.context.resource_graph, &self.context)?;

    let resources = resource_graph.resources_mut();

    resources.into_par_iter().try_for_each(|resource| {
      self
        .context
        .plugin_driver
        .render_resource(resource, &self.context)?;
      self
        .context
        .plugin_driver
        .optimize_resource(resource, &self.context)?;
      self
        .context
        .plugin_driver
        .generate_resource(resource, &self.context)?;
      self
        .context
        .plugin_driver
        .write_resource(resource, &self.context)
    })?;

    self.context.plugin_driver.generate_end(&self.context)
  }
}
