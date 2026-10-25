use std::sync::Arc;

use farmfe_core::{
  cache::resource_cache::resource_memory_store::CachedResourcePot,
  context::CompilationContext,
  plugin::PluginGenerateResourcesHookResult,
  resource::{meta_data::ResourcePotMetaData, resource_pot::ResourcePot},
  swc_common::Globals,
};
use farmfe_toolkit::script::swc_try_with::resolve_module_mark;

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::{
    config::Config,
    module::{
      Module, ModuleMetaData, ModuleType,
      meta_data::script::{ModuleExportIdent, ModuleExportIdentType, statement::SwcId},
      module_graph::ModuleGraphEdgeDataItem,
    },
    plugin::ResolveKind,
    resource::resource_pot::ResourcePotType,
  };

  #[test]
  fn advertised_resource_without_payload_is_a_cache_miss() {
    use farmfe_core::{
      cache::{
        CacheContext,
        resource_cache::ResourceCacheManager,
        store::{CacheStoreKey, constant::CacheStoreFactory, memory::MemoryCacheFactory},
      },
      config::persistent_cache::PersistentCacheConfig,
    };

    let config = Config {
      persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
      ..Default::default()
    };
    let context = CompilationContext::new(config, vec![]).unwrap();
    let pot = ResourcePot::new("missing-resource", "test", ResourcePotType::Js);
    let hash = get_resource_cache_key(&pot, &Arc::new(context));
    let mut context = CompilationContext::new(
      Config {
        persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
        ..Default::default()
      },
      vec![],
    )
    .unwrap();
    let factory = MemoryCacheFactory::new();
    let store = factory.create_cache_store("resource");
    store
      .write_single_cache(CacheStoreKey::from((pot.id.clone(), hash.clone())), vec![])
      .unwrap();
    // The memory store advertises the manifest key but cannot remove a payload
    // by its resource name when the bytes are stored under a different hash.
    // This reproduces the resource consumer's missing-payload state without IO.
    context.cache_manager.resource_cache = ResourceCacheManager::new(Arc::new(CacheContext {
      store_factory: Box::new(factory),
      ..Default::default()
    }));
    context.config.persistent_cache = Box::new(PersistentCacheConfig::Bool(true));
    assert!(context.cache_manager.resource_cache.has_cache(&pot.id));
    assert!(
      !context
        .cache_manager
        .resource_cache
        .is_cache_changed(pot.id.clone(), hash)
    );
    assert!(
      try_get_resource_cache(&pot, &Arc::new(context))
        .unwrap()
        .is_none()
    );
  }

  #[test]
  fn importer_resource_key_tracks_dependency_export_mapping() {
    let context = Arc::new(CompilationContext::new(Config::default(), vec![]).unwrap());
    let mut importer = Module::new("orders.js".into());
    importer.content_hash = "unchanged-importer".into();
    let mut dependency = Module::new("domain.js".into());
    dependency.module_type = ModuleType::Js;
    *dependency.meta = ModuleMetaData::Script(Default::default());
    dependency.meta.as_script_mut().export_ident_map.insert(
      "a".into(),
      ModuleExportIdent::new(
        dependency.id.clone(),
        SwcId::new("transitionOrder".into(), 0),
        ModuleExportIdentType::Declaration,
      ),
    );
    {
      let mut graph = context.module_graph.write();
      graph.add_module(importer);
      graph.add_module(dependency);
      graph
        .add_edge_item(
          &"orders.js".into(),
          &"domain.js".into(),
          ModuleGraphEdgeDataItem {
            source: "./domain.js".into(),
            kind: ResolveKind::Import,
            order: 0,
          },
        )
        .unwrap();
    }
    let mut pot = ResourcePot::new("orders", "test", ResourcePotType::Js);
    pot.add_module("orders.js".into());
    let before = get_resource_cache_key(&pot, &context);
    {
      let mut graph = context.module_graph.write();
      let exports = &mut graph
        .module_mut(&"domain.js".into())
        .unwrap()
        .meta
        .as_script_mut()
        .export_ident_map;
      let ident = exports.remove("a").unwrap();
      exports.insert("b".into(), ident);
    }
    assert_ne!(before, get_resource_cache_key(&pot, &context));
  }
}

