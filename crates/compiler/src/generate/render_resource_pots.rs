use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  parking_lot::Mutex,
  plugin::PluginHookContext,
  rayon::prelude::{IntoParallelIterator, ParallelIterator},
  resource::{resource_pot::ResourcePot, Resource},
};
use farmfe_toolkit::{fs::transform_output_filename, hash::sha256, tracing};

#[tracing::instrument(skip_all)]
pub fn render_resource_pots_and_generate_resources(
  resource_pots: Vec<&mut ResourcePot>,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<()> {
  tracing::trace!("Starting render_resource_pots_and_generate_resources");

  let resources = Mutex::new(vec![]);

  // Note: Plugins should not using context.resource_pot_map, as it may cause deadlock
  resource_pots.into_par_iter().try_for_each(|resource_pot| {
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

  tracing::trace!("render_resource_pots_and_generate_resources finished");
  Ok(())
}

#[tracing::instrument(skip_all)]
pub fn render_resource_pot_generate_resources(
  resource_pot: &mut ResourcePot,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
  skip_render: bool,
) -> Result<Vec<Resource>> {
  tracing::trace!("Starting render_resource_pot_generate_resources");

  if !skip_render {
    tracing::trace!("Starting render_resource_pot_generate_resources: render_resource_pot");

    context
      .plugin_driver
      .render_resource_pot(resource_pot, context)?;

    tracing::trace!("render_resource_pot_generate_resources: optimize_resource_pot finished");
  }

  tracing::trace!("Starting render_resource_pot_generate_resources: optimize_resource_pot");
  context
    .plugin_driver
    .optimize_resource_pot(resource_pot, context)?;
  tracing::trace!("render_resource_pot_generate_resources: optimize_resource_pot finished");

  tracing::trace!("Starting render_resource_pot_generate_resources: generate_resources");
  let res = context
    .plugin_driver
    .generate_resources(resource_pot, context, hook_context)?
    .ok_or(CompilationError::GenerateResourcesError {
      name: resource_pot.id.to_string(),
      ty: resource_pot.resource_pot_type.clone(),
      source: None,
    });
  tracing::trace!("render_resource_pot_generate_resources: generate_resources finished");

  tracing::trace!("render_resource_pot_generate_resources finished");
  res
}
