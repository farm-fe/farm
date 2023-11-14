use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  parking_lot::Mutex,
  plugin::{
    PluginGenerateResourcesHookResult, PluginHookContext, PluginRenderResourcePotHookParam,
    ResourcePotInfoOfPluginRenderResourcePotHookParam,
  },
  rayon::prelude::{IntoParallelIterator, ParallelIterator},
  resource::{resource_pot::ResourcePot, ResourceType},
};
use farmfe_toolkit::{
  common::append_source_map_comment,
  fs::{transform_output_entry_filename, transform_output_filename},
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

  let mut resource_pots_need_render = vec![];

  for resource_pot in resource_pots {
    let start = std::time::Instant::now();
    let cached_resource = try_get_resource_cache(resource_pot, context)?;
    println!("try_get_resource_cache time: {:?}", start.elapsed());

    if let Some((meta, cached_resource)) = cached_resource {
      println!("cached_resource: {:?}", cached_resource.resource.name);
      resource_pot.meta = meta;

      resource_pot.add_resource(cached_resource.resource.name.clone());
      resources.lock().push(cached_resource.resource);

      if let Some(map) = cached_resource.source_map {
        resource_pot.add_resource(map.name.clone());
        resources.lock().push(map);
      }
    } else {
      resource_pots_need_render.push(resource_pot);
    }
  }

  // Note: Plugins should not using context.resource_pot_map, as it may cause deadlock
  resource_pots_need_render
    .into_par_iter()
    .try_for_each(|resource_pot| {
      let start = std::time::Instant::now();
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

      let mut cached_result = PluginGenerateResourcesHookResult {
        resource: Default::default(),
        source_map: None,
      };
      // if source map is generated, we need to update the resource name and the content of the resource
      // to make sure the source map can be found.
      if let Some(mut source_map) = res.source_map {
        source_map.name = format!("{}.{}", r.name, source_map.resource_type.to_ext());
        append_source_map_comment(&mut res.resource, &source_map, &context.config.sourcemap);

        if context.config.persistent_cache.enabled() {
          cached_result.source_map = Some(source_map.clone());
        }

        resource_pot.add_resource(source_map.name.clone());
        resources.lock().push(source_map);
      }

      if context.config.persistent_cache.enabled() {
        cached_result.resource = res.resource.clone();
        set_resource_cache(resource_pot, &cached_result, context);
      }

      resource_pot.add_resource(res.resource.name.clone());
      resources.lock().push(res.resource);

      println!(
        "render_resource_pot_generate_resources {} time: {:?}",
        resource_pot.name,
        start.elapsed()
      );
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
    let meta = context
      .plugin_driver
      .render_resource_pot_modules(resource_pot, context, hook_context)?
      .ok_or(CompilationError::PluginHookResultCheckError {
        hook_name: format!("render_resource_pot_modules({:?})", resource_pot.id),
      })?;

    resource_pot.meta = meta;

    let mut param = PluginRenderResourcePotHookParam {
      content: resource_pot.meta.rendered_content.clone(),
      resource_pot_info: ResourcePotInfoOfPluginRenderResourcePotHookParam::new(
        resource_pot,
        context,
      ),
    };
    context
      .plugin_driver
      .render_resource_pot(&mut param, context)?;
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
