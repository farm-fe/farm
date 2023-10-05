use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  hashbrown::HashMap,
  parking_lot::Mutex,
  plugin::{PluginGenerateResourcesHookResult, PluginHookContext},
  rayon::prelude::{IntoParallelIterator, ParallelIterator},
  resource::{resource_pot::ResourcePot, Resource, ResourceType},
};
use farmfe_toolkit::fs::{transform_output_entry_filename, transform_output_filename};

pub fn render_resource_pots_and_generate_resources(
  resource_pots: Vec<&mut ResourcePot>,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<()> {
  #[cfg(feature = "profile")]
  farmfe_core::puffin::profile_function!();

  let resources = Mutex::new(vec![]);
  let entries = context.module_graph.read().entries.clone();

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
    let r = &mut res.resource;
    // ignore runtime resource
    if !matches!(r.resource_type, ResourceType::Runtime) {
      if let Some(name) = resource_pot.entry_module.as_ref() {
        let entry_name = entries.get(name).unwrap();
        r.name = transform_output_entry_filename(
          context.config.output.entry_filename.clone(),
          resource_pot.id.to_string().as_str(),
          entry_name,
          &r.bytes,
          &r.resource_type.to_ext(),
        );
      } else {
        r.name = transform_output_filename(
          context.config.output.filename.clone(),
          &r.name,
          &r.bytes,
          &r.resource_type.to_ext(),
        );
      }
    }

    // if source map is generated, we need to update the resource name and the content of the resource
    // to make sure the source map can be found.
    if let Some(mut source_map) = res.source_map {
      source_map.name = format!("{}.{}", r.name, source_map.resource_type.to_ext());
      let source_mapping_url = format!("\n//# sourceMappingURL=/{}", source_map.name);
      r.bytes.append(&mut source_mapping_url.as_bytes().to_vec());

      resource_pot.add_resource(source_map.name.clone());
      resources.lock().push(source_map);
    }

    resource_pot.add_resource(res.resource.name.clone());
    resources.lock().push(res.resource);

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
) -> Result<PluginGenerateResourcesHookResult> {
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

  {
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
  }
}
