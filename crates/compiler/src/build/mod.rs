use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{
    mpsc::{channel, Receiver, Sender},
    Arc,
  },
};

use farmfe_core::{
  cache::module_cache::CachedModule,
  context::CompilationContext,
  error::{CompilationError, Result},
  farm_profile_function, farm_profile_scope,
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdgeDataItem},
    Module, ModuleId, ModuleType,
  },
  plugin::{
    constants::PLUGIN_BUILD_STAGE_META_RESOLVE_KIND,
    plugin_driver::PluginDriverTransformHookResult, PluginAnalyzeDepsHookResultEntry,
    PluginHookContext, PluginLoadHookParam, PluginParseHookParam, PluginProcessModuleHookParam,
    PluginResolveHookParam, PluginResolveHookResult, PluginTransformHookParam, ResolveKind,
  },
  rayon::ThreadPool,
  serde_json::json,
};

use farmfe_plugin_lazy_compilation::DYNAMIC_VIRTUAL_SUFFIX;
use farmfe_toolkit::resolve::load_package_json;
use farmfe_utils::stringify_query;

use crate::{
  build::{
    analyze_deps::analyze_deps, finalize_module::finalize_module, load::load, parse::parse,
    resolve::resolve, transform::transform,
  },
  utils::get_module_ids_from_compilation_errors,
  Compiler,
};

use self::module_cache::{
  get_content_hash_of_module, get_timestamp_of_module, handle_cached_modules,
  set_module_graph_cache, try_get_module_cache_by_hash, try_get_module_cache_by_timestamp,
};

macro_rules! call_and_catch_error {
  ($func:ident, $($args:expr),+) => {
    match $func($($args),+) {
      Ok(r) => r,
      Err(e) => {
        return Err(e);
      }
    }
  };
}

pub(crate) mod analyze_deps;
pub(crate) mod finalize_module;
pub(crate) mod load;
pub(crate) mod module_cache;
pub(crate) mod parse;
pub(crate) mod resolve;
pub(crate) mod transform;

#[derive(Debug)]
pub(crate) struct ResolveModuleIdResult {
  pub module_id: ModuleId,
  pub resolve_result: PluginResolveHookResult,
}

pub(crate) struct ResolvedModuleInfo {
  pub module: Module,
  pub resolve_module_id_result: ResolveModuleIdResult,
}

pub(crate) struct BuildModuleGraphThreadedParams {
  pub thread_pool: Arc<ThreadPool>,
  pub resolve_param: PluginResolveHookParam,
  pub context: Arc<CompilationContext>,
  pub err_sender: Sender<CompilationError>,
  pub order: usize,
  pub cached_dependency: Option<ModuleId>,
}

pub(crate) struct HandleDependenciesParams {
  pub module: Module,
  pub resolve_param: PluginResolveHookParam,
  pub order: usize,
  pub deps: Vec<(PluginAnalyzeDepsHookResultEntry, Option<ModuleId>)>,
  pub thread_pool: Arc<ThreadPool>,
  pub err_sender: Sender<CompilationError>,
  pub context: Arc<CompilationContext>,
}

enum ResolveModuleResult {
  /// The module is already built
  Built(ModuleId),
  Cached(ModuleId),
  /// A full new normal module resolved successfully
  Success(Box<ResolvedModuleInfo>),
}

impl Compiler {
  fn set_module_graph_stats(&self) {
    if self.context.config.record {
      let module_graph = self.context.module_graph.read();
      self
        .context
        .record_manager
        .set_module_graph_stats(&module_graph);
      self
        .context
        .record_manager
        .set_entries(module_graph.entries.keys().cloned().collect())
    }
  }

  pub fn set_last_fail_module_ids(&self, errors: &[CompilationError]) {
    let mut last_fail_module_ids = self.last_fail_module_ids.lock();
    last_fail_module_ids.clear();

    for id in get_module_ids_from_compilation_errors(errors) {
      if !last_fail_module_ids.contains(&id) {
        last_fail_module_ids.push(id);
      }
    }
  }

