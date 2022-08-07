use std::{collections::HashMap, path::PathBuf};

use farmfe_core::{
  error::{CompilationError, Result},
  parking_lot::Mutex,
  plugin::PluginHookContext,
  rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    prelude::ParallelDrainRange,
  },
};

use crate::Compiler;

impl Compiler {
  pub(crate) fn generate(&self) -> Result<()> {
    self.context.plugin_driver.generate_start(&self.context)?;
    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::new(),
    };

    let mut module_graph = self.context.module_graph.write();
    println!("optimize module graph start");
    // =============== Optimize Module Graph Start ================
    self
      .context
      .plugin_driver
      .optimize_module_graph(&mut *module_graph, &self.context)?;
    // =============== Optimize Module Graph End ================

    println!("analyze module graph start");
    // =============== Analyze Module Graph Start ================
    let module_group_map = self
      .context
      .plugin_driver
      .analyze_module_graph(&mut *module_graph, &self.context, &hook_context)?
      .unwrap();
    // =============== Analyze Module Graph End ================
    println!("module group map len: {}", module_group_map.len());
    drop(module_graph);

    println!("merge modules start");
    // =============== Merge Modules Start ================
    let resource_pot_graph = self
      .context
      .plugin_driver
      .merge_modules(&module_group_map, &self.context, &hook_context)?
      .unwrap();
    // =============== Merge Modules End ================

    let mut g = self.context.resource_pot_graph.write();
    g.replace(resource_pot_graph);
    drop(g);

    // =============== Process Resource Pot Graph Start ================
    let mut resource_pot_graph = self.context.resource_pot_graph.write();
    println!("process resource pot start");
    self
      .context
      .plugin_driver
      .process_resource_pot_graph(&mut *resource_pot_graph, &self.context)?;
    // =============== Process Resource Pot Graph End ================

    let resource_pots = resource_pot_graph.resource_pots_mut();
    let resources = Mutex::new(vec![]);

    println!("resources len: {}", resource_pots.len());
    // Note: Plugins should not using context.resource_pot_graph, as it may cause deadlock
    resource_pots.into_par_iter().try_for_each(|resource_pot| {
      println!("render resource pot start");
      self
        .context
        .plugin_driver
        .render_resource_pot(resource_pot, &self.context)?;
      println!("optimize resource pot start");
      self
        .context
        .plugin_driver
        .optimize_resource_pot(resource_pot, &self.context)?;
      println!("generate resource pot start");
      let res = self.context.plugin_driver.generate_resources(
        resource_pot,
        &self.context,
        &hook_context,
      )?;

      if let Some(res) = res {
        let mut resources = resources.lock();
        resources.extend(res);
      } else {
        return Err(CompilationError::GenerateResourcesError {
          name: resource_pot.id.to_string(),
          ty: resource_pot.resource_pot_type.clone(),
        });
      }

      Ok(())
    })?;

    let mut resources = resources.lock();

    self
      .context
      .plugin_driver
      .write_resources(&mut *resources, &self.context)?;

    resources.par_drain(..).try_for_each(|resource| {
      if !resource.emitted {
        let root = PathBuf::from(self.context.config.root.as_str());
        let output_path = root
          .join("dist")
          .join(resource.name.split('/').last().unwrap().to_string() + ".js");

        std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
        std::fs::write(output_path, resource.bytes).unwrap();
      }

      Ok::<(), CompilationError>(())
    })?;

    self.context.plugin_driver.generate_end(&self.context)
  }
}
