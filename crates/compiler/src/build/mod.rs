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
  module::{module_graph::ModuleGraphEdge, ModuleId},
  plugin::{
    PluginAnalyzeDepsHookParam, PluginHookContext, PluginLoadHookParam, PluginParseHookParam,
    PluginResolveHookParam, PluginTransformHookParam, ResolveKind,
  },
  rayon,
  rayon::ThreadPool,
};
use farmfe_toolkit::npm_package::load_package_json_or_default;

use crate::{
  build::{load::load, parse::parse, resolve::resolve, transform::transform},
  Compiler,
};

mod analyze_deps;
mod load;
mod parse;
mod resolve;
mod transform;

impl Compiler {
  pub(crate) fn build(&self) -> Result<()> {
    self.context.plugin_driver.build_start(&self.context)?;

    let thread_pool = Arc::new(rayon::ThreadPoolBuilder::new().build().unwrap());
    let (err_sender, err_receiver) = channel::<CompilationError>();

    for (order, source) in self.context.config.input.values().enumerate() {
      Self::build_module(
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

  /// resolving, loading, transforming and parsing a module in a separate thread
  fn build_module(
    thread_pool: Arc<ThreadPool>,
    resolve_param: PluginResolveHookParam,
    context: Arc<CompilationContext>,
    err_sender: Sender<CompilationError>,
    order: usize,
  ) {
    let c_thread_pool = thread_pool.clone();
    thread_pool.spawn(move || {
      let hook_context = PluginHookContext {
        caller: None,
        meta: HashMap::new(),
      };

      macro_rules! call_and_cache_error {
        ($func:ident, $($args:expr),+) => {
          match $func($($args),+) {
            Ok(r) => r,
            Err(e) => {
              err_sender.send(e).expect("send error to main thread failed");
              return;
            }
          }
        };
      }

      // ================ Resolve Start ===============
      let resolve_result = call_and_cache_error!(resolve, &resolve_param, &context, &hook_context);
      // the module has already been handled and it should not be handled twice
      if context.cache_manager.is_module_handled(&resolve_result.id) {
        return;
      } else {
        context
          .cache_manager
          .mark_module_handled(&resolve_result.id);
      }
      // ================ Resolve End ===============

      // ================ Load Start ===============
      let load_param = PluginLoadHookParam {
        id: &resolve_result.id,
        query: resolve_result.query.clone(),
      };

      let load_result = call_and_cache_error!(load, &load_param, &context, &hook_context);
      // ================ Load End ===============

      // ================ Transform Start ===============
      let transform_param = PluginTransformHookParam {
        content: load_result.content,
        id: &resolve_result.id,
        module_type: load_result.module_type.clone(),
        query: resolve_result.query.clone(),
      };

      let transform_result = call_and_cache_error!(transform, transform_param, &context);
      // ================ Transform End ===============

      // ================ Parse Start ===============
      let parse_param = PluginParseHookParam {
        id: resolve_result.id.clone(),
        query: resolve_result.query,
        package_json_info: resolve_result
          .package_json_info
          .unwrap_or(load_package_json_or_default(&context.config.root)),
        side_effects: resolve_result.side_effects,
        module_type: transform_result
          .module_type
          .unwrap_or(load_result.module_type),
        content: transform_result.content,
        source_map_chain: transform_result.source_map_chain,
      };
      let mut module = call_and_cache_error!(parse, parse_param, &context, &hook_context);
      // ================ Parse End ===============

      // ================ Process Module Start ===============
      if let Err(e) = context.plugin_driver.process_module(&mut module, &context) {
        err_sender
          .send(CompilationError::ModuleParsedError {
            id: resolve_result.id.clone(),
            source: Some(Box::new(e)),
          })
          .unwrap();
        return;
      }
      // ================ Process Module End ===============

      // ================ Analyze Deps Start ===============
      let mut analyze_deps_param = PluginAnalyzeDepsHookParam {
        module: &module,
        deps: vec![],
      };
      if let Err(e) = context
        .plugin_driver
        .analyze_deps(&mut analyze_deps_param, &context)
      {
        err_sender
          .send(CompilationError::AnalyzeDepsError {
            id: resolve_result.id.clone(),
            source: Some(Box::new(e)),
          })
          .unwrap();
        return;
      }
      let analyze_deps_result = analyze_deps_param.deps;
      // ================ Analyze Deps End ===============

      let module_id = module.id.clone();
      let mut module_graph = context.module_graph.write();
      module_graph.add_module(module);

      // mark entry module
      if matches!(resolve_param.kind, ResolveKind::Entry) {
        module_graph.entries.push(module_id.clone());
      }

      if let Some(importer) = &resolve_param.importer {
        let importer_id = ModuleId::new(importer, &context.config.root);

        if let Err(e) = module_graph.add_edge(
          &importer_id,
          &module_id,
          ModuleGraphEdge {
            source: resolve_param.source.clone(),
            kind: resolve_param.kind.clone(),
            order,
          },
        ) {
          err_sender.send(e).expect("send error failed!");
          return;
        };
      }
      drop(module_graph);

      // resolving dependencies recursively in the thread pool
      for (order, dep) in analyze_deps_result.into_iter().enumerate() {
        Self::build_module(
          c_thread_pool.clone(),
          PluginResolveHookParam {
            source: dep.source,
            importer: Some(resolve_result.id.clone()),
            kind: dep.kind,
          },
          context.clone(),
          err_sender.clone(),
          order,
        );
      }
    });
  }
}
