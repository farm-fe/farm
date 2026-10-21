use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  farm_profile_function, farm_profile_scope,
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge},
    ModuleId,
  },
  plugin::hooks::freeze_module::PluginFreezeModuleHookParam,
  rayon::iter::{IntoParallelRefMutIterator, ParallelIterator},
  HashMap, HashSet,
};

use super::module_cache::set_module_graph_cache;

/// Finalize module graph when module graph is built:
/// 1. call module_graph_build_end hook
/// 2. update execution order of module graph
pub fn finalize_module_graph(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  let mut module_graph = context.module_graph.write();

  {
    farm_profile_scope!("call module_graph_build_end hook".to_string());
    context
      .plugin_driver
      .module_graph_build_end(&mut module_graph, context)?;
  }

  // update execution order when the module graph is freezed in build stage
  module_graph.update_execution_order_for_modules();

  Ok(())
}

pub(super) fn cache_module_graph(context: &Arc<CompilationContext>) {
  let module_ids = {
    let module_graph = context.module_graph.read();
    module_graph
      .modules()
      .iter()
      .map(|m| m.id.clone())
      .collect::<Vec<_>>()
  };

  // set new module cache
  set_module_graph_cache(module_ids, context);
}

pub(super) fn freeze_module_of_module_graph(
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  farm_profile_function!("call freeze_module hook".to_string());
  let mut module_graph = context.module_graph.write();

  let module_ids = module_graph.modules().into_iter().map(|m| m.id.clone());

  let mut resolved_deps_map = get_resolved_deps_of_modules(module_ids.collect(), &module_graph);

  let hook_params = module_graph
    .modules_mut()
    .into_iter()
    .map(|module| {
      let resolved_deps = resolved_deps_map.remove(&module.id).unwrap();
      PluginFreezeModuleHookParam {
        module,
        resolved_deps,
      }
    })
    .collect::<Vec<_>>();

  let module_resolved_deps =
    call_freeze_module_with_params(hook_params, &HashSet::default(), context)?;

  update_modules_resolved_deps(module_resolved_deps, &mut module_graph);

  Ok(())
}

pub fn get_resolved_deps_of_modules(
  module_ids: Vec<ModuleId>,
  module_graph: &ModuleGraph,
) -> HashMap<ModuleId, Vec<(ModuleId, ModuleGraphEdge)>> {
  let mut resolved_deps = HashMap::default();

  for module_id in module_ids {
    let deps = module_graph.dependencies(&module_id);
    resolved_deps.insert(
      module_id,
      deps
        .into_iter()
        .map(|(id, edge)| (id, edge.clone()))
        .collect::<Vec<_>>(),
    );
  }

  resolved_deps
}

/// Call freeze module hook with persistent cache
pub fn call_freeze_module_hook(
  param: &mut PluginFreezeModuleHookParam,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  // the result of freeze_module will be used from module cache if the module does not change
  if context
    .cache_manager
    .module_cache
    .is_cache_changed(param.module)
  {
    return context.plugin_driver.freeze_module(param, context);
  }

  Ok(())
}

pub type ModuleResolvedDeps = Vec<(ModuleId, Vec<(ModuleId, ModuleGraphEdge)>)>;

pub fn call_freeze_module_with_params(
  mut hook_params: Vec<PluginFreezeModuleHookParam>,
  explicitly_updated_modules: &HashSet<ModuleId>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<ModuleResolvedDeps> {
  hook_params.par_iter_mut().try_for_each(|param| {
    // Update roots are rebuilt even when their source bytes are unchanged.
    // Their generated content may depend on metadata from newly added modules
    // (for example Tailwind candidates discovered during lazy compilation).
    // Newly added cached dependencies still use the normal cache decision.
    if explicitly_updated_modules.contains(&param.module.id) {
      context.plugin_driver.freeze_module(param, context)
    } else {
      call_freeze_module_hook(param, context)
    }
  })?;

  Ok(
    hook_params
      .into_iter()
      .map(|param| (param.module.id.clone(), param.resolved_deps))
      .collect::<Vec<_>>(),
  )
}

pub fn update_modules_resolved_deps(
  module_resolved_deps: Vec<(ModuleId, Vec<(ModuleId, ModuleGraphEdge)>)>,
  module_graph: &mut ModuleGraph,
) {
  for (module_id, resolved_deps) in module_resolved_deps {
    for (dep_id, edge) in resolved_deps {
      if module_graph.has_edge(&module_id, &dep_id) {
        module_graph.update_edge(&module_id, &dep_id, edge).unwrap();
      } else {
        module_graph.add_edge(&module_id, &dep_id, edge).unwrap();
      }
    }
  }
}

#[cfg(test)]
mod explicit_update_freeze_tests {
  use super::*;
  use farmfe_core::{
    cache::module_cache::CachedModule,
    config::{persistent_cache::PersistentCacheConfig, Config},
    module::Module,
    plugin::Plugin,
  };
  use std::sync::atomic::{AtomicUsize, Ordering};

  struct CountFreeze(Arc<AtomicUsize>);

  impl Plugin for CountFreeze {
    fn name(&self) -> &str {
      "CountFreeze"
    }

    fn freeze_module(
      &self,
      _param: &mut PluginFreezeModuleHookParam,
      _context: &Arc<CompilationContext>,
    ) -> farmfe_core::error::Result<Option<()>> {
      self.0.fetch_add(1, Ordering::SeqCst);
      Ok(Some(()))
    }
  }

  fn fixture() -> (Arc<CompilationContext>, Arc<AtomicUsize>, Module) {
    let mut config = Config::default();
    let mut cache = PersistentCacheConfig::get_default_config(&config.root).as_obj(&config.root);
    cache.memory = true;
    config.persistent_cache = Box::new(PersistentCacheConfig::Obj(cache));
    let calls = Arc::new(AtomicUsize::new(0));
    let context = Arc::new(
      CompilationContext::new(config, vec![Arc::new(CountFreeze(calls.clone()))]).unwrap(),
    );
    let mut module = Module::new(ModuleId::from("styles.css"));
    module.content_hash = "unchanged-css-source".to_string();
    context.cache_manager.module_cache.set_cache(
      module.id.clone(),
      CachedModule {
        module: module.clone(),
        dependencies: vec![],
        watch_dependencies: vec![],
        is_expired: false,
      },
    );
    context.cache_manager.module_cache.write_cache();
    assert!(!context.cache_manager.module_cache.is_cache_changed(&module));
    (context, calls, module)
  }

  #[test]
  fn explicitly_updated_unchanged_module_runs_freeze() {
    let (context, calls, mut module) = fixture();
    let mut updated = HashSet::default();
    updated.insert(module.id.clone());
    call_freeze_module_with_params(
      vec![PluginFreezeModuleHookParam {
        module: &mut module,
        resolved_deps: vec![],
      }],
      &updated,
      &context,
    )
    .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
  }

  #[test]
  fn cached_dependency_outside_explicit_update_set_keeps_frozen_output() {
    let (context, calls, mut module) = fixture();
    let mut updated = HashSet::default();
    updated.insert(ModuleId::from("other-updated.ts"));
    call_freeze_module_with_params(
      vec![PluginFreezeModuleHookParam {
        module: &mut module,
        resolved_deps: vec![],
      }],
      &updated,
      &context,
    )
    .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
  }

  #[test]
  fn initial_build_keeps_cached_freeze_output() {
    let (context, calls, mut module) = fixture();
    call_freeze_module_with_params(
      vec![PluginFreezeModuleHookParam {
        module: &mut module,
        resolved_deps: vec![],
      }],
      &HashSet::default(),
      &context,
    )
    .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
  }
}
