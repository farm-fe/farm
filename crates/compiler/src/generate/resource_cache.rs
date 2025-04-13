use std::sync::Arc;

use farmfe_core::{
  cache::resource_cache::resource_memory_store::CachedResourcePot, context::CompilationContext,
  plugin::PluginGenerateResourcesHookResult, resource::resource_pot::ResourcePot,
};

/// Cache key of resource is consist of:
/// - modules id
/// - used_exports of modules
pub fn get_resource_cache_key(
  resource_pot: &ResourcePot,
  context: &Arc<CompilationContext>,
) -> String {
  let module_graph = context.module_graph.read();
  let mut code = resource_pot.id.to_string();

  for module_id in &resource_pot.modules() {
    let module = module_graph
      .module(module_id)
      .unwrap_or_else(|| panic!("module not found: {:?}", module_id.to_string()));

    // make sure cache is correct when tree shaking is enabled
    code.push_str(&module.content_hash);

    if context.cache_manager.module_cache.cache_outdated(module_id) {
      code.push_str(&format!("[cache_outdated+{}]", module.id.to_string()));
    }

    // if tree shaking is not enabled, we don't need to cache used_exports
    if context.config.tree_shaking.enabled() {
      code.push_str(&module.used_exports.join(","));
    }
  }

  farmfe_toolkit::hash::sha256(code.as_bytes(), 32)
}

pub fn try_get_resource_cache(
  resource_pot: &ResourcePot,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<Option<CachedResourcePot>> {
  if !context.config.persistent_cache.enabled()
    || !context
      .cache_manager
      .resource_cache
      .has_cache(&resource_pot.id)
  {
    // println!("cache not found : {:?}", resource_pot.id);
    return Ok(None);
  }

  let hash = get_resource_cache_key(resource_pot, context);

  if !context
    .cache_manager
    .resource_cache
    .is_cache_changed(resource_pot.id.clone(), hash)
  {
    let cached_resource_pot = context
      .cache_manager
      .resource_cache
      .get_cache(&resource_pot.id)
      .unwrap();
    return Ok(Some(cached_resource_pot));
  } else {
    // println!(
    //   "cache not found : {:?} hash: {:?}, cause resource cache changed",
    //   resource_pot.id, hash
    // );
  }

  Ok(None)
}

pub fn set_resource_cache(
  resource_pot: &ResourcePot,
  resource: PluginGenerateResourcesHookResult,
  context: &Arc<CompilationContext>,
) {
  let cache_key = get_resource_cache_key(resource_pot, context);

  context.cache_manager.resource_cache.set_cache(
    &resource_pot.id,
    CachedResourcePot {
      resources: resource,
      meta: resource_pot.meta.clone(),
      hash: cache_key,
    },
  );
}
