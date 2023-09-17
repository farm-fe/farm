use std::{
  collections::HashMap,
  path::Path,
  sync::{
    mpsc::{channel, Receiver, Sender},
    Arc,
  },
};

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraphEdgeDataItem, Module, ModuleId},
  plugin::{
    constants::PLUGIN_BUILD_STAGE_META_RESOLVE_KIND, PluginAnalyzeDepsHookResultEntry,
    PluginHookContext, PluginLoadHookParam, PluginParseHookParam, PluginProcessModuleHookParam,
    PluginResolveHookParam, PluginResolveHookResult, PluginTransformHookParam, ResolveKind,
  },
  rayon,
  rayon::ThreadPool,
  relative_path::RelativePath,
};

use farmfe_toolkit::hash::base64_decode;
use farmfe_utils::stringify_query;

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
pub mod validate_config;

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
  /// A full new normal module resolved successfully
  Success(Box<ResolvedModuleInfo>),
}

impl Compiler {
  pub(crate) fn build(&self) -> Result<()> {
    self.context.plugin_driver.build_start(&self.context)?;
    validate_config::validate_config(&self.context.config);

    let (thread_pool, err_sender, err_receiver) = Self::create_thread_pool();

    for (order, (name, source)) in self.context.config.input.iter().enumerate() {
      Self::build_module_graph_threaded(
        thread_pool.clone(),
        PluginResolveHookParam {
          source: source.clone(),
          importer: None,
          kind: ResolveKind::Entry(name.clone()),
        },
        self.context.clone(),
        err_sender.clone(),
        order,
      );
    }

    drop(err_sender);

    let mut errors = vec![];

    for err in err_receiver {
      errors.push(err);
    }

    for err in self.context.log_store.read().errors() {
      errors.push(CompilationError::GenericError(err.to_string()));
    }

    if !self.context.log_store.read().warnings().is_empty() {
      for warning in self.context.log_store.read().warnings() {
        println!("[warn] {}", warning);
      }
    }

    if !errors.is_empty() {
      let mut error_messages = vec![];
      for error in errors {
        error_messages.push(error.to_string());
      }
      let error_message = format!(
        "\n Build failed due to errors: \n\n {}",
        error_messages.join("\n")
      );
      println!("{}", error_message);
      // TODO Temporarily exit the process with exit
      std::process::exit(1);
      // return Err(CompilationError::GenericError(error_messages.join(", ")));
    }

    // Topo sort the module graph
    let mut module_graph = self.context.module_graph.write();
    module_graph.update_execution_order_for_modules();

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

    // make query part of module id
    let module_id = ModuleId::new(
      &resolve_result.resolved_path,
      &stringify_query(&resolve_result.query),
      &context.config.root,
    );

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
      meta: resolve_result.meta.clone(),
    };

    let load_result = call_and_catch_error!(load, &load_param, context, &hook_context);

    // try load source map after load module content.
    if load_result.content.contains("//# sourceMappingURL") {
      // detect that the source map is inline or not
      let source_map = if load_result
        .content
        .contains("//# sourceMappingURL=data:application/json;base64,")
      {
        // inline source map
        let mut source_map = load_result
          .content
          .split("//# sourceMappingURL=data:application/json;base64,");

        source_map
          .nth(1)
          .map(|source_map| base64_decode(source_map.as_bytes()))
      } else {
        // external source map
        let mut source_map_path = load_result.content.split("//# sourceMappingURL=");
        let source_map_path = source_map_path.nth(1).unwrap().to_string();
        let resolved_path = Path::new(&load_param.resolved_path);
        let base_dir = resolved_path.parent().unwrap();
        let source_map_path = RelativePath::new(source_map_path.trim()).to_logical_path(base_dir);

        if source_map_path.exists() {
          let source_map = std::fs::read_to_string(source_map_path).unwrap();
          Some(source_map)
        } else {
          None
        }
      };

      if let Some(source_map) = source_map {
        module.source_map_chain.push(source_map);
      }
    }
    // ================ Load End ===============

    // ================ Transform Start ===============
    let transform_param = PluginTransformHookParam {
      content: load_result.content,
      resolved_path: &resolve_result.resolved_path,
      module_type: load_result.module_type.clone(),
      query: resolve_result.query.clone(),
      meta: resolve_result.meta.clone(),
    };

    let mut transform_result = call_and_catch_error!(transform, transform_param, context);
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
    module.size = parse_param.content.as_bytes().len();
    module.module_type = parse_param.module_type;
    module.side_effects = resolve_result.side_effects;
    module.external = false;
    module
      .source_map_chain
      .append(&mut transform_result.source_map_chain);
    module.meta = module_meta;

    // ================ Analyze Deps Start ===============
    let analyze_deps_result = call_and_catch_error!(analyze_deps, module, context);
    // ================ Analyze Deps End ===============

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
        ResolveModuleResult::Success(box ResolvedModuleInfo {
          mut module,
          resolve_module_id_result,
        }) => {
          if resolve_module_id_result.resolve_result.external {
            // insert external module to the graph
            let module_id: ModuleId = resolve_param.source.as_str().into();
            let mut module = Module::new(module_id.clone());
            module.external = true;

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
  /// if the module is already in the graph, return false
  pub(crate) fn add_module(
    module: Module,
    kind: &ResolveKind,
    context: &CompilationContext,
  ) -> bool {
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
}

fn resolve_module(
  resolve_param: &PluginResolveHookParam,
  context: &Arc<CompilationContext>,
) -> Result<ResolveModuleResult> {
  let resolve_module_id_result = Compiler::resolve_module_id(resolve_param, context)?;
  let mut module_graph = context.module_graph.write();
  let module_id = if resolve_module_id_result.resolve_result.external {
    resolve_param.source.as_str().into()
  } else {
    resolve_module_id_result.module_id.clone()
  };

  let res = if module_graph.has_module(&module_id) {
    // the module has already been handled and it should not be handled twice
    ResolveModuleResult::Built(resolve_module_id_result.module_id)
  } else {
    // insert a dummy module to the graph to prevent the module from being handled twice
    module_graph.add_module(Compiler::create_module(
      resolve_module_id_result.module_id.clone(),
      false,
      false,
    ));
    ResolveModuleResult::Success(Box::new(ResolvedModuleInfo {
      module: Compiler::create_module(
        resolve_module_id_result.module_id.clone(),
        resolve_module_id_result.resolve_result.external,
        context
          .config
          .partial_bundling
          .immutable_modules
          .iter()
          .any(|im| im.is_match(&resolve_module_id_result.module_id.to_string())),
      ),
      resolve_module_id_result,
    }))
  };

  Ok(res)
}
