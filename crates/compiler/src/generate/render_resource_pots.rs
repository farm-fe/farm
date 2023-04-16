use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  parking_lot::Mutex,
  plugin::PluginHookContext,
  rayon::prelude::{IntoParallelIterator, ParallelIterator},
  resource::{resource_pot::ResourcePot, Resource},
};
use farmfe_toolkit::fs::transform_output_filename;

pub fn render_resource_pots_and_generate_resources(
  resource_pots: Vec<&mut ResourcePot>,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<()> {
  #[cfg(feature = "profile")]
  farmfe_core::puffin::profile_function!();

  let resources = Mutex::new(vec![]);

  // Note: Plugins should not using context.resource_pot_map, as it may cause deadlock
  resource_pots.into_par_iter().try_for_each(|resource_pot| {
    #[cfg(feature = "profile")]
    let id = farmfe_utils::transform_string_to_static_str(format!(
      "Render and generate resources for {:?}",
      resource_pot.id
    ));
    #[cfg(feature = "profile")]
    farmfe_core::puffin::profile_scope!(id);

    let mut res =
      render_resource_pot_generate_resources(resource_pot, context, hook_context, false)?;

    for r in &mut res {
      let mut filename_config = context.config.output.filename.clone();

      if !r.preserve_name {
        filename_config = transform_output_filename(
          filename_config,
          &r.name,
          &r.bytes,
          &r.resource_type.to_ext(),
        );
      } else {
        filename_config = r.name.clone();
      }

      r.name = filename_config;
    }

    let mut resources = resources.lock();

    for r in &res {
      resource_pot.add_resource(r.name.clone());
    }

    resources.extend(res);

    Ok(())
  })?;

  let mut resources_map = context.resources_map.lock();

  for resource in resources.lock().drain(..) {
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
    #[cfg(feature = "profile")]
    let id = farmfe_utils::transform_string_to_static_str(format!(
      "Render resource pot {:?}",
      resource_pot.id
    ));
    #[cfg(feature = "profile")]
    farmfe_core::puffin::profile_scope!(id);

    context
      .plugin_driver
      .render_resource_pot(resource_pot, context)?;
  }

  {
    #[cfg(feature = "profile")]
    let id = farmfe_utils::transform_string_to_static_str(format!(
      "Optimize resource pot {:?}",
      resource_pot.id
    ));
    #[cfg(feature = "profile")]
    farmfe_core::puffin::profile_scope!(id);

    context
      .plugin_driver
      .optimize_resource_pot(resource_pot, context)?;
  }

  let res = {
    #[cfg(feature = "profile")]
    let id = farmfe_utils::transform_string_to_static_str(format!(
      "Generate resources for {:?}",
      resource_pot.id
    ));
    #[cfg(feature = "profile")]
    farmfe_core::puffin::profile_scope!(id);

    context
      .plugin_driver
      .generate_resources(resource_pot, context, hook_context)?
      .ok_or(CompilationError::GenerateResourcesError {
        name: resource_pot.id.to_string(),
        ty: resource_pot.resource_pot_type.clone(),
        source: None,
      })
  };

  res
}
