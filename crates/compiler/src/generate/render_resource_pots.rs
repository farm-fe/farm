use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  parking_lot::Mutex,
  plugin::PluginHookContext,
  rayon::prelude::{IntoParallelIterator, ParallelIterator},
  resource::resource_pot::ResourcePotId,
};

pub fn render_resource_pots_and_generate_resources(
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<()> {
  let mut resource_pot_graph = context.resource_pot_graph.write();
  let resource_pots = resource_pot_graph.resource_pots_mut();
  let resources = Mutex::new(vec![]);

  println!(
    "resource pots len: {}, {:?}",
    resource_pots.len(),
    resource_pots
      .iter()
      .map(|r| r.id.clone())
      .collect::<Vec<ResourcePotId>>()
  );

  // Note: Plugins should not using context.resource_pot_graph, as it may cause deadlock
  resource_pots.into_par_iter().try_for_each(|resource_pot| {
    println!("render resource pot start");

    context
      .plugin_driver
      .render_resource_pot(resource_pot, context)?;
    println!("optimize resource pot start");

    context
      .plugin_driver
      .optimize_resource_pot(resource_pot, context)?;
    println!("generate resource pot start");
    let res = context
      .plugin_driver
      .generate_resources(resource_pot, context, hook_context)?;

    println!("set generated resources for {:?}", resource_pot.id);

    if let Some(res) = res {
      let mut resources = resources.lock();

      for r in &res {
        resource_pot.add_resource(r.name.clone());
      }

      resources.extend(res);
    } else {
      return Err(CompilationError::GenerateResourcesError {
        name: resource_pot.id.to_string(),
        ty: resource_pot.resource_pot_type.clone(),
        source: None,
      });
    }

    Ok(())
  })?;

  let mut resources_map = context.resources_map.lock();

  for resource in resources.lock().drain(..) {
    println!(
      "insert resource {:?} {:?}",
      resource.name, resource.resource_type
    );

    resources_map.insert(resource.name.clone(), resource);
  }

  Ok(())
}
