use std::sync::Arc;

use farmfe_core::{
  HashMap, HashSet,
  cache::module_cache::CachedModule,
  context::CompilationContext,
  error::CompilationError,
  module::{
    Module, ModuleId, ModuleType,
    module_graph::ModuleGraphEdgeDataItem,
    module_group::{ModuleGroupId, ModuleGroupType},
  },
  plugin::{PluginResolveHookParam, ResolveKind, UpdateResult, UpdateType},
  resource::ResourceType,
  serde::Serialize,
  serde_json::{self, json},
  stats::CompilationPluginHookStats,
};

use farmfe_toolkit::resources::get_dynamic_resources_map;
use finalize_module_graph::{
  finalize_updated_module_graph, freeze_module_of_affected_module_graph,
};

use crate::{
  Compiler,
  build::{
    BuildModuleGraphThreadedParams, HandleDependenciesParams, ResolvedModuleInfo,
    module_cache::handle_cached_modules,
  },
  generate::finalize_resources::finalize_resources,
};
use farmfe_core::error::Result;

use self::{
  diff_and_patch_module_graph::{DiffResult, diff_module_graph, patch_module_graph},
  handle_update_modules::handle_update_modules,
  module_cache::set_updated_modules_cache,
  patch_module_group_graph::patch_module_group_graph,
  regenerate_resources::{
    regenerate_resources_for_affected_module_groups, render_and_generate_update_resource,
  },
  update_context::UpdateContext,
};

mod diff_and_patch_module_graph;
mod finalize_module_graph;
mod find_hmr_boundaries;
mod handle_update_modules;
mod module_cache;
mod patch_module_group_graph;
mod regenerate_resources;
mod update_context;

type DynamicResourcesMap = HashMap<ModuleId, Vec<(String, ResourceType)>>;

enum ResolveModuleResult {
  Cached(ModuleId),
  /// This module is already in previous module graph before the update, and we met it again when resolving dependencies
  ExistingBeforeUpdate(ModuleId),
  /// This module is added during the update, and we met it again when resolving dependencies
  ExistingWhenUpdate(ModuleId),
  /// This module is a full new resolved module, and we need to do the full building process
  Success(Box<ResolvedModuleInfo>),
}

#[cfg(test)]
mod update_finished_lock_tests {
  use super::*;
  use farmfe_core::{config::Config, plugin::Plugin};

  struct ResourceReader;

  struct CompletionOrder(Arc<std::sync::atomic::AtomicBool>);

  struct FinalizeFailure;
  impl Plugin for FinalizeFailure {
    fn name(&self) -> &str {
      "FinalizeFailure"
    }

    fn finalize_resources(
      &self,
      _param: &mut farmfe_core::plugin::PluginFinalizeResourcesHookParam,
      _context: &Arc<CompilationContext>,
    ) -> Result<Option<()>> {
      Err(CompilationError::GenericError(
        "P08 deliberate resource finalization failure".into(),
      ))
    }
  }

  struct RegenerationFailure;
  impl Plugin for RegenerationFailure {
    fn name(&self) -> &str {
      "RegenerationFailure"
    }

    fn process_resource_pots(
      &self,
      _pots: &mut Vec<&mut farmfe_core::resource::resource_pot::ResourcePot>,
      _context: &Arc<CompilationContext>,
    ) -> Result<Option<()>> {
      Err(CompilationError::GenericError(
        "P08 deliberate regeneration failure".into(),
      ))
    }
  }

  struct CompletionFailure;
  impl Plugin for CompletionFailure {
    fn name(&self) -> &str {
      "CompletionFailure"
    }

    fn update_finished(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
      Err(CompilationError::GenericError(
        "P08 deliberate completion failure".into(),
      ))
    }
  }

  #[test]
  fn resource_finalization_error_is_returned_without_panic_or_success_callback() {
    assert_failed_update(
      Arc::new(FinalizeFailure),
      "P08 deliberate resource finalization failure",
    );
  }

  #[test]
  fn resource_regeneration_error_is_returned_without_panic_or_success_callback() {
    assert_failed_update(
      Arc::new(RegenerationFailure),
      "P08 deliberate regeneration failure",
    );
  }

  #[test]
  fn completion_hook_error_is_returned_without_panic_or_success_callback() {
    assert_failed_update(
      Arc::new(CompletionFailure),
      "P08 deliberate completion failure",
    );
  }