  pub(crate) fn build(&self) -> Result<()> {
    self.context.plugin_driver.build_start(&self.context)?;

    let (err_sender, err_receiver) = Self::create_thread_channel();

    for (order, (name, source)) in self.context.config.input.iter().enumerate() {
      let params = BuildModuleGraphThreadedParams {
        thread_pool: self.thread_pool.clone(),
        resolve_param: PluginResolveHookParam {
          source: source.clone(),
          importer: None,
          kind: ResolveKind::Entry(name.clone()),
        },
        context: self.context.clone(),
        err_sender: err_sender.clone(),
        order,
        cached_dependency: None,
      };
      Self::build_module_graph_threaded(params);
    }

    drop(err_sender);

    let mut errors = vec![];

    for err in err_receiver {
      errors.push(err);
    }

    self.handle_global_log(&mut errors);
    let mut res = Ok(());

    if !errors.is_empty() {
      // set stats if stats is enabled
      self.context.record_manager.set_build_end_time();
      self.context.record_manager.set_end_time();
      self.set_module_graph_stats();

      // set last failed module ids
      self.set_last_fail_module_ids(&errors);

      let mut error_messages = vec![];
      for error in errors {
        error_messages.push(error.to_string());
      }
      let errors_json = json!(error_messages
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>());
      res = Err(CompilationError::GenericError(errors_json.to_string()));
    } else {
      self.set_last_fail_module_ids(&[]);
    }

    // set module graph cache
    if self.context.config.persistent_cache.enabled() {
      let module_ids = self
        .context
        .module_graph
        .read()
        .modules()
        .into_iter()
        .map(|m| m.id.clone())
        .collect();
      // set new module cache
      set_module_graph_cache(module_ids, &self.context);
    }

    // Topo sort the module graph
    let mut module_graph = self.context.module_graph.write();
    module_graph.update_execution_order_for_modules();
    drop(module_graph);

    // set stats if stats is enabled
    self.set_module_graph_stats();

    {
      farm_profile_scope!("call build_end hook".to_string());
      self.context.plugin_driver.build_end(&self.context)?;
    }

    res
  }

  pub(crate) fn handle_global_log(&self, errors: &mut Vec<CompilationError>) {
    for err in self.context.log_store.lock().errors() {
      errors.push(CompilationError::GenericError(err.to_string()));
    }

    if !self.context.log_store.lock().warnings().is_empty() {
      for warning in self.context.log_store.lock().warnings() {
        println!("[warn] {}", warning);
      }
    }

    // clear log store
    self.context.log_store.lock().clear();
  }

  pub(crate) fn resolve_module_id(
    resolve_param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<ResolveModuleIdResult> {
    let get_module_id = |resolve_result: &PluginResolveHookResult| {
      // make query part of module id
      ModuleId::new(
        &resolve_result.resolved_path,
        &stringify_query(&resolve_result.query),
        &context.config.root,
      )
    };

    if let Some(result) = context.get_resolve_cache(resolve_param) {
      return Ok(ResolveModuleIdResult {
        module_id: get_module_id(&result),
        resolve_result: result,
      });
    }

    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::new(),
    };
    let resolve_kind = resolve_param.kind.clone();
    let mut resolve_result = match resolve(resolve_param, context, &hook_context) {
      Ok(resolved) => resolved,
      Err(e) => {
        return Err(e);
      }
    };

    if !resolve_result
      .meta
      .contains_key(PLUGIN_BUILD_STAGE_META_RESOLVE_KIND)
    {
      resolve_result.meta.insert(
        PLUGIN_BUILD_STAGE_META_RESOLVE_KIND.to_string(),
        resolve_kind.into(),
      );
    }

    context.set_resolve_cache(resolve_param.clone(), resolve_result.clone());

    let module_id = get_module_id(&resolve_result);

    Ok(ResolveModuleIdResult {
      module_id,
      resolve_result,
    })
  }

