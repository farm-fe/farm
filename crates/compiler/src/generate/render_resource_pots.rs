use std::sync::Arc;

use farmfe_core::{
  config::LibraryBundleType,
  context::CompilationContext,
  error::{CompilationError, Result},
  parking_lot::Mutex,
  plugin::{GeneratedResource, PluginGenerateResourcesHookResult, PluginHookContext},
  rayon::prelude::{IntoParallelIterator, ParallelIterator},
  resource::{resource_pot::ResourcePot, Resource, ResourceType},
  HashMap,
};
use farmfe_toolkit::{
  fs::{transform_output_entry_filename, transform_output_filename, TransformOutputFileNameParams},
  sourcemap::append_sourcemap_comment,
};

use farmfe_plugin_library::FARM_BUNDLE_PLACEHOLDER_PREFIX;

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

  let is_library = context.config.output.target_env.is_library();
  // Collect module_id -> JS resource filename mapping for library mode
  // placeholder replacement (used to replace FARM_BUNDLE_PLACEHOLDER:: markers)
  let module_to_resource: Mutex<HashMap<String, String>> = Mutex::new(HashMap::default());

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

      // For cached resource pots, also collect module -> resource mapping
      if is_library {
        let js_name = cached_resources.resources.iter().find_map(|r| {
          if matches!(r.resource.resource_type, ResourceType::Js) {
            Some(r.resource.name.clone())
          } else {
            None
          }
        });
        if let Some(js_name) = js_name {
          let mut map = module_to_resource.lock();
          for module_id in resource_pot.modules() {
            map.insert(module_id.to_string(), js_name.clone());
          }
        }
      }

      for cached_resource in cached_resources.resources {
        resources.lock().push(cached_resource.resource);

        if let Some(map) = cached_resource.source_map {
          if !context.config.sourcemap.is_inline() {
            resource_pot.add_resource(map.name.clone());
            resources.lock().push(map);
          }
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

          // In bundle-less mode, don't auto-append name_hash to filenames.
          // Each module has its own resource pot with a unique name already, so hashes
          // are unnecessary unless the user explicitly configures [hash] or [contentHash]
          // in the output filename config.
          let is_bundle_less =
            context.config.output.library_bundle_type == LibraryBundleType::BundleLess;
          let name_hash = if is_bundle_less {
            ""
          } else {
            r.name_hash.as_str()
          };

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
                name_hash,
                ext: &r.resource_type.to_ext(),
                bytes: content_with_extra_content_hash,
                special_placeholders: &r.special_placeholders,
              },
            );
          } else {
            r.name = transform_output_filename(TransformOutputFileNameParams {
              filename_config: context.config.output.filename.clone(),
              name: &r.name,
              name_hash,
              ext: &r.resource_type.to_ext(),
              bytes: content_with_extra_content_hash,
              special_placeholders: &r.special_placeholders,
            });
          }
        }
      }

      // Collect module_id -> JS resource filename mapping for library mode
      if is_library {
        let js_name = generated_resources.resources.iter().find_map(|r| {
          if matches!(r.resource.resource_type, ResourceType::Js) {
            Some(r.resource.name.clone())
          } else {
            None
          }
        });
        if let Some(js_name) = js_name {
          let mut map = module_to_resource.lock();
          for module_id in resource_pot.modules() {
            map.insert(module_id.to_string(), js_name.clone());
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

          if !context.config.sourcemap.is_inline() {
            resource_pot.add_resource(source_map.name.clone());
            resources.lock().push(source_map);
          }
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

  // Replace bundle placeholders in library mode JS resources with actual relative paths
  if is_library {
    let module_to_resource = module_to_resource.into_inner();
    let mut resources_vec = resources.lock();
    for resource in resources_vec.iter_mut() {
      if matches!(resource.resource_type, ResourceType::Js) {
        replace_bundle_placeholders(resource, &module_to_resource);
      }
    }
  }

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
  let augment_resource_pot_hash = {
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

    context
      .plugin_driver
      .process_rendered_resource_pot(resource_pot, context)?;

    context
      .plugin_driver
      .augment_resource_pot_hash(resource_pot, context)?
  };

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
      augment_resource_pot_hash,
    ))
  }
}

/// Replace FARM_BUNDLE_PLACEHOLDER:: markers in JS resource bytes with actual
/// relative paths to the target resource files.
fn replace_bundle_placeholders(resource: &mut Resource, module_to_resource: &HashMap<String, String>) {
  let content = String::from_utf8_lossy(&resource.bytes);
  if !content.contains(FARM_BUNDLE_PLACEHOLDER_PREFIX) {
    return;
  }

  let mut new_content = content.to_string();
  for (module_id, target_resource_name) in module_to_resource {
    let placeholder = format!("{FARM_BUNDLE_PLACEHOLDER_PREFIX}{module_id}");
    if new_content.contains(&placeholder) {
      let relative = compute_relative_path(&resource.name, target_resource_name);
      new_content = new_content.replace(&placeholder, &relative);
    }
  }

  resource.bytes = new_content.into_bytes();
}

/// Compute a relative path from one resource to another.
/// Both paths are relative to the output directory (e.g., "index.js", "lib/utils.js").
/// Returns a path suitable for JS imports (e.g., "./lib/utils.js", "../other.js").
fn compute_relative_path(from_resource: &str, to_resource: &str) -> String {
  // Get the directory of the source resource
  let from_parts: Vec<&str> = from_resource.split('/').collect();
  let from_dir: Vec<&str> = if from_parts.len() > 1 {
    from_parts[..from_parts.len() - 1].to_vec()
  } else {
    vec![]
  };

  let to_parts: Vec<&str> = to_resource.split('/').collect();

  // Find common prefix length
  let common_len = from_dir
    .iter()
    .zip(to_parts.iter())
    .take_while(|(a, b)| a == b)
    .count();

  // Number of ".." needed
  let ups = from_dir.len() - common_len;
  let remaining = &to_parts[common_len..];

  let mut result = String::new();
  if ups == 0 {
    result.push_str("./");
  } else {
    for _ in 0..ups {
      result.push_str("../");
    }
  }
  result.push_str(&remaining.join("/"));

  result
}