  fn assert_failed_update(plugin: Arc<dyn Plugin>, message: &str) {
    let completed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let compiler = Compiler::new(Config::default(), vec![plugin]).unwrap();
    let callback_completed = completed.clone();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      compiler.regenerate_resources(
        HashSet::default(),
        HashSet::default(),
        &vec![],
        DiffResult::default(),
        HashMap::default(),
        move || {
          callback_completed.store(true, std::sync::atomic::Ordering::SeqCst);
        },
        true,
      )
    }));
    assert!(outcome.is_ok(), "recoverable update error must not panic");
    let error = outcome
      .unwrap()
      .expect_err("failed update must return its error");
    assert!(
      error.to_string().contains(message),
      "the original diagnostic must reach the update caller"
    );
    assert!(!completed.load(std::sync::atomic::Ordering::SeqCst));
  }

  impl Plugin for CompletionOrder {
    fn name(&self) -> &str {
      "CompletionOrder"
    }
    fn update_finished(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
      assert!(
        !self.0.load(std::sync::atomic::Ordering::SeqCst),
        "completion callback ran before update_finished hooks"
      );
      Ok(None)
    }
  }

  #[test]
  fn completion_callback_follows_update_finished_hooks() {
    let completed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let compiler = Compiler::new(
      Config::default(),
      vec![Arc::new(CompletionOrder(completed.clone()))],
    )
    .unwrap();
    let callback_completed = completed.clone();
    compiler
      .regenerate_resources(
        HashSet::default(),
        HashSet::default(),
        &vec![],
        DiffResult::default(),
        HashMap::default(),
        move || {
          callback_completed.store(true, std::sync::atomic::Ordering::SeqCst);
        },
        true,
      )
      .unwrap();
    assert!(completed.load(std::sync::atomic::Ordering::SeqCst));
  }

  impl Plugin for ResourceReader {
    fn name(&self) -> &str {
      "ResourceReader"
    }

    fn update_finished(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
      assert!(
        context.resources_map.try_lock().is_some(),
        "update_finished must be able to read generated resources"
      );
      assert!(context.module_graph.try_write().is_some());
      assert!(context.module_group_graph.try_write().is_some());
      assert!(context.resource_pot_map.try_write().is_some());
      Ok(None)
    }
  }

  #[test]
  fn synchronous_update_releases_graph_and_resource_locks_before_hooks() {
    let compiler = Compiler::new(Config::default(), vec![Arc::new(ResourceReader)]).unwrap();
    compiler
      .regenerate_resources(
        HashSet::default(),
        HashSet::default(),
        &vec![],
        DiffResult::default(),
        HashMap::default(),
        || {},
        true,
      )
      .unwrap();
  }
}

struct BuildUpdateModuleGraphThreadedParams {
  build_module_graph_threaded_params: BuildModuleGraphThreadedParams,
  // if order is [None], means this is the root update module, it should always be rebuilt
  order: Option<usize>,
  update_context: Arc<UpdateContext>,
}

struct HandleUpdateDependenciesParams {
  handle_dependencies_params: HandleDependenciesParams,
  order: Option<usize>,
  update_context: Arc<UpdateContext>,
}

impl Compiler {
  fn set_update_module_graph_stats(&self, update_context: &Arc<UpdateContext>) {
    if self.context.config.record {
      let update_module_graph = update_context.module_graph.read();
      self
        .context
        .stats
        .set_module_graph_stats(&update_module_graph);
      self
        .context
        .stats
        .set_entries(update_module_graph.entries.keys().cloned().collect())
    }
  }

  fn set_module_group_graph_stats(&self) {
    if self.context.config.record {
      let module_group_graph = self.context.module_group_graph.read();
      self
        .context
        .stats
        .add_plugin_hook_stats(CompilationPluginHookStats {
          plugin_name: "HmrUpdate".to_string(),
          hook_name: "analyze_module_graph".to_string(),
          hook_context: None,
          module_id: "".into(),
          input: "".to_string(),
          output: serde_json::to_string(&module_group_graph.print_graph()).unwrap(),
          duration: 0,
          start_time: 0,
          end_time: 0,
        })
    }
  }

