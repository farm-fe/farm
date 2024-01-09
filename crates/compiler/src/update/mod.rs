use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};

use farmfe_core::{
  cache::module_cache::CachedModule,
  context::CompilationContext,
  error::CompilationError,
  module::{
    module_graph::ModuleGraphEdgeDataItem, module_group::ModuleGroupId, Module, ModuleId,
    ModuleType,
  },
  plugin::{PluginResolveHookParam, ResolveKind, UpdateResult, UpdateType},
  resource::ResourceType,
};
use farmfe_plugin_css::transform_css_to_script::transform_css_to_script_modules;
use farmfe_toolkit::get_dynamic_resources_map::get_dynamic_resources_map;

use crate::{
  build::{
    module_cache::handle_cached_modules, BuildModuleGraphThreadedParams, HandleDependenciesParams,
    ResolvedModuleInfo,
  },
  generate::finalize_resources::finalize_resources,
  Compiler,
};
use farmfe_core::error::Result;

use self::{
  diff_and_patch_module_graph::{diff_module_graph, patch_module_graph, DiffResult},
  handle_update_modules::handle_update_modules,
  module_cache::set_updated_modules_cache,
  patch_module_group_graph::patch_module_group_graph,
  regenerate_resources::{
    regenerate_resources_for_affected_module_groups, render_and_generate_update_resource,
  },
  update_context::UpdateContext,
};

mod diff_and_patch_module_graph;
mod find_hmr_boundaries;
mod handle_update_modules;
mod module_cache;
mod patch_module_group_graph;
mod regenerate_resources;
mod update_context;

enum ResolveModuleResult {
  Cached(ModuleId),
  /// This module is already in previous module graph before the update, and we met it again when resolving dependencies
  ExistingBeforeUpdate(ModuleId),
  /// This module is added during the update, and we met it again when resolving dependencies
  ExistingWhenUpdate(ModuleId),
  /// This module is a full new resolved module, and we need to do the full building process
  Success(Box<ResolvedModuleInfo>),
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
  pub fn update<F>(
    &self,
    paths: Vec<(String, UpdateType)>,
    callback: F,
    sync: bool,
  ) -> Result<UpdateResult>
  where
    F: FnOnce() + Send + Sync + 'static,
  {
    let (err_sender, err_receiver) = Self::create_thread_channel();
    let update_context = Arc::new(UpdateContext::new());

    let watch_graph = self.context.watch_graph.read();
    let module_graph = self.context.module_graph.read();
    // fetch watch file relation module, and replace watch file
    let paths: Vec<(String, UpdateType)> = paths
      .into_iter()
      .flat_map(|(path, update_type)| {
        let id = ModuleId::new(&path, "", &self.context.config.root);

        if watch_graph.has_module(&id) {
          let r: Vec<(String, UpdateType)> = watch_graph
            .relation_roots(&id)
            .into_iter()
            .map(|item| {
              (
                item.resolved_path(&self.context.config.root),
                UpdateType::Updated,
              )
            })
            .collect();

          if module_graph.has_module(&ModuleId::new(path.as_str(), "", &self.context.config.root)) {
            return [r, vec![(path, update_type)]].concat();
          };

          if !r.is_empty() {
            r
          } else {
            vec![(path, update_type)]
          }
        } else {
          vec![(path, update_type)]
        }
      })
      .collect();

    drop(watch_graph);
    drop(module_graph);

    let mut old_watch_extra_resources: HashSet<ModuleId> = self
      .context
      .watch_graph
      .read()
      .modules()
      .into_iter()
      .cloned()
      .collect();

    let mut update_result = UpdateResult::default();
    let paths = handle_update_modules(paths, &self.context, &mut update_result)?;

    for (path, update_type) in paths.clone() {
      match update_type {
        UpdateType::Added => {
          return Err(farmfe_core::error::CompilationError::GenericError(
            "Added is not supported yet".to_string(),
          ));
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
              thread_pool: self.thread_pool.clone(),
              order: 0,
              cached_dependency: None,
            },
            order: None,
            update_context: update_context.clone(),
          };

          Self::update_module_graph_threaded(params);
        }
        UpdateType::Removed => {
          return Err(farmfe_core::error::CompilationError::GenericError(
            "Removed is not supported yet".to_string(),
          ));
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
      return Err(CompilationError::GenericError(
        errors
          .into_iter()
          .map(|e| e.to_string())
          .collect::<Vec<_>>()
          .join("\n"),
      ));
    }

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

    // update cache
    set_updated_modules_cache(&updated_module_ids, &diff_result, &self.context);

    // TODO Add a separate hook after module graph are updated
    let mut module_ids = updated_module_ids.clone();
    module_ids.extend(diff_result.added_modules.clone());
    transform_css_to_script_modules(module_ids, &self.context)?;

    let dynamic_resources_map = self.regenerate_resources(
      affected_module_groups,
      previous_module_groups,
      &updated_module_ids,
      diff_result.clone(),
      removed_modules,
      callback,
      sync,
    );

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

      watch_diff_result.remove.extend(
        old_watch_extra_resources
          .into_iter()
          .map(|r| r.resolved_path(&self.context.config.root)),
      );
    }

