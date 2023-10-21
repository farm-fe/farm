use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  resource::{
    resource_pot::{ResourcePot, ResourcePotMetaData},
    Resource,
  },
};

use crate::build::module_cache::is_cache_enabled;

/// Cache key of resource is consist of:
/// - modules id
/// - used_exports of modules
pub fn get_resource_cache_key(
  resource_pot: &ResourcePot,
  context: &Arc<CompilationContext>,
) -> String {
  let module_graph = context.module_graph.read();
  let mut code = String::new();

  for module_id in &resource_pot.modules() {
    let module = module_graph.module(module_id).unwrap();
    code.push_str(&module.id.to_string());
    // make sure cache is correct when tree shaking is enabled
    code.push_str(&module.used_exports.join(","));
  }

  farmfe_toolkit::hash::sha256(&code.as_bytes(), 32)
}

pub fn try_get_resource_cache(
  resource_pot: &ResourcePot,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<Option<(ResourcePotMetaData, Resource)>> {
  if !is_cache_enabled(resource_pot.immutable, context) {
    return Ok(None);
  }

  // Resource is cached only:
  // - cache key is cached
  // - all modules are cached initially
  let cache_key = get_resource_cache_key(resource_pot, context);

  if !context
    .cache_manager
    .resource_cache
    .has_resource_cache(&cache_key)
    || !context
      .cache_manager
      .resource_cache
      .has_resource_pot_meta_cache(&cache_key)
  {
    return Ok(None);
  }

  for module_id in resource_pot.modules() {
    if !context
      .cache_manager
      .module_cache
      .is_initial_cache(module_id)
    {
      return Ok(None);
    }
  }

  let cached_resource = context
    .cache_manager
    .resource_cache
    .get_resource_cache(&cache_key);
  let cached_resource_pot_meta = context
    .cache_manager
    .resource_cache
    .get_resource_pot_meta_cache(&cache_key);

  Ok(Some((cached_resource_pot_meta, cached_resource)))
}

pub fn set_resource_cache(
  resource_pot: &ResourcePot,
  resource: &Resource,
  context: &Arc<CompilationContext>,
) {
  let cache_key = get_resource_cache_key(resource_pot, context);
  context
    .cache_manager
    .resource_cache
    .set_resource_cache(&cache_key, resource);
  context
    .cache_manager
    .resource_cache
    .set_resource_pot_meta_cache(&cache_key, &resource_pot.meta);
}