  fn set_hmr_diff_stats(
    &self,
    removed_modules: &HashMap<ModuleId, Module>,
    affected_module_groups: &HashSet<ModuleGroupId>,
    diff_result: &DiffResult,
    start_points: &Vec<ModuleId>,
  ) {
    // record diff and patch result
    if self.context.config.record {
      self
        .context
        .stats
        .add_plugin_hook_stats(CompilationPluginHookStats {
          plugin_name: "HmrUpdate".to_string(),
          hook_name: "diffAndPatchContext".to_string(),
          module_id: "".into(),
          hook_context: None,
          input: "".to_string(),
          output: serde_json::to_string(&PrintedDiffAndPatchContext {
            removed_modules: removed_modules.iter().map(|m| m.0.clone()).collect(),
            affected_module_groups: affected_module_groups.clone(),
            diff_result: diff_result.clone(),
            start_points: start_points.clone(),
          })
          .unwrap_or_default(),
          duration: 0,
          start_time: 0,
          end_time: 0,
        });
    }
  }

  pub fn update<F>(
    &self,
    paths: Vec<(String, UpdateType)>,
    callback: F,
    sync: bool,
    generate_update_resource: bool,
  ) -> Result<UpdateResult>
  where
    F: FnOnce() + Send + Sync + 'static,
  {
    self.context.stats.add_hmr_compilation_stats();
    self.context.stats.set_start_time();

    // mark the compilation as update
    // self.context.set_update();
    let (err_sender, err_receiver) = Self::create_thread_channel();
    let update_context = Arc::new(UpdateContext::new());

    let mut old_watch_extra_resources: HashSet<ModuleId> = self
      .context
      .watch_graph
      .read()
      .modules()
      .into_iter()
      .cloned()
      .collect();

    let mut update_result = UpdateResult::default();
    self.context.clear_log_store();

    let last_failed_module_ids = self.last_fail_module_ids.lock();
    let paths = handle_update_modules(
      paths,
      &last_failed_module_ids,
      &self.context,
      &mut update_result,
    )?;
    drop(last_failed_module_ids);

    for (path, update_type) in paths.clone() {
      match update_type {
        UpdateType::Added | UpdateType::Removed => {
          /* Do nothing for Added and Removed by default, they should be handled by plugin in update_modules hook */
        }

        UpdateType::Updated => {
          let resolve_param = PluginResolveHookParam {
            kind: ResolveKind::HmrUpdate,
            source: path,
            importer: None,
          };

          let params = BuildUpdateModuleGraphThreadedParams {
            build_module_graph_threaded_params: BuildModuleGraphThreadedParams {
              resolve_param,
              context: self.context.clone(),
              err_sender: err_sender.clone(),
              thread_pool: self.context.thread_pool.clone(),
              order: 0,
              cached_dependency: None,
            },
            order: None,
            update_context: update_context.clone(),
          };

          Self::update_module_graph_threaded(params);
        }
      }
    }

    drop(err_sender);

    let mut errors = vec![];

    while let Ok(err) = err_receiver.recv() {
      errors.push(err);
    }

    self.handle_global_log(&mut errors);

    if !errors.is_empty() {
      self.context.stats.set_build_end_time();
      self.context.stats.set_end_time();
      self.set_update_module_graph_stats(&update_context);
      self.set_last_fail_module_ids(&errors);

      let mut error_messages = vec![];
      for error in errors {
        error_messages.push(error.to_string());
      }
      let errors_json = json!(
        error_messages
          .iter()
          .map(|e| e.to_string())
          .collect::<Vec<_>>()
      );
      return Err(CompilationError::GenericError(errors_json.to_string()));
    } else {
      self.set_last_fail_module_ids(&[]);
    }

    self.context.stats.set_build_end_time();
    self.set_update_module_graph_stats(&update_context);

    let previous_module_groups = {
      let module_group_graph = self.context.module_group_graph.read();
      module_group_graph
        .module_groups()
        .into_iter()
        .map(|m| m.id.clone())
        .collect::<HashSet<_>>()
    };

    let (affected_module_groups, updated_module_ids, diff_result, removed_modules) =
      self.diff_and_patch_context(paths, &update_context);
    // record graph patch result
    self.set_module_group_graph_stats();

    freeze_module_of_affected_module_graph(&updated_module_ids, &diff_result, &self.context)?;

    // Remember existing CSS before module_graph_updated converts it to script.
    // Lazy navigation must update already-loaded styles, but must not duplicate
    // every newly discovered page/dependency in the lazy response.
    let lazy_css_modules = if !generate_update_resource
      && self.context.config.lazy_compilation
      && self.context.config.output.target_env.is_browser()
    {
      let graph = self.context.module_graph.read();
      updated_module_ids
        .iter()
        .filter(|id| {
          graph
            .module(id)
            .is_some_and(|module| module.module_type == ModuleType::Css)
        })
        .cloned()
        .collect::<Vec<_>>()
    } else {
      vec![]
    };

    // update cache
    if self.context.config.persistent_cache.enabled() {
      set_updated_modules_cache(&updated_module_ids, &diff_result, &self.context);
    }

    finalize_updated_module_graph(
      &updated_module_ids,
      removed_modules.keys().cloned().collect(),
      &diff_result,
      &self.context,
    )?;

    // after update_module, diff old_resource and new_resource
    {
      let watch_graph = self.context.watch_graph.read();
      let module_ids: HashSet<&ModuleId> = watch_graph.modules().into_iter().collect();

      let watch_diff_result = &mut update_result.extra_watch_result;

      for id in module_ids {
        if !old_watch_extra_resources.remove(id) {
          watch_diff_result
            .add
            .push(id.resolved_path(&self.context.config.root));
        };
      }
    }

    // If the module type is not script, we should skip render and generate update resource.
    // and just return `window.location.reload()`
    let should_reload_page = updated_module_ids.iter().any(|id| {
      let module_graph = self.context.module_graph.read();
      let module = module_graph.module(id).unwrap();
      !module.module_type.is_script()
    });

    let (immutable_resources, mutable_resources) = if should_reload_page && generate_update_resource
    {
      ("window.location.reload()".to_string(), "{}".to_string())
    } else if generate_update_resource {
      render_and_generate_update_resource(&updated_module_ids, &diff_result, &self.context)?
    } else if !lazy_css_modules.is_empty() {
      let css_scripts = {
        let graph = self.context.module_graph.read();
        lazy_css_modules
          .into_iter()
          .filter(|id| {
            graph
              .module(id)
              .is_some_and(|module| module.module_type.is_script())
          })
          .collect::<Vec<_>>()
      };
      render_and_generate_update_resource(&css_scripts, &DiffResult::default(), &self.context)?
    } else {
      ("{}".to_string(), "{}".to_string())
    };

    // find the boundaries.
    let boundaries = find_hmr_boundaries::find_hmr_boundaries(&updated_module_ids, &self.context);

    let dynamic_resources_map = self.regenerate_resources(
      affected_module_groups,
      previous_module_groups,
      &updated_module_ids,
      diff_result.clone(),
      removed_modules,
      callback,
      sync,
    )?;

    update_result
      .added_module_ids
      .extend(diff_result.added_modules);
    update_result.updated_module_ids.extend(updated_module_ids);
    update_result
      .removed_module_ids
      .extend(diff_result.removed_modules);
    update_result.immutable_resources = immutable_resources;
    update_result.mutable_resources = mutable_resources;
    update_result.boundaries = boundaries;
    update_result.dynamic_resources_map = dynamic_resources_map;

    Ok(update_result)
  }