    // If the module type is not script, we should skip render and generate update resource.
    // and just return `window.location.reload()`
    let should_reload_page = updated_module_ids.iter().any(|id| {
      let module_graph = self.context.module_graph.read();
      let module = module_graph.module(id).unwrap();
      !module.module_type.is_script()
    });
    let (immutable_resources, mutable_resources) = if should_reload_page {
      ("window.location.reload()".to_string(), "{}".to_string())
    } else {
      render_and_generate_update_resource(&updated_module_ids, &diff_result, &self.context)?
    };

    // find the boundaries.
    let boundaries = find_hmr_boundaries::find_hmr_boundaries(&updated_module_ids, &self.context);

    // TODO: support sourcemap for hmr. and should generate the hmr update response body in rust side.
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
          let mut graph_watch = context.watch_graph.write();
          graph_watch.delete_module(&module.id);
          drop(graph_watch);

          if resolve_module_id_result.resolve_result.external {
            // insert external module to the graph
            let module_id: ModuleId = resolve_param.source.as_str().into();
            let mut module = Module::new(module_id.clone());
            module.external = true;

            Self::add_module_to_update_module_graph(&update_context, module);
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
    Self::add_module_to_update_module_graph(&update_context, module);
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

  fn add_module_to_update_module_graph(update_context: &Arc<UpdateContext>, module: Module) {
    let mut update_module_graph = update_context.module_graph.write();

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
    HashSet<ModuleId>,
    Vec<ModuleId>,
    DiffResult,
    HashMap<ModuleId, Module>,
  ) {
    let start_points: Vec<ModuleId> = paths
      .into_iter()
      // Note: HMR does not support the module with query
      .map(|path| ModuleId::from_resolved_path_with_query(&path.0, &self.context.config.root))
      .collect();
    let mut module_graph = self.context.module_graph.write();
    let mut update_module_graph = update_context.module_graph.write();

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

    (
      affected_module_groups,
      start_points,
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
  ) -> Option<HashMap<ModuleId, Vec<(String, ResourceType)>>>
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
      )
      .unwrap();

      finalize_resources(&cloned_context).unwrap();
      let module_group_graph = self.context.module_group_graph.read();
      let resource_pot_map = self.context.resource_pot_map.read();
      let resources_map = self.context.resources_map.lock();
      let module_graph = self.context.module_graph.read();
      let html_entries_ids = module_graph
        .entries
        .clone()
        .into_iter()
        .filter_map(|(m, _)| {
          let module = module_graph.module(&m).unwrap();
          if matches!(module.module_type, ModuleType::Html) {
            Some(m)
          } else {
            None
          }
        })
        .collect::<Vec<_>>();
      let mut dynamic_resources = HashMap::new();

      for html_entry_id in html_entries_ids {
        dynamic_resources.extend(get_dynamic_resources_map(
          &module_group_graph,
          &html_entry_id,
          &resource_pot_map,
          &resources_map,
        ));
      }

      dynamic_resources_map = Some(dynamic_resources);
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
          println!("Failed to regenerate resources: {}", e);
          println!("modules to regenerate: {:?}", cloned_updated_module_ids);
        }

        finalize_resources(&cloned_context).unwrap();
        callback();
      });
    }

    dynamic_resources_map
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

  let module_graph = context.module_graph.read();
  if module_graph.has_module(&module_id) {
    return Ok(ResolveModuleResult::ExistingBeforeUpdate(module_id));
  }

  let mut update_module_graph = update_context.module_graph.write();
  if update_module_graph.has_module(&module_id) {
    return Ok(ResolveModuleResult::ExistingWhenUpdate(module_id));
  }

  if let Some(cached_dependency) = cached_dependency {
    let module_cache_manager = &context.cache_manager.module_cache;

    if module_cache_manager.has_cache(&cached_dependency) {
      return Ok(ResolveModuleResult::Cached(cached_dependency));
    }
  }

  let resolve_module_id_result =
    resolve_module_id_result.unwrap_or(Compiler::resolve_module_id(resolve_param, context)?);
  // just a placeholder module, it will be replaced by the real module later
  update_module_graph.add_module(Compiler::create_module(
    resolve_module_id_result.module_id.clone(),
    false,
    false,
  ));

  Ok(ResolveModuleResult::Success(Box::new(ResolvedModuleInfo {
    module: Compiler::create_module(
      resolve_module_id_result.module_id.clone(),
      resolve_module_id_result.resolve_result.external,
      // TODO: make it configurable
      context
        .config
        .partial_bundling
        .immutable_modules
        .iter()
        .any(|im| im.is_match(&module_id.to_string())),
    ),
    resolve_module_id_result,
  })))
}
