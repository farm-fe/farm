use std::sync::Arc;

use farmfe_core::{HashSet, context::CompilationContext, module::ModuleId};

#[cfg(test)]
use crate::build::module_cache::set_module_graph_cache;
use crate::build::module_cache::set_module_graph_cache_with_forced_updates;

use super::diff_and_patch_module_graph::DiffResult;

pub fn get_affected_module_ids(
  updated_modules: &Vec<ModuleId>,
  diff_result: &DiffResult,
  context: &Arc<CompilationContext>,
) -> Vec<ModuleId> {
  let mut affected_module_ids = vec![];

  for module_id in updated_modules {
    affected_module_ids.push(module_id.clone());
  }

  for added_id in &diff_result.added_modules {
    affected_module_ids.push(added_id.clone());
  }

  for removed_id in &diff_result.removed_modules {
    context
      .cache_manager
      .module_cache
      .invalidate_cache(removed_id);
  }

  affected_module_ids
}

pub fn set_updated_modules_cache(
  updated_modules: &Vec<ModuleId>,
  diff_result: &DiffResult,
  context: &Arc<CompilationContext>,
) {
  let affected_module_ids = get_affected_module_ids(updated_modules, diff_result, context);
  let updated_roots = updated_modules.iter().cloned().collect::<HashSet<_>>();
  // Graph patching retains the old pot associations until partial bundling.
  let old_pots = {
    let graph = context.module_graph.read();
    updated_modules
      .iter()
      .filter_map(|id| graph.module(id))
      .flat_map(|module| module.resource_pots.iter().cloned())
      .collect::<HashSet<_>>()
  };
  // Removing a persistent entry may restore a shard. Do that without graph locks.
  for pot in old_pots {
    context.cache_manager.resource_cache.invalidate_cache(&pot);
  }
  // set_cache replaces the in-memory module and removes its lower-store entry,
  // so the updated metadata is written even when the source key is unchanged.
  set_module_graph_cache_with_forced_updates(affected_module_ids, &updated_roots, context);
}

#[cfg(test)]
mod same_source_generated_cache_tests {
  use super::*;
  use farmfe_core::{
    cache::resource_cache::resource_memory_store::CachedResourcePot,
    cache::{CacheContext, module_cache::ModuleCacheManager},
    config::{Config, persistent_cache::PersistentCacheConfig},
    module::{Module, ModuleMetaData, ModuleType, meta_data::css::CssModuleMetaData},
    plugin::PluginGenerateResourcesHookResult,
    resource::meta_data::ResourcePotMetaData,
    rkyv,
  };
  use farmfe_toolkit::css::{codegen_css_stylesheet, parse_css_stylesheet};

  fn generated_css(generation: &str) -> Box<ModuleMetaData> {
    let parsed = parse_css_stylesheet(
      "fixture.css",
      Arc::new(format!(".{generation}{{color:red}}")),
    )
    .unwrap();
    Box::new(ModuleMetaData::Css(Box::new(CssModuleMetaData {
      ast: parsed.ast,
      comments: Default::default(),
      custom: Default::default(),
    })))
  }

  fn css_content(module: &Module) -> String {
    codegen_css_stylesheet(&module.meta.as_css().ast, false, None, false).0
  }

  fn fixture() -> Arc<CompilationContext> {
    let mut config = Config::default();
    let mut cache = PersistentCacheConfig::get_default_config(&config.root).as_obj(&config.root);
    cache.memory = true;
    config.persistent_cache = Box::new(PersistentCacheConfig::Obj(cache));
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    for (id, pot) in [
      ("styles.css", "styles-pot"),
      ("unrelated.css", "unrelated-pot"),
    ] {
      let mut module = Module::new(id.into());
      module.module_type = ModuleType::Css;
      module.content_hash = "unchanged-source".into();
      module.resource_pots.insert(pot.into());
      module.meta = generated_css("old");
      context.module_graph.write().add_module(module);
      context.cache_manager.resource_cache.set_cache(
        pot,
        CachedResourcePot {
          resources: PluginGenerateResourcesHookResult { resources: vec![] },
          meta: ResourcePotMetaData::default(),
          hash: format!("key-{pot}"),
        },
      );
    }
    set_module_graph_cache(vec!["styles.css".into(), "unrelated.css".into()], &context);
    context.cache_manager.module_cache.write_cache();
    context.cache_manager.resource_cache.write_cache();
    let mut graph = context.module_graph.write();
    let module = graph.module_mut(&"styles.css".into()).unwrap();
    assert!(!context.cache_manager.module_cache.is_cache_changed(module));
    module.meta = generated_css("new");
    drop(graph);
    context
  }