  /// Resolving, loading, transforming and parsing a module in a separate thread.
  /// This method is similar to the build_module_graph_threaded method in the build/mod.rs file,
  /// the difference is that this method is used for updating the module graph, only handles the updated and added module, and ignores the existing unchanged module,
  /// while the build_module_threaded method is used for building full module graph and every module is handled.
  fn update_module_graph_threaded(params: BuildUpdateModuleGraphThreadedParams) {
    let BuildUpdateModuleGraphThreadedParams {
      build_module_graph_threaded_params:
        BuildModuleGraphThreadedParams {
          resolve_param,
          context,
          err_sender,
          thread_pool,
          order: _,
          cached_dependency,
        },
      order,
      update_context,
    } = params;
    let c_thread_pool = thread_pool.clone();

    thread_pool.spawn(move || {
      let resolve_module_result = match resolve_module(
        &resolve_param,
        cached_dependency,
        &context,
        &update_context,
        order.is_none(),
      ) {
        Ok(result) => result,
        Err(e) => {
          err_sender.send(e).unwrap();
          return;
        }
      };

      match resolve_module_result {
        ResolveModuleResult::Cached(module_id) => {
          let mut cached_module = context.cache_manager.module_cache.get_cache(&module_id);
          // if the dependency is immutable, skip building
          if let Err(e) = handle_cached_modules(&mut cached_module, &context) {
            err_sender.send(e).unwrap();
            return;
          }

          let handle_dependencies_params = HandleDependenciesParams {
            module: cached_module.module,
            resolve_param,
            order: order.unwrap_or(0),
            deps: CachedModule::dep_sources(cached_module.dependencies),
            thread_pool: c_thread_pool,
            err_sender,
            context,
          };

          let params = HandleUpdateDependenciesParams {
            handle_dependencies_params,
            order,
            update_context,
          };

          Self::handle_update_dependencies(params);
        }
        ResolveModuleResult::ExistingBeforeUpdate(module_id) => {
          // if the module does not exist, insert a placeholder module to the update module graph
          {
            let mut update_module_graph = update_context.module_graph.write();

            if !update_module_graph.has_module(&module_id) {
              let module = Module::new(module_id.clone());
              update_module_graph.add_module(module);
            }
          }
          Self::add_edge_to_update_module_graph(&update_context, &resolve_param, &module_id, order);
        }
        ResolveModuleResult::ExistingWhenUpdate(module_id) => {
          Self::add_edge_to_update_module_graph(&update_context, &resolve_param, &module_id, order);
        }
        ResolveModuleResult::Success(box ResolvedModuleInfo {
          mut module,
          resolve_module_id_result,
        }) => {
          if resolve_module_id_result.resolve_result.external {
            // insert external module to the graph
            let module_id: ModuleId = resolve_param.source.as_str().into();
            let mut module = Module::new(module_id.clone());
            module.external = true;

            Self::add_module_to_update_module_graph(&update_context, &resolve_param.kind, module);
            Self::add_edge_to_update_module_graph(
              &update_context,
              &resolve_param,
              &module_id,
              order,
            );
            return;
          }

          match Self::build_module(
            resolve_module_id_result.resolve_result,
            &mut module,
            &context,
            // Explicit update roots must rebuild generated metadata even when
            // their source bytes and timestamp are unchanged.
            order.is_some(),
          ) {
            Ok(deps) => {
              let params = HandleUpdateDependenciesParams {
                handle_dependencies_params: HandleDependenciesParams {
                  resolve_param,
                  context,
                  err_sender,
                  thread_pool: c_thread_pool,
                  module,
                  deps,
                  order: order.unwrap_or(0),
                },
                order,
                update_context,
              };
              Self::handle_update_dependencies(params);
            }
            Err(e) => {
              err_sender.send(e).unwrap();
            }
          }
        }
      }
    });
  }

