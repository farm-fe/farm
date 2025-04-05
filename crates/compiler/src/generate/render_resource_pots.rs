use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  parking_lot::Mutex,
  plugin::{GeneratedResource, PluginGenerateResourcesHookResult, PluginHookContext},
  rayon::prelude::{IntoParallelIterator, ParallelIterator},
  resource::resource_pot::ResourcePot,
  HashMap,
};
use farmfe_toolkit::{
  fs::{transform_output_entry_filename, transform_output_filename, TransformOutputFileNameParams},
  sourcemap::append_sourcemap_comment,
};

use crate::generate::resource_cache::{set_resource_cache, try_get_resource_cache};

pub fn render_resource_pots_and_generate_resources(
  resource_pots: Vec<&mut ResourcePot>,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<()> {
  #[cfg(feature = "profile")]
  farmfe_core::puffin::profile_function!();

  let resources = Mutex::new(vec![]);
  let entries = context.module_graph.read().entries.clone();
  let dynamic_entries = context.module_graph.read().dynamic_entries.clone();

  let mut resource_pots_need_render = vec![];

  for resource_pot in resource_pots {
    let cached_resource_pot = try_get_resource_cache(resource_pot, context)?;

    if let Some(cached_resource_pot) = cached_resource_pot {
      let cached_resources = cached_resource_pot.resources;
      let cached_meta = cached_resource_pot.meta;

      resource_pot.meta = cached_meta;

      for cached_resource in &cached_resources.resources {
        resource_pot.add_resource(cached_resource.resource.name.clone());
      }

      for cached_resource in cached_resources.resources {
        resources.lock().push(cached_resource.resource);

        if let Some(map) = cached_resource.source_map {
          resource_pot.add_resource(map.name.clone());

          resources.lock().push(map);
        }
      }
    } else {
      resource_pots_need_render.push(resource_pot);
    }
  }

  context
    .plugin_driver
    .render_start(&context.config, context)?;

  // Note: Plugins should not using context.resource_pot_map, as it may cause deadlock
  resource_pots_need_render
    .into_par_iter()
    .try_for_each(|resource_pot| {
      #[cfg(feature = "profile")]
      let id = farmfe_utils::transform_string_to_static_str(format!(
        "Render and generate resources for {:?}",
        resource_pot.id
      ));
      #[cfg(feature = "profile")]
      farmfe_core::puffin::profile_scope!(id);

      // let mut resource_pot_info: Option<ResourcePotInfo> = None;
      let (mut generated_resources, augment_resource_hash) =
        render_resource_pot_generate_resources(resource_pot, context, hook_context)?;

      let mut cached_result: PluginGenerateResourcesHookResult =
        PluginGenerateResourcesHookResult { resources: vec![] };
      let augment_resource_hash = augment_resource_hash.unwrap_or_default();
      let augment_resource_hash_bytes = augment_resource_hash.as_bytes();

      for res in &mut generated_resources.resources {
        let r = &mut res.resource;

        // ignore runtime resource
        if r.should_transform_output_filename {
          let content_with_extra_content_hash = &[&r.bytes, augment_resource_hash_bytes].concat();
          if let Some(name) = resource_pot.entry_module.as_ref() {
            let entry_name = entries
              .get(name)
              .or_else(|| dynamic_entries.get(name))
              .unwrap();

            r.name = transform_output_entry_filename(
              entry_name,
              TransformOutputFileNameParams {
                // use entry_filename first and fallback to filename if entry_filename is not set
                filename_config: if !context.config.output.entry_filename.is_empty() {
                  context.config.output.entry_filename.clone()
                } else {
                  context.config.output.filename.clone()
                },
                name: &r.name,
                name_hash: &r.name_hash,
                ext: &r.resource_type.to_ext(),
                bytes: content_with_extra_content_hash,
              },
            );
          } else {
            r.name = transform_output_filename(TransformOutputFileNameParams {
              filename_config: context.config.output.filename.clone(),
              name: &r.name,
              name_hash: &r.name_hash,
              ext: &r.resource_type.to_ext(),
              bytes: content_with_extra_content_hash,
            });
          }
        }
      }

      // process generated resources after rendering
      context
        .plugin_driver
        .process_generated_resources(&mut generated_resources, context)?;

      for mut res in generated_resources.resources {
        let mut cached_resource = GeneratedResource {
          resource: Default::default(),
          source_map: None,
        };
        // if source map is generated, we need to update the resource name and the content of the resource
        // to make sure the source map can be found.
        if let Some(mut source_map) = res.source_map {
          source_map.name = format!(
            "{}.{}",
            res.resource.name,
            source_map.resource_type.to_ext()
          );
          append_sourcemap_comment(&mut res.resource, &source_map, &context.config.sourcemap);

          if context.config.persistent_cache.enabled() {
            cached_resource.source_map = Some(source_map.clone());
          }

          resource_pot.add_resource(source_map.name.clone());

          resources.lock().push(source_map);
        }

        if context.config.persistent_cache.enabled() {
          cached_resource.resource = res.resource.clone();
          cached_result.resources.push(cached_resource);
        }

        resource_pot.add_resource(res.resource.name.clone());

        resources.lock().push(res.resource);
      }

      if !cached_result.resources.is_empty() {
        set_resource_cache(resource_pot, cached_result, context);
      }

      Ok::<(), CompilationError>(())
    })?;

  let mut resources_map: farmfe_core::parking_lot::lock_api::MutexGuard<
    '_,
    farmfe_core::parking_lot::RawMutex,
    HashMap<String, farmfe_core::resource::Resource>,
  > = context.resources_map.lock();

  for resource in resources.lock().drain(..) {
    resources_map.insert(resource.name.clone(), resource);
  }

  Ok(())
}

pub fn render_resource_pot_generate_resources(
  resource_pot: &mut ResourcePot,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
  // chunk_resource_info: &mut Option<ResourcePotInfo>,
) -> Result<(PluginGenerateResourcesHookResult, Option<String>)> {
  let mut augment_resource_hash = None;

  {
    #[cfg(feature = "profile")]
    let id = farmfe_utils::transform_string_to_static_str(format!(
      "Render resource pot {:?}",
      resource_pot.id
    ));
    #[cfg(feature = "profile")]
    farmfe_core::puffin::profile_scope!(id);

    let meta = context
      .plugin_driver
      .render_resource_pot(resource_pot, context, hook_context)?
      .ok_or(CompilationError::PluginHookResultCheckError {
        hook_name: format!(
          "render_resource_pot(name:{}, type:{:?})",
          resource_pot.id, resource_pot.resource_pot_type
        ),
      })?;

    resource_pot.meta = meta;

    augment_resource_hash = context
      .plugin_driver
      .augment_resource_hash(resource_pot, context)?;

    // TODO augment resource hash
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

    Ok((
      context
        .plugin_driver
        .generate_resources(resource_pot, context, hook_context)?
        .ok_or(CompilationError::GenerateResourcesError {
          name: resource_pot.id.to_string(),
          ty: resource_pot.resource_pot_type.clone(),
          source: None,
        })?,
      augment_resource_hash,
    ))
  }
}
