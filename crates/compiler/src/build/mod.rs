use std::{
  collections::HashMap,
  sync::{
    mpsc::{channel, Sender},
    Arc,
  },
};

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraphEdge, Module, ModuleId, ModuleType},
  plugin::{
    PluginAnalyzeDepsHookResultEntry, PluginHookContext, PluginLoadHookParam, PluginParseHookParam,
    PluginProcessModuleHookParam, PluginResolveHookParam, PluginTransformHookParam, ResolveKind,
  },
  rayon,
  rayon::ThreadPool,
};

use crate::{
  build::{
    analyze_deps::analyze_deps, finalize_module::finalize_module, load::load, parse::parse,
    resolve::resolve, transform::transform,
  },
  Compiler,
};

pub(crate) mod analyze_deps;
pub(crate) mod finalize_module;
pub(crate) mod load;
pub(crate) mod parse;
pub(crate) mod resolve;
pub(crate) mod transform;

/// Preserved module type for the empty module, custom module type from plugins should not be this
pub(crate) const FARM_EMPTY_MODULE: &str = "farmfe_empty_module";

pub(crate) struct BuildModuleResult {
  /// The built module
  module: Module,
  /// The dependencies of the built module
  deps: Vec<PluginAnalyzeDepsHookResultEntry>,
}

pub(crate) enum BuildModuleStatus {
  /// The module is cached, no need to build
  Cached(ModuleId),
  /// Error occurred during the build process
  Error,
  /// The module is built successfully
  Success(Box<BuildModuleResult>),
}

impl Compiler {
  pub(crate) fn build(&self) -> Result<()> {
    self.context.plugin_driver.build_start(&self.context)?;

    let thread_pool = Arc::new(rayon::ThreadPoolBuilder::new().build().unwrap());
    let (err_sender, err_receiver) = channel::<CompilationError>();

    for (order, source) in self.context.config.input.values().enumerate() {
      Self::build_module_graph_threaded(
        thread_pool.clone(),
        PluginResolveHookParam {
          source: source.clone(),
          importer: None,
          kind: ResolveKind::Entry,
        },
        self.context.clone(),
        err_sender.clone(),
        order,
      );
    }

    drop(err_sender);

    if let Ok(err) = err_receiver.recv() {
      return Err(err);
    }

    self.context.plugin_driver.build_end(&self.context)
  }

  /// Resolving, loading, transforming and parsing a module, return the module and its dependencies if success
  pub(crate) fn build_module(
    resolve_param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    err_sender: Sender<CompilationError>,
  ) -> BuildModuleStatus {
    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::new(),
    };

    macro_rules! call_and_catch_error {
      ($func:ident, $($args:expr),+) => {
        match $func($($args),+) {
          Ok(r) => r,
          Err(e) => {
            err_sender.send(e).expect("send error to main thread failed");
            return BuildModuleStatus::Error;
          }
        }
      };
    }

    // ================ Resolve Start ===============
    let resolve_result = call_and_catch_error!(resolve, resolve_param, context, &hook_context);
    let module_id = ModuleId::new(
      &resolve_result.resolved_path,
      &resolve_result.query,
      &context.config.root,
    );

    let mut module = if context
      .cache_manager
      .is_module_handled(&module_id.to_string())
    {
      // the module has already been handled and it should not be handled twice
      return BuildModuleStatus::Cached(module_id);
    } else {
      // first generate a module
      let module = Module::new(module_id.clone());

      context
        .cache_manager
        .mark_module_handled(&module_id.to_string());

      module
    };

    // if the module is external, return a external module
    if resolve_result.external {
      // let mut module = Module::new(module_id.clone());
      module.module_type = ModuleType::Custom("farm_external".to_string());
      module.external = true;

      return BuildModuleStatus::Success(Box::new(BuildModuleResult {
        module,
        deps: vec![],
      }));
    }

    // ================ Resolve End ===============

    // ================ Load Start ===============
    let load_param = PluginLoadHookParam {
      resolved_path: &resolve_result.resolved_path,
      query: resolve_result.query.clone(),
    };

    let load_result = call_and_catch_error!(load, &load_param, context, &hook_context);
    // ================ Load End ===============

    // ================ Transform Start ===============
    let transform_param = PluginTransformHookParam {
      content: load_result.content,
      resolved_path: &resolve_result.resolved_path,
      module_type: load_result.module_type.clone(),
      query: resolve_result.query.clone(),
    };

    let transform_result = call_and_catch_error!(transform, transform_param, context);
    // ================ Transform End ===============

    // ================ Parse Start ===============
    let parse_param = PluginParseHookParam {
      module_id,
      resolved_path: resolve_result.resolved_path.clone(),
      query: resolve_result.query.clone(),
      module_type: transform_result
        .module_type
        .unwrap_or(load_result.module_type),
      content: transform_result.content,
    };

