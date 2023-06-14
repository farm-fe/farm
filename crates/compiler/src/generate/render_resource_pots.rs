use std::{path::Path, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  hashbrown::HashMap,
  parking_lot::Mutex,
  plugin::PluginHookContext,
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

    let mut res_map = HashMap::new();

    for r in &res {
      // deal source map resource
      if let ResourceType::SourceMap(original_resource_name) = &r.resource_type {
        if let Some(orig) = res.iter().find(|item| &item.name == original_resource_name) {
          res_map.insert(orig.name.to_string(), r.name.to_string());
        }
      }
    }

    // deal with non-sourcemap resources
    for r in &mut res {
      if !matches!(r.resource_type, ResourceType::SourceMap(_)) {
        let name_before_update = r.name.clone();

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

        if res_map.contains_key(&name_before_update) {
          let source_mapping_url = format!(
            "\n//# sourceMappingURL={}.{}",
            r.name,
            ResourceType::SourceMap("".to_string()).to_ext()
          );
          r.bytes.append(&mut source_mapping_url.as_bytes().to_vec());

          let v = res_map.remove(&name_before_update).unwrap();
          // reverse the map to speed up the lookup
          res_map.insert(v, r.name.to_string());
        }
      }
    }

    // replace sourcemap resource
    for r in &mut res {
      // deal source map resource
      if let ResourceType::SourceMap(_) = &r.resource_type {
        if let Some(orig_name) = res_map.get(&r.name) {
          r.name = format!("{}.{}", orig_name, r.resource_type.to_ext());
        }
      }
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