  fn handle_update_dependencies(params: HandleUpdateDependenciesParams) {
    let HandleUpdateDependenciesParams {
      handle_dependencies_params:
        HandleDependenciesParams {
          resolve_param,
          context,
          err_sender,
          thread_pool,
          module,
          deps,
          ..
        },
      order,
      update_context,
    } = params;

    let module_id = module.id.clone();
    let immutable = module.immutable;
    Self::add_module_to_update_module_graph(&update_context, &resolve_param.kind, module);
    Self::add_edge_to_update_module_graph(&update_context, &resolve_param, &module_id, order);

    for (order, (dep, cached_dependency)) in deps.into_iter().enumerate() {
      let params = BuildUpdateModuleGraphThreadedParams {
        build_module_graph_threaded_params: BuildModuleGraphThreadedParams {
          thread_pool: thread_pool.clone(),
          resolve_param: PluginResolveHookParam {
            source: dep.source,
            importer: Some(module_id.clone()),
            kind: dep.kind,
          },
          context: context.clone(),
          err_sender: err_sender.clone(),
          order,
          cached_dependency: if immutable { cached_dependency } else { None },
        },
        order: Some(order),
        update_context: update_context.clone(),
      };
      Self::update_module_graph_threaded(params);
    }
  }

  fn add_module_to_update_module_graph(
    update_context: &Arc<UpdateContext>,
    kind: &ResolveKind,
    module: Module,
  ) {
    let mut update_module_graph = update_context.module_graph.write();

    if ResolveKind::HmrUpdate == *kind {
      update_module_graph
        .entries
        .insert(module.id.clone(), module.id.to_string());
    }

    if update_module_graph.has_module(&module.id) {
      update_module_graph.replace_module(module);
    } else {
      update_module_graph.add_module(module);
    }
  }