/// Cache key of resource consists of:
/// - modules id
/// - used_exports of modules
/// - resolved export contracts used by generated imports
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

    // Mangling can change imports in an unchanged module when a dependency's
    // exports change. Source hashes alone must not reuse the old generated AST.
    let mut contracts = module_graph
      .dependencies(module_id)
      .into_iter()
      .map(|(id, _)| id)
      .collect::<Vec<_>>();
    contracts.push(module.id.clone());
    contracts.sort_by_key(|id| id.to_string());
    contracts.dedup();
    for contract_id in contracts {
      if let Some(contract) = module_graph.module(&contract_id)
        && contract.module_type.is_script()
      {
        let exports = contract
          .meta
          .as_script()
          .get_export_idents()
          .into_iter()
          .map(|(name, ident)| {
            let internal = ident.as_internal();
            // Syntax contexts are recreated on cache load and are not identity.
            (
              name,
              internal.module_id.to_string(),
              internal.ident.sym.to_string(),
              format!("{:?}", internal.export_type),
            )
          })
          .collect::<Vec<_>>();
        code.push_str(&format!(
          "[export-contract:{:?}:{:?}]",
          contract_id.to_string(),
          exports
        ));
      }
    }

    // freeze_module can derive output from watched sources without changing
    // the module's load/transform hash (for example Tailwind CSS candidates).
    // Use the fingerprints already captured by the module cache, rather than
    // reading files again or changing the source hash used for module reuse.
    if context.cache_manager.module_cache.has_cache(module_id) {
      let cached = context.cache_manager.module_cache.get_cache_ref(module_id);
      code.push_str(&cached.watch_dependencies_hash());
    }

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
    let Some(mut cached_resource_pot) = context
      .cache_manager
      .resource_cache
      .get_cache(&resource_pot.id)
    else {
      // A manifest entry may outlive its shard after an interrupted write.
      // Render this pot again instead of aborting the compiler.
      return Ok(None);
    };

    if let ResourcePotMetaData::Js(js_resource_pot_meta_data) = &mut cached_resource_pot.meta {
      let merged_globals = Globals::new();
      let (unresolved_mark, top_level_mark) =
        resolve_module_mark(&mut js_resource_pot_meta_data.ast, false, &merged_globals);
      context
        .meta
        .set_resource_pot_globals(&resource_pot.id, merged_globals);
      js_resource_pot_meta_data.top_level_mark = top_level_mark.as_u32();
      js_resource_pot_meta_data.unresolved_mark = unresolved_mark.as_u32();
    }

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

#[cfg(test)]
mod watched_resource_cache_tests {
  use super::*;
  use farmfe_core::{
    cache::module_cache::{CachedModule, CachedWatchDependency},
    config::{
      Config,
      persistent_cache::{PersistentCacheConfig, PersistentCacheConfigObj},
    },
    module::Module,
    resource::resource_pot::ResourcePotType,
  };

  #[test]
  fn watched_content_and_membership_change_resource_cache_key() {
    let context = Arc::new(
      CompilationContext::new(
        Config {
          persistent_cache: Box::new(PersistentCacheConfig::Obj(PersistentCacheConfigObj {
            memory: true,
            ..Default::default()
          })),
          ..Default::default()
        },
        vec![],
      )
      .unwrap(),
    );
    let mut module = Module::new("style.css".into());
    module.content_hash = "unchanged-css-source".into();
    context.module_graph.write().add_module(module.clone());
    let mut pot = ResourcePot::new("style", "modules", ResourcePotType::Css);
    pot.add_module(module.id.clone());
    let set_watches = |watches: &[(&str, &str, u128)]| {
      context.cache_manager.module_cache.set_cache(
        module.id.clone(),
        CachedModule {
          module: module.clone(),
          dependencies: vec![],
          watch_dependencies: watches
            .iter()
            .map(|(id, hash, timestamp)| CachedWatchDependency {
              dependency: (*id).into(),
              hash: (*hash).into(),
              timestamp: *timestamp,
            })
            .collect(),
          is_expired: false,
        },
      );
    };
    set_watches(&[("App.js", "p4", 1), ("Other.js", "same", 1)]);
    let original = get_resource_cache_key(&pot, &context);
    set_watches(&[("App.js", "p8", 1), ("Other.js", "same", 1)]);
    let changed = get_resource_cache_key(&pot, &context);
    assert_ne!(
      original, changed,
      "freeze-derived CSS must not reuse an old resource"
    );
    set_watches(&[("Other.js", "same", 2), ("App.js", "p8", 2)]);
    assert_eq!(
      changed,
      get_resource_cache_key(&pot, &context),
      "watch order and timestamp-only edits must preserve the key"
    );
    set_watches(&[("Other.js", "same", 2)]);
    assert_ne!(
      changed,
      get_resource_cache_key(&pot, &context),
      "removed candidate sources must invalidate generated CSS"
    );
  }
}