  /// Resolving, loading, transforming and parsing a module, return the module and its dependencies if success
  pub(crate) fn build_module(
    resolve_result: PluginResolveHookResult,
    module: &mut Module,
    context: &Arc<CompilationContext>,
  ) -> Result<Vec<(PluginAnalyzeDepsHookResultEntry, Option<ModuleId>)>> {
    // skip timestamp and content hash for modules
    module.last_update_timestamp = if module.immutable {
      0
    } else {
      get_timestamp_of_module(&module.id, &context.config.root)
    };

    if let Some(cached_module) =
      try_get_module_cache_by_timestamp(&module.id, module.last_update_timestamp, context)?
    {
      *module = cached_module.module;
      return Ok(CachedModule::dep_sources(cached_module.dependencies));
    }

    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::new(),
    };

    // ================ Load Start ===============
    let load_param = PluginLoadHookParam {
      resolved_path: &resolve_result.resolved_path,
      query: resolve_result.query.clone(),
      meta: resolve_result.meta.clone(),
      module_id: module.id.to_string(),
    };

    let load_result = call_and_catch_error!(load, &load_param, context, &hook_context);
    let mut source_map_chain = vec![];

    if let Some(source_map) = load_result.source_map {
      source_map_chain.push(Arc::new(source_map));
    }

    // ================ Load End ===============
    // ================ Transform Start ===============
    let load_module_type = load_result.module_type.clone();
    let transform_param = PluginTransformHookParam {
      content: load_result.content,
      resolved_path: &resolve_result.resolved_path,
      module_type: load_module_type.clone(),
      query: resolve_result.query.clone(),
      meta: resolve_result.meta.clone(),
      module_id: module.id.to_string(),
      source_map_chain,
    };

    let transform_result = call_and_catch_error!(transform, transform_param, context);
    // ================ Transform End ===============
    module.content = Arc::new(transform_result.content.clone());
    module.content_hash = if module.immutable {
      "immutable_module".to_string()
    } else {
      get_content_hash_of_module(&transform_result.content)
    };

    // skip building if the module is already built and the cache is enabled
    if let Some(cached_module) =
      try_get_module_cache_by_hash(&module.id, &module.content_hash, context)?
    {
      *module = cached_module.module;
      return Ok(CachedModule::dep_sources(cached_module.dependencies));
    }

    let deps = Self::build_module_after_transform(
      resolve_result,
      load_module_type,
      transform_result,
      module,
      context,
      &hook_context,
    )?;