    let mut module_meta = call_and_catch_error!(parse, &parse_param, context, &hook_context);
    // ================ Parse End ===============
    println!("parsed {}", resolve_result.resolved_path);

    // ================ Process Module Start ===============
    if let Err(e) = context.plugin_driver.process_module(
      &mut PluginProcessModuleHookParam {
        module_id: &parse_param.module_id,
        module_type: &parse_param.module_type,
        meta: &mut module_meta,
      },
      context,
    ) {
      err_sender
        .send(CompilationError::ModuleParsedError {
          resolved_path: resolve_result.resolved_path.clone(),
          source: Some(Box::new(e)),
        })
        .unwrap();
      return BuildModuleStatus::Error;
    }
    // ================ Process Module End ===============
    println!("processed module {}", resolve_result.resolved_path);

    module.module_type = parse_param.module_type.clone();
    module.side_effects = resolve_result.side_effects;
    module.external = false;
    module.source_map_chain = transform_result.source_map_chain;
    module.meta = module_meta;

    // ================ Analyze Deps Start ===============
    let analyze_deps_result = call_and_catch_error!(analyze_deps, &module, context);
    // ================ Analyze Deps End ===============
    println!(
      "analyzed deps {} -> {:?}",
      resolve_result.resolved_path, analyze_deps_result
    );

    // ================ Finalize Module Start ===============
    call_and_catch_error!(finalize_module, &mut module, &analyze_deps_result, context);
    // ================ Finalize Module End ===============

    BuildModuleStatus::Success(Box::new(BuildModuleResult {
      module,
      deps: analyze_deps_result,
    }))
  }

  /// resolving, loading, transforming and parsing a module in a separate thread
  fn build_module_graph_threaded(
    thread_pool: Arc<ThreadPool>,
    resolve_param: PluginResolveHookParam,
    context: Arc<CompilationContext>,
    err_sender: Sender<CompilationError>,
    order: usize,
  ) {
    let c_thread_pool = thread_pool.clone();
    thread_pool.spawn(move || {
      match Self::build_module(&resolve_param, &context, err_sender.clone()) {
        BuildModuleStatus::Cached(module_id) => {
          // try add an empty module to the graph, if a module which has the same module_id is already in the graph, this empty module will be ignored
          let mut empty_module = Module::new(module_id.clone());
          empty_module.module_type = ModuleType::Custom(FARM_EMPTY_MODULE.to_string());
          Self::add_or_update_module(empty_module, &resolve_param.kind, &context);
          // dependencies relationship should be preserved
          Self::add_edge(&resolve_param, module_id, order, err_sender, &context);
        }
        BuildModuleStatus::Error => {
          // error is already sent to the main thread, just do nothing
        }
        BuildModuleStatus::Success(box BuildModuleResult { module, deps }) => {
          let module_id = module.id.clone();
          // add module to the graph
          Self::add_or_update_module(module, &resolve_param.kind, &context);
          // add edge to the graph
          Self::add_edge(
            &resolve_param,
            module_id.clone(),
            order,
            err_sender.clone(),
            &context,
          );
          // resolving dependencies recursively in the thread pool
          for (order, dep) in deps.into_iter().enumerate() {
            Self::build_module_graph_threaded(
              c_thread_pool.clone(),
              PluginResolveHookParam {
                source: dep.source,
                importer: Some(module_id.clone()),
                kind: dep.kind,
              },
              context.clone(),
              err_sender.clone(),
              order,
            );
          }
        }
      }
    });
  }

  fn add_edge(
    resolve_param: &PluginResolveHookParam,
    module_id: ModuleId,
    order: usize,
    err_sender: Sender<CompilationError>,
    context: &CompilationContext,
  ) {
    let mut module_graph = context.module_graph.write();

    if let Some(importer_id) = &resolve_param.importer {
      if let Err(e) = module_graph.add_edge(
        importer_id,
        &module_id,
        ModuleGraphEdge {
          source: resolve_param.source.clone(),
          kind: resolve_param.kind.clone(),
          order,
        },
      ) {
        err_sender.send(e).expect("send error failed!");
      };
    }
  }

  /// add a module to the module graph, if the module already exists, update it
  fn add_or_update_module(module: Module, kind: &ResolveKind, context: &CompilationContext) {
    let mut module_graph = context.module_graph.write();

    // mark entry module
    if matches!(kind, ResolveKind::Entry) {
      module_graph.entries.insert(module.id.clone());
    }

    if module_graph.has_module(&module.id) {
      // if current module is a empty module, we should not update it
      if module.module_type == ModuleType::Custom(FARM_EMPTY_MODULE.to_string()) {
        return;
      }

      module_graph.update_module(module);
    } else {
      module_graph.add_module(module);
    }
  }
}