  #[test]
  fn explicit_same_source_update_replaces_generated_module_metadata() {
    let context = fixture();
    set_updated_modules_cache(&vec!["styles.css".into()], &DiffResult::default(), &context);
    let cached = context
      .cache_manager
      .module_cache
      .get_cache(&"styles.css".into());
    let content = css_content(&cached.module);
    assert!(
      content.contains(".new"),
      "explicitly updated module must keep generated metadata: {content}"
    );
    let unrelated = context
      .cache_manager
      .module_cache
      .get_cache(&"unrelated.css".into());
    assert!(css_content(&unrelated.module).contains(".old"));
  }

  #[test]
  fn explicit_same_source_update_invalidates_only_its_resource_pots() {
    let context = fixture();
    assert!(context.cache_manager.resource_cache.has_cache("styles-pot"));
    let unrelated_before = context
      .cache_manager
      .resource_cache
      .get_cache("unrelated-pot")
      .unwrap();
    set_updated_modules_cache(&vec!["styles.css".into()], &DiffResult::default(), &context);
    assert!(
      !context.cache_manager.resource_cache.has_cache("styles-pot"),
      "explicitly updated pot must not reuse its previous render"
    );
    assert!(
      context
        .cache_manager
        .resource_cache
        .has_cache("unrelated-pot")
    );
    let unrelated_after = context
      .cache_manager
      .resource_cache
      .get_cache("unrelated-pot")
      .unwrap();
    assert_eq!(
      farmfe_core::serialize!(&unrelated_before),
      farmfe_core::serialize!(&unrelated_after)
    );
  }

  #[test]
  fn explicit_immutable_update_persists_generated_metadata_after_restart() {
    let store = Arc::new(CacheContext {
      cache_enable: true,
      ..Default::default()
    });
    let mut config = Config::default();
    let mut cache = PersistentCacheConfig::get_default_config(&config.root).as_obj(&config.root);
    cache.memory = true;
    config.persistent_cache = Box::new(PersistentCacheConfig::Obj(cache));
    let mut context = CompilationContext::new(config, vec![]).unwrap();
    context.cache_manager.module_cache = ModuleCacheManager::new(store.clone());
    let mut context = Arc::new(context);
    for id in ["package/styles.css", "package/sibling.css"] {
      let mut module = Module::new(id.into());
      module.module_type = ModuleType::Css;
      module.immutable = true;
      module.package_name = "fixture-styles".into();
      module.package_version = "1.0.0".into();
      module.content_hash = "unchanged-source".into();
      module.meta = generated_css("old");
      context.module_graph.write().add_module(module);
    }
    set_module_graph_cache(
      vec!["package/styles.css".into(), "package/sibling.css".into()],
      &context,
    );
    context.cache_manager.module_cache.write_cache();
    // Reload the package manifest, rather than updating the initial empty store.
    Arc::get_mut(&mut context)
      .unwrap()
      .cache_manager
      .module_cache = ModuleCacheManager::new(store.clone());
    let old = context
      .cache_manager
      .module_cache
      .get_cache(&"package/styles.css".into());
    assert!(css_content(&old.module).contains(".old"));
    context
      .module_graph
      .write()
      .module_mut(&"package/styles.css".into())
      .unwrap()
      .meta = generated_css("new");
    set_updated_modules_cache(
      &vec!["package/styles.css".into()],
      &DiffResult::default(),
      &context,
    );
    context.cache_manager.module_cache.write_cache();
    let restored = ModuleCacheManager::new(store);
    let updated = restored.get_cache(&"package/styles.css".into());
    let content = css_content(&updated.module);
    assert!(
      content.contains(".new"),
      "explicit immutable update must persist generated metadata: {content}"
    );
    let sibling = restored.get_cache(&"package/sibling.css".into());
    assert!(css_content(&sibling.module).contains(".old"));
  }
}