  fn add_edge_to_update_module_graph(
    update_context: &Arc<UpdateContext>,
    resolve_param: &PluginResolveHookParam,
    module_id: &ModuleId,
    order: Option<usize>,
  ) {
    let mut update_module_graph = update_context.module_graph.write();

    if let Some(order) = order {
      let importer = resolve_param.importer.as_ref().unwrap().clone();

      update_module_graph
        .add_edge_item(
          &importer,
          module_id,
          ModuleGraphEdgeDataItem {
            kind: resolve_param.kind.clone(),
            source: resolve_param.source.clone(),
            order,
          },
        )
        .expect("Both the importer and the module should be in the update module graph");
    }
  }

  fn diff_and_patch_context(
    &self,
    paths: Vec<(String, UpdateType)>,
    update_context: &Arc<UpdateContext>,
  ) -> (
    HashSet<ModuleGroupId>,
    Vec<ModuleId>,
    DiffResult,
    HashMap<ModuleId, Module>,
  ) {
    let start_points: Vec<ModuleId> = paths
      .into_iter()
      .filter(|(_, ty)| matches!(ty, UpdateType::Updated))
      // Note: HMR does not support the module with query
      .map(|path| ModuleId::from_resolved_path_with_query(&path.0, &self.context.config.root))
      .collect();
    let mut module_graph = self.context.module_graph.write();
    let mut update_module_graph = update_context.module_graph.write();
    update_module_graph.update_execution_order_for_modules();

    let diff_result = diff_module_graph(start_points.clone(), &module_graph, &update_module_graph);

    let removed_modules = patch_module_graph(
      start_points.clone(),
      &diff_result,
      &mut module_graph,
      &mut update_module_graph,
    );

    let mut module_group_graph = self.context.module_group_graph.write();

    let affected_module_groups = patch_module_group_graph(
      start_points.clone(),
      &diff_result,
      &removed_modules,
      &mut module_graph,
      &mut module_group_graph,
    );

    // record diff and patch result
    self.set_hmr_diff_stats(
      &removed_modules,
      &affected_module_groups,
      &diff_result,
      &start_points,
    );

    (
      affected_module_groups,
      start_points
        .into_iter()
        .filter(|id| !diff_result.added_modules.contains(id))
        .collect(), // remove added modules from start_points
      diff_result,
      removed_modules,
    )
  }

  fn regenerate_resources<F>(
    &self,
    affected_module_groups: HashSet<ModuleGroupId>,
    previous_module_groups: HashSet<ModuleGroupId>,
    updated_module_ids: &Vec<ModuleId>,
    diff_result: DiffResult,
    removed_modules: HashMap<ModuleId, Module>,
    callback: F,
    sync: bool,
  ) -> Result<Option<DynamicResourcesMap>>
  where
    F: FnOnce() + Send + Sync + 'static,
  {
    let mut dynamic_resources_map = None;
    let cloned_updated_module_ids = updated_module_ids.clone();
    let cloned_context = self.context.clone();

    // if there are new module groups, we should run the tasks synchronously
    if sync
      || affected_module_groups
        .iter()
        .any(|ag| !previous_module_groups.contains(ag))
    {
      regenerate_resources_for_affected_module_groups(
        affected_module_groups,
        diff_result,
        &cloned_updated_module_ids,
        &removed_modules,
        &cloned_context,
      )?;

      finalize_resources(&cloned_context)?;
      let module_group_graph = self.context.module_group_graph.read();
      let resource_pot_map = self.context.resource_pot_map.read();
      let resources_map = self.context.resources_map.lock();
      let module_graph = self.context.module_graph.read();

      let mut dynamic_resources = HashMap::default();

      for entry_id in module_graph.entries.keys() {
        dynamic_resources.extend(get_dynamic_resources_map(
          &module_group_graph,
          &ModuleGroupId::new(entry_id, &ModuleGroupType::Entry),
          &resource_pot_map,
          &resources_map,
          &module_graph,
        ));
      }

      dynamic_resources_map = Some(dynamic_resources);
      // Hooks and queued updates may acquire these locks again.
      drop(module_graph);
      drop(resources_map);
      drop(resource_pot_map);
      drop(module_group_graph);
      self.context.plugin_driver.update_finished(&self.context)?;
      self.context.stats.set_end_time();
      callback();
    } else {
      std::thread::spawn(move || {
        if let Err(e) = regenerate_resources_for_affected_module_groups(
          affected_module_groups,
          diff_result,
          &cloned_updated_module_ids,
          &removed_modules,
          &cloned_context,
        ) {
          println!("Failed to regenerate resources: {e}");
          println!("modules to regenerate: {cloned_updated_module_ids:?}");
        }

        finalize_resources(&cloned_context).unwrap();
        cloned_context
          .plugin_driver
          .update_finished(&cloned_context)
          .unwrap();
        cloned_context.stats.set_end_time();
        callback();
      });
    }

    // The legacy asynchronous branch completes through its callback. Callers
    // requiring generation errors before publishing an update must use sync.
    Ok(dynamic_resources_map)
  }
}