    Ok(deps.into_iter().map(|dep| (dep, None)).collect())
  }

  fn build_module_after_transform(
    resolve_result: PluginResolveHookResult,
    load_module_type: ModuleType,
    transform_result: PluginDriverTransformHookResult,
    module: &mut Module,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> Result<Vec<PluginAnalyzeDepsHookResultEntry>> {
    // ================ Parse Start ===============
    let parse_param = PluginParseHookParam {
      module_id: module.id.clone(),
      resolved_path: resolve_result.resolved_path.clone(),
      query: resolve_result.query.clone(),
      module_type: transform_result.module_type.unwrap_or(load_module_type),
      content: Arc::new(transform_result.content),
    };

    let mut module_meta = call_and_catch_error!(parse, &parse_param, context, hook_context);
    // ================ Parse End ===============

    // ================ Process Module Start ===============
    if let Err(e) = context.plugin_driver.process_module(
      &mut PluginProcessModuleHookParam {
        module_id: &parse_param.module_id,
        module_type: &parse_param.module_type,
        content: module.content.clone(),
        meta: &mut module_meta,
      },
      context,
    ) {
      return Err(CompilationError::ProcessModuleError {
        resolved_path: resolve_result.resolved_path,
        source: Some(Box::new(e)),
      });
    }

    // ================ Process Module End ===============
    module.size = parse_param.content.as_bytes().len();
    module.module_type = parse_param.module_type;
    module.side_effects = resolve_result.side_effects;
    module.external = false;
    module.source_map_chain = transform_result.source_map_chain;
    module.meta = Box::new(module_meta);

    let resolved_path = module.id.resolved_path(&context.config.root);
    let package_info =
      load_package_json(PathBuf::from(resolved_path), Default::default()).unwrap_or_default();
    module.package_name = package_info.name.unwrap_or("default".to_string());
    module.package_version = package_info.version.unwrap_or("0.0.0".to_string());

    // ================ Analyze Deps Start ===============
    let analyze_deps_result = call_and_catch_error!(analyze_deps, module, context);
    // ================ Analyze Deps End ===============

    // ================ Finalize Module Start ===============
    call_and_catch_error!(finalize_module, module, &analyze_deps_result, context);
    // ================ Finalize Module End ===============

    Ok(analyze_deps_result)
  }

  /// resolving, loading, transforming and parsing a module in a separate thread
  fn build_module_graph_threaded(params: BuildModuleGraphThreadedParams) {
    let BuildModuleGraphThreadedParams {
      thread_pool,
      resolve_param,
      context,
      err_sender,
      order,
      cached_dependency,
    } = params;

    let c_thread_pool = thread_pool.clone();
    thread_pool.spawn(move || {
      farm_profile_function!(format!(
        "build_module_graph_threaded from {:?} -> {:?}, cached: {:?}",
        resolve_param.importer, resolve_param.source, cached_dependency
      ));
      // skip resolve, build and timestamp/hash check for cached immutable modules
      let resolve_module_result = match resolve_module(&resolve_param, cached_dependency, &context)
      {
        Ok(r) => r,
        Err(e) => {
          err_sender.send(e).unwrap();
          return;
        }
      };

      match resolve_module_result {
        ResolveModuleResult::Built(module_id) => {
          farm_profile_scope!(format!("module {:?} already exists", module_id));
          // add edge to the graph
          Self::add_edge(&resolve_param, module_id, order, &context);
        }
        ResolveModuleResult::Cached(module_id) => {
          farm_profile_scope!(format!("cache module {:?}", module_id));
          let mut cached_module = context.cache_manager.module_cache.get_cache(&module_id);

          if let Err(e) = handle_cached_modules(&mut cached_module, &context) {
            err_sender.send(e).unwrap();
            return;
          }

          let params = HandleDependenciesParams {
            module: cached_module.module,
            resolve_param,
            order,
            deps: CachedModule::dep_sources(cached_module.dependencies),
            thread_pool: c_thread_pool,
            err_sender,
            context,
          };

          Self::handle_dependencies(params);
        }
        ResolveModuleResult::Success(box ResolvedModuleInfo {
          mut module,
          resolve_module_id_result,
        }) => {
          farm_profile_scope!(format!("new module {:?}", module.id));
          if resolve_module_id_result.resolve_result.external {
            // insert external module to the graph
            let module_id = module.id.clone();
            Self::add_module(module, &resolve_param.kind, &context);
            Self::add_edge(&resolve_param, module_id, order, &context);
            return;
          }

          match Self::build_module(
            resolve_module_id_result.resolve_result,
            &mut module,
            &context,
          ) {
            Err(e) => {
              err_sender.send(e).unwrap();
            }
            Ok(deps) => {
              let params = HandleDependenciesParams {
                module,
                resolve_param,
                order,
                deps,
                thread_pool: c_thread_pool,
                err_sender,
                context,
              };
              Self::handle_dependencies(params);
            }
          }
        }
      }
    });
  }

  fn handle_dependencies(params: HandleDependenciesParams) {
    let HandleDependenciesParams {
      module,
      resolve_param,
      order,
      deps,
      thread_pool,
      err_sender,
      context,
    } = params;

    let module_id = module.id.clone();
    let immutable = module.immutable;
    // add module to the graph
    Self::add_module(module, &resolve_param.kind, &context);
    // add edge to the graph
    Self::add_edge(&resolve_param, module_id.clone(), order, &context);

    // resolving dependencies recursively in the thread pool
    for (order, (dep, cached_dependency)) in deps.into_iter().enumerate() {
      let params = BuildModuleGraphThreadedParams {
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
      };
      Self::build_module_graph_threaded(params);
    }
  }

  pub(crate) fn add_edge(
    resolve_param: &PluginResolveHookParam,
    module_id: ModuleId,
    order: usize,
    context: &CompilationContext,
  ) {
    let mut module_graph = context.module_graph.write();

    if let Some(importer_id) = &resolve_param.importer {
      module_graph.add_edge_item(
        importer_id,
        &module_id,
        ModuleGraphEdgeDataItem {
          source: resolve_param.source.clone(),
          kind: resolve_param.kind.clone(),
          order,
        },
      ).expect("failed to add edge to the module graph, the endpoint modules of the edge should be in the graph")
    }
  }

  /// add a module to the module graph, if the module already exists, update it
  pub(crate) fn add_module(module: Module, kind: &ResolveKind, context: &CompilationContext) {
    let mut module_graph = context.module_graph.write();

    // mark entry module
    if let ResolveKind::Entry(name) = kind {
      module_graph
        .entries
        .insert(module.id.clone(), name.to_string());
    }

    // check if the module already exists
    if module_graph.has_module(&module.id) {
      module_graph.replace_module(module);
    } else {
      module_graph.add_module(module);
    }
  }

  pub(crate) fn create_thread_channel() -> (Sender<CompilationError>, Receiver<CompilationError>) {
    let (err_sender, err_receiver) = channel::<CompilationError>();

    (err_sender, err_receiver)
  }

  pub(crate) fn create_module(module_id: ModuleId, external: bool, immutable: bool) -> Module {
    let mut module = Module::new(module_id);

    // if the module is external, return a external module
    if external {
      module.external = true;
    }

    if immutable {
      module.immutable = true;
    }

    module
  }

  pub(crate) fn create_module_from_resolve_result(
    resolve_module_id_result: &ResolveModuleIdResult,
    context: &Arc<CompilationContext>,
  ) -> Module {
    let module_id_str = resolve_module_id_result.module_id.to_string();
    Compiler::create_module(
      resolve_module_id_result.module_id.clone(),
      resolve_module_id_result.resolve_result.external,
      // treat all lazy virtual modules as mutable
      !module_id_str.ends_with(DYNAMIC_VIRTUAL_SUFFIX)
        && context
          .config
          .partial_bundling
          .immutable_modules
          .iter()
          .any(|im| im.is_match(&module_id_str)),
    )
  }

  pub(crate) fn insert_dummy_module(module_id: &ModuleId, module_graph: &mut ModuleGraph) {
    // insert a dummy module to the graph to prevent the module from being handled twice
    module_graph.add_module(Compiler::create_module(module_id.clone(), false, false));
  }
}

