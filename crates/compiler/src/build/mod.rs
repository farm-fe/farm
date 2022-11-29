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
  module::{
    module_graph::ModuleGraphEdge, Module, ModuleBasicInfo, ModuleId, ModuleMetaData, ModuleType,
  },
  plugin::{
    PluginHookContext, PluginLoadHookParam, PluginParseHookParam, PluginProcessModuleHookParam,
    PluginResolveHookParam, PluginTransformHookParam, ResolveKind,
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

mod analyze_deps;
mod finalize_module;
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

      macro_rules! call_and_catch_error {
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
      let resolve_result = call_and_catch_error!(resolve, &resolve_param, &context, &hook_context);
      let module_id = ModuleId::new(
        &resolve_result.resolved_path,
        &resolve_result.query,
        &context.config.root,
      );

      // the module has already been handled and it should not be handled twice
      if context
        .cache_manager
        .is_module_handled(&module_id.to_string())
      {
        if &resolve_param.source == "react" {
          println!("cached {:?} {:?}", resolve_param.importer, module_id);
        }
        // dependencies relationship should be preserved
        Self::add_edge(&resolve_param, module_id, order, err_sender, &context);
        return;
      } else {
        // first generate a module
        let module = Module::new(module_id.clone());
        Self::add_module(module, &resolve_param.kind, &context);

        context
          .cache_manager
          .mark_module_handled(&module_id.to_string());
      }

      // if the module is external, return a external module
      if resolve_result.external {
        let mut module = Module::new(module_id.clone());
        module.module_type = ModuleType::Custom("farm_external".to_string());
        module.external = true;

        Self::add_module(module, &resolve_param.kind, &context);
        Self::add_edge(&resolve_param, module_id, order, err_sender, &context);
        return;
      }

      // ================ Resolve End ===============

      // ================ Load Start ===============
      let load_param = PluginLoadHookParam {
        resolved_path: &resolve_result.resolved_path,
        query: resolve_result.query.clone(),
      };

      let load_result = call_and_catch_error!(load, &load_param, &context, &hook_context);
      // ================ Load End ===============

      // ================ Transform Start ===============
      let transform_param = PluginTransformHookParam {
        content: load_result.content,
        resolved_path: &resolve_result.resolved_path,
        module_type: load_result.module_type.clone(),
        query: resolve_result.query.clone(),
      };

      let transform_result = call_and_catch_error!(transform, transform_param, &context);
      // ================ Transform End ===============

      // ================ Parse Start ===============
      let parse_param = PluginParseHookParam {
        module_id: module_id.clone(),
        resolved_path: resolve_result.resolved_path.clone(),
        query: resolve_result.query.clone(),
        module_type: transform_result
          .module_type
          .unwrap_or(load_result.module_type),
        content: transform_result.content,
      };

      let mut module_meta = call_and_catch_error!(parse, &parse_param, &context, &hook_context);
      // ================ Parse End ===============
      println!("parsed {}", resolve_result.resolved_path);

      // ================ Process Module Start ===============
      if let Err(e) = context.plugin_driver.process_module(
        &mut PluginProcessModuleHookParam {
          module_id: &parse_param.module_id,
          module_type: &parse_param.module_type,
          meta: &mut module_meta,
        },
        &context,
      ) {
        err_sender
          .send(CompilationError::ModuleParsedError {
            resolved_path: resolve_result.resolved_path.clone(),
            source: Some(Box::new(e)),
          })
          .unwrap();
        return;
      }
      // ================ Process Module End ===============
      println!("processed module {}", resolve_result.resolved_path);

      let module_info = ModuleBasicInfo {
        side_effects: resolve_result.side_effects,
        source_map_chain: transform_result.source_map_chain,
        external: false,
        module_type: parse_param.module_type.clone(),
      };

      // must update module info before analyze dependencies
      Self::update_module(&module_id, module_info, module_meta, &context);
      Self::add_edge(
        &resolve_param,
        module_id.clone(),
        order,
        err_sender.clone(),
        &context,
      );

      // ================ Analyze Deps Start ===============
      let analyze_deps_result = call_and_catch_error!(analyze_deps, &module_id, &context);
      // ================ Analyze Deps End ===============
      println!(
        "analyzed deps {} -> {:?}",
        resolve_result.resolved_path, analyze_deps_result
      );

      // ================ Finalize Module Start ===============
      call_and_catch_error!(finalize_module, &module_id, &context);
      // ================ Finalize Module End ===============

      // resolving dependencies recursively in the thread pool
      for (order, dep) in analyze_deps_result.into_iter().enumerate() {
        Self::build_module(
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

  fn add_module(module: Module, kind: &ResolveKind, context: &CompilationContext) {
    let mut module_graph = context.module_graph.write();

    // mark entry module
    if matches!(kind, ResolveKind::Entry) {
      module_graph.entries.insert(module.id.clone());
    }

    module_graph.add_module(module);
  }

  fn update_module(
    module_id: &ModuleId,
    module_info: ModuleBasicInfo,
    meta: ModuleMetaData,
    context: &CompilationContext,
  ) {
    let mut module_graph = context.module_graph.write();
    let module = module_graph.module_mut(module_id).unwrap();

    let ModuleBasicInfo {
      side_effects,
      source_map_chain,
      external,
      module_type,
    } = module_info;

    module.module_type = module_type;
    module.side_effects = side_effects;
    module.external = external;
    module.source_map_chain = source_map_chain;
    module.meta = meta;
  }
}