/// Similar to [crate::build::resolve_module], but the resolved module may be existed in both context and update_context
fn resolve_module(
  resolve_param: &PluginResolveHookParam,
  cached_dependency: Option<ModuleId>,
  context: &Arc<CompilationContext>,
  update_context: &Arc<UpdateContext>,
  is_root: bool,
) -> Result<ResolveModuleResult> {
  let mut resolve_module_id_result = None; //  Compiler::resolve_module_id(resolve_param, context)?;
  let module_id = if let Some(cached_dependency) = &cached_dependency {
    cached_dependency.clone()
  } else {
    resolve_module_id_result = Some(Compiler::resolve_module_id(resolve_param, context)?);
    resolve_module_id_result.as_ref().unwrap().module_id.clone()
  };

  // if this is the root module, we should always rebuild it
  if is_root {
    let resolve_module_id_result =
      resolve_module_id_result.expect("For root module, resolve_module_id_result should be Some");
    return Ok(ResolveModuleResult::Success(Box::new(ResolvedModuleInfo {
      module: Compiler::create_module(
        resolve_module_id_result.module_id.clone(),
        resolve_module_id_result.resolve_result.external,
        context
          .config
          .partial_bundling
          .immutable_modules
          .iter()
          .any(|im| im.is_match(&module_id.to_string())),
      ),
      resolve_module_id_result,
    })));
  }

  let module_graph = context.module_graph.write();
  if module_graph.has_module(&module_id) {
    return Ok(ResolveModuleResult::ExistingBeforeUpdate(module_id));
  }
  drop(module_graph);

  let mut update_module_graph = update_context.module_graph.write();
  if update_module_graph.has_module(&module_id) {
    return Ok(ResolveModuleResult::ExistingWhenUpdate(module_id));
  }

  if let Some(cached_dependency) = cached_dependency {
    let module_cache_manager = &context.cache_manager.module_cache;

    if module_cache_manager.has_cache(&cached_dependency) {
      let cached_module = module_cache_manager.get_cache_ref(&cached_dependency);
      let should_invalidate_cached_module = context
        .plugin_driver
        .handle_persistent_cached_module(&cached_module.module, context)?
        .unwrap_or(false);

      if should_invalidate_cached_module {
        module_cache_manager.invalidate_cache(&cached_dependency);
      } else {
        Compiler::insert_dummy_module(&cached_dependency, &mut update_module_graph);
        return Ok(ResolveModuleResult::Cached(cached_dependency));
      }
    }
  }

  let resolve_module_id_result =
    resolve_module_id_result.unwrap_or(Compiler::resolve_module_id(resolve_param, context)?);
  // just a placeholder module, it will be replaced by the real module later
  Compiler::insert_dummy_module(
    &resolve_module_id_result.module_id,
    &mut update_module_graph,
  );

  Ok(ResolveModuleResult::Success(Box::new(ResolvedModuleInfo {
    module: Compiler::create_module_from_resolve_result(&resolve_module_id_result, context),
    resolve_module_id_result,
  })))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", crate = "farmfe_core::serde")]
struct PrintedDiffAndPatchContext {
  affected_module_groups: HashSet<ModuleGroupId>,
  start_points: Vec<ModuleId>,
  diff_result: DiffResult,
  removed_modules: HashSet<ModuleId>,
}