fn resolve_module(
  resolve_param: &PluginResolveHookParam,
  cached_dependency: Option<ModuleId>,
  context: &Arc<CompilationContext>,
) -> Result<ResolveModuleResult> {
  farm_profile_function!(format!(
    "resolve_module from {:?} -> {:?}, cached: {:?}",
    resolve_param.importer, resolve_param.source, cached_dependency
  ));
  let mut resolve_module_id_result = None;
  let module_id = if let Some(cached_dependency) = &cached_dependency {
    cached_dependency.clone()
  } else {
    resolve_module_id_result = Some(Compiler::resolve_module_id(resolve_param, context)?);
    resolve_module_id_result.as_ref().unwrap().module_id.clone()
  };

  let mut module_graph = context.module_graph.write();

  let res = if module_graph.has_module(&module_id) {
    farm_profile_scope!(format!("module {:?} already exists", module_id));
    // the module has already been handled and it should not be handled twice
    ResolveModuleResult::Built(module_id)
  } else {
    if let Some(cached_dependency) = cached_dependency {
      farm_profile_scope!(format!("cache module {:?} ", module_id));
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
          Compiler::insert_dummy_module(&cached_dependency, &mut module_graph);
          return Ok(ResolveModuleResult::Cached(cached_dependency));
        }
      }
    }

    farm_profile_scope!(format!("new module {:?}", module_id));
    let resolve_module_id_result = if let Some(result) = resolve_module_id_result {
      result
    } else {
      Compiler::resolve_module_id(resolve_param, context)?
    };

    Compiler::insert_dummy_module(&resolve_module_id_result.module_id, &mut module_graph);
    ResolveModuleResult::Success(Box::new(ResolvedModuleInfo {
      module: Compiler::create_module_from_resolve_result(&resolve_module_id_result, context),
      resolve_module_id_result,
    }))
  };

  Ok(res)
}
