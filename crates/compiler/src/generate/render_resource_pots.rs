use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  parking_lot::Mutex,
  plugin::PluginHookContext,
  rayon::prelude::{IntoParallelIterator, ParallelIterator},
  resource::{
    resource_pot::{ResourcePot, ResourcePotId},
    Resource,
  },
};

pub fn render_resource_pots_and_generate_resources(
  resource_pots: Vec<&mut ResourcePot>,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<()> {
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
    let res = render_resource_pot_generate_resources(resource_pot, context, hook_context, false)?;

    println!("set generated resources for {:?}", resource_pot.id);

    let mut resources = resources.lock();

    for r in &res {
      resource_pot.add_resource(r.name.clone());
    }

    resources.extend(res);

    Ok(())
  })?;

  let mut resources_map = context.resources_map.lock();
  // resources_map.clear();

  for resource in resources.lock().drain(..) {
    println!(
      "insert resource {:?} {:?}",
      resource.name, resource.resource_type
    );

    resources_map.insert(resource.name.clone(), resource);
  }

  Ok(())
}

pub fn render_resource_pot_generate_resources(
  resource_pot: &mut ResourcePot,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
  skip_render: bool,
) -> Result<Vec<Resource>> {
  if !skip_render {
    println!("render resource pot start");
    context
      .plugin_driver
      .render_resource_pot(resource_pot, context)?;
  }

  println!("optimize resource pot start");
  context
    .plugin_driver
    .optimize_resource_pot(resource_pot, context)?;
  println!("generate resource pot start");
  context
    .plugin_driver
    .generate_resources(resource_pot, context, hook_context)?
    .ok_or(CompilationError::GenerateResourcesError {
      name: resource_pot.id.to_string(),
      ty: resource_pot.resource_pot_type.clone(),
      source: None,
    })
}
