use std::{
  collections::HashMap,
  sync::{
    mpsc::{channel, Receiver, Sender},
    Arc,
  },
};

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraphEdge, Module, ModuleId},
  plugin::{
    PluginAnalyzeDepsHookResultEntry, PluginHookContext, PluginLoadHookParam, PluginParseHookParam,
    PluginProcessModuleHookParam, PluginResolveHookParam, PluginResolveHookResult,
    PluginTransformHookParam, ResolveKind,
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

pub(crate) struct ResolveModuleIdResult {
  pub module_id: ModuleId,
  pub resolve_result: PluginResolveHookResult,
}

pub(crate) struct ResolvedModuleInfo {
  pub module: Module,
  pub resolve_module_id_result: ResolveModuleIdResult,
}

enum ResolveModuleResult {
  /// The module is already built
  Built(ModuleId),
  /// The module is cached
  Cached(Box<ResolvedModuleInfo>),
  /// A full new normal module resolved successfully
  Success(Box<ResolvedModuleInfo>),
}

impl Compiler {
  pub(crate) fn build(&self) -> Result<()> {
    self.context.plugin_driver.build_start(&self.context)?;

    let (thread_pool, err_sender, err_receiver) = Self::create_thread_pool();

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

  pub(crate) fn resolve_module_id(
    resolve_param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<ResolveModuleIdResult> {
    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::new(),
    };

    let resolve_result = match resolve(resolve_param, context, &hook_context) {
      Ok(resolved) => resolved,
      Err(e) => {
        return Err(e);
      }
    };

    let module_id = ModuleId::new(&resolve_result.resolved_path, &context.config.root);

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
  ) -> Result<Vec<PluginAnalyzeDepsHookResultEntry>> {
    let hook_context = PluginHookContext {
      caller: None,
      meta: HashMap::new(),
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
      module_id: module.id.clone(),
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
      return Err(CompilationError::ProcessModuleError {
        resolved_path: resolve_result.resolved_path.clone(),
        source: Some(Box::new(e)),
      });
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
    call_and_catch_error!(finalize_module, module, &analyze_deps_result, context);
    // ================ Finalize Module End ===============

    Ok(analyze_deps_result)
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
      let resolve_module_result = match resolve_module(&resolve_param, &context) {
        Ok(r) => r,
        Err(e) => {
          err_sender.send(e).unwrap();
          return;
        }
      };

      match resolve_module_result {
        ResolveModuleResult::Built(module_id) => {
          // add edge to the graph
          Self::add_edge(&resolve_param, module_id, order, &context);
        }
        ResolveModuleResult::Cached(_) => unimplemented!("module cache is not implemented yet"),
        ResolveModuleResult::Success(box ResolvedModuleInfo {
          mut module,
          resolve_module_id_result,
        }) => {
          if resolve_module_id_result.resolve_result.external {
            // skip external modules
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
              let module_id = module.id.clone();
              // add module to the graph
              let status = Self::add_module(module, &resolve_param.kind, &context);
              // add edge to the graph
              Self::add_edge(&resolve_param, module_id.clone(), order, &context);
              // if status is false, means the module is handled and is already in the graph, no need to resolve its dependencies again
              if !status {
                return;
              }

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
        }
      }
    });
  }

  pub(crate) fn add_edge(
    resolve_param: &PluginResolveHookParam,
    module_id: ModuleId,
    order: usize,
    context: &CompilationContext,
  ) {
    let mut module_graph = context.module_graph.write();

    // TODO check if the edge already exists
    if let Some(importer_id) = &resolve_param.importer {
      module_graph.add_edge(
        importer_id,
        &module_id,
        ModuleGraphEdge {
          source: resolve_param.source.clone(),
          kind: resolve_param.kind.clone(),
          order,
        },
      ).expect("failed to add edge to the module graph, the endpoint modules of the edge should be in the graph")
    }
  }

  /// add a module to the module graph, if the module already exists, update it
  /// if the module is already in the graph, return false
  pub(crate) fn add_module(
    module: Module,
    kind: &ResolveKind,
    context: &CompilationContext,
  ) -> bool {
    let mut module_graph = context.module_graph.write();

    // check if the module already exists
    if module_graph.has_module(&module.id) {
      return false;
    }

    // mark entry module
    if matches!(kind, ResolveKind::Entry) {
      module_graph.entries.insert(module.id.clone());
    }

    module_graph.add_module(module);
    true
  }

  pub(crate) fn create_thread_pool() -> (
    Arc<ThreadPool>,
    Sender<CompilationError>,
    Receiver<CompilationError>,
  ) {
    let thread_pool = Arc::new(rayon::ThreadPoolBuilder::new().build().unwrap());
    let (err_sender, err_receiver) = channel::<CompilationError>();

    (thread_pool, err_sender, err_receiver)
  }

  pub(crate) fn create_module(module_id: ModuleId, external: bool) -> Module {
    let mut module = Module::new(module_id);

    // if the module is external, return a external module
    if external {
      module.external = true;
    }

    module
  }
}

fn resolve_module(
  resolve_param: &PluginResolveHookParam,
  context: &Arc<CompilationContext>,
) -> Result<ResolveModuleResult> {
  let resolve_module_id_result = Compiler::resolve_module_id(resolve_param, context)?;

  // be care of dead lock, see https://github.com/Amanieu/parking_lot/issues/212
  let module_graph = context.module_graph.read();

  Ok(
    if module_graph.has_module(&resolve_module_id_result.module_id) {
      // the module has already been handled and it should not be handled twice
      ResolveModuleResult::Built(resolve_module_id_result.module_id.clone())
    } else {
      ResolveModuleResult::Success(Box::new(ResolvedModuleInfo {
        module: Compiler::create_module(
          resolve_module_id_result.module_id.clone(),
          resolve_module_id_result.resolve_result.external,
        ),
        resolve_module_id_result,
      }))
    },
  )
}
