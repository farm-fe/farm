use farmfe_core::{
  error::{CompilationError, Result},
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

    let resource_pot_graph = self
      .context
      .plugin_driver
      .merge_modules(&module_group_map, &self.context)?
      .unwrap();

    let mut g = self.context.resource_pot_graph.write();
    g.replace(resource_pot_graph);
    drop(g);

    self
      .context
      .plugin_driver
      .process_resource_pot_graph(&self.context.resource_pot_graph, &self.context)?;

    let mut g = self.context.resource_pot_graph.write();
    let resources = g.resources_mut();

    // Note: Plugins should not using context.resource_pot_graph, as it may cause deadlock
    resources.into_par_iter().try_for_each(|resource| {
      self
        .context
        .plugin_driver
        .render_resource_pot(resource, &self.context)?;
      self
        .context
        .plugin_driver
        .optimize_resource_pot(resource, &self.context)?;
      let resources = self
        .context
        .plugin_driver
        .generate_resources(resource, &self.context)?;

      if let Some(resources) = resources {
        resources.into_par_iter().try_for_each(|resource| {
          self
            .context
            .plugin_driver
            .write_resource(&resource, &self.context)?;

          Ok(())
        })?;
      } else {
        return Err(CompilationError::GenerateResourcesError {
          name: resource.name.clone(),
          ty: resource.resource_pot_type.clone(),
        });
      }

      Ok(())
    })?;

    self.context.plugin_driver.generate_end(&self.context)
  }
}
