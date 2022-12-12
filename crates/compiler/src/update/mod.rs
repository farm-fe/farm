use std::sync::{mpsc::Sender, Arc};

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  module::{module_graph::ModuleGraphEdge, Module, ModuleId},
  plugin::{PluginResolveHookParam, ResolveKind},
  rayon::ThreadPool,
};
use farmfe_utils::relative;

use crate::{build::ResolvedModuleInfo, Compiler};
use farmfe_core::error::Result;

use self::update_context::UpdateContext;

mod diff_module_graph;
mod update_context;

/// The output after the updating process
#[derive(Debug, Default)]
pub struct UpdateOutput {
  pub added_or_updated_module_ids: Vec<ModuleId>,
  pub removed_module_ids: Vec<ModuleId>,
  pub resources: String,
}

pub enum UpdateType {
  // added a new module
  Added,
  // updated a module
  Updated,
  // removed a module
  Removed,
}

enum ResolveModuleResult {
  /// This module is already in previous module graph before the update, and we met it again when resolving dependencies
  ExistingBeforeUpdate(ModuleId),
  /// This module is added during the update, and we met it again when resolving dependencies
  ExistingWhenUpdate(ModuleId),
  Cached(Box<ResolvedModuleInfo>),
  Success(Box<ResolvedModuleInfo>),
}

impl Compiler {
  pub fn update(&self, paths: Vec<(String, UpdateType)>) -> Result<UpdateOutput> {
    let (thread_pool, err_sender, err_receiver) = Self::create_thread_pool();
    let update_context = Arc::new(UpdateContext::new());

    for (path, update_type) in paths {
      match update_type {
        UpdateType::Added => {
          return Err(farmfe_core::error::CompilationError::GenericError(
            "Added is not supported yet".to_string(),
          ));
        }

        UpdateType::Updated => {
          let mut source = relative(&self.context.config.root, &path).to_string();
          // if the source is not a relative path, we need to add a `./` prefix
          if !source.starts_with(".") {
            source = format!("./{}", source);
          }

          let resolve_param = PluginResolveHookParam {
            kind: ResolveKind::HmrUpdate,
            source,
            importer: None,
          };

          Self::update_module_graph_threaded(
            thread_pool.clone(),
            resolve_param,
            self.context.clone(),
            update_context.clone(),
            err_sender.clone(),
            None,
          );
        }
        UpdateType::Removed => {
          return Err(farmfe_core::error::CompilationError::GenericError(
            "Removed is not supported yet".to_string(),
          ));
        }
      }
    }

    drop(err_sender);

    for err in err_receiver {
      return Err(err);
    }

    Ok(UpdateOutput::default())
  }

  /// Resolving, loading, transforming and parsing a module in a separate thread.
  /// This method is similar to the build_module_graph_threaded method in the build/mod.rs file,
  /// the difference is that this method is used for updating the module graph, only handles the updated and added module, and ignores the existing unchanged module,
  /// while the build_module_threaded method is used for building full module graph and every module is handled.
  fn update_module_graph_threaded(
    thread_pool: Arc<ThreadPool>,
    resolve_param: PluginResolveHookParam,
    context: Arc<CompilationContext>,
    update_context: Arc<UpdateContext>,
    err_sender: Sender<CompilationError>,
    order: Option<usize>, // if order is [None], means this is the root update module, it should always be rebuilt
  ) {
    let c_thread_pool = thread_pool.clone();

    thread_pool.spawn(move || {
      let resolve_module_result =
        match resolve_module(&resolve_param, &context, &update_context, order.is_none()) {
          Ok(result) => result,
          Err(e) => {
            err_sender.send(e).unwrap();
            return;
          }
        };

      match resolve_module_result {
        ResolveModuleResult::ExistingBeforeUpdate(module_id) => {
          // insert a placeholder module to the update module graph
          let module = Module::new(module_id.clone());
          Self::add_module_to_update_module_graph(&update_context, module);
          Self::add_edge_to_update_module_graph(&update_context, &resolve_param, &module_id, order);
        }
        ResolveModuleResult::ExistingWhenUpdate(module_id) => {
          Self::add_edge_to_update_module_graph(&update_context, &resolve_param, &module_id, order);
        }
        ResolveModuleResult::Cached(_) => unimplemented!("Cached is not supported yet"),
        ResolveModuleResult::Success(box ResolvedModuleInfo {
          mut module,
          resolve_module_id_result,
        }) => {
          match Self::build_module(
            resolve_module_id_result.resolve_result,
            &mut module,
            &context,
          ) {
            Ok(deps) => {
              let module_id = module.id.clone();
              Self::add_module_to_update_module_graph(&update_context, module);
              Self::add_edge_to_update_module_graph(
                &update_context,
                &resolve_param,
                &module_id,
                order,
              );

              for (order, dep) in deps.into_iter().enumerate() {
                Self::update_module_graph_threaded(
                  c_thread_pool.clone(),
                  PluginResolveHookParam {
                    source: dep.source,
                    importer: Some(module_id.clone()),
                    kind: dep.kind,
                  },
                  context.clone(),
                  update_context.clone(),
                  err_sender.clone(),
                  Some(order),
                );
              }
            }
            Err(e) => {
              err_sender.send(e).unwrap();
              return;
            }
          }
        }
      }
    });
  }

  fn add_module_to_update_module_graph(update_context: &Arc<UpdateContext>, module: Module) {
    let mut update_module_graph = update_context.module_graph.write();

    if update_module_graph.has_module(&module.id) {
      return;
    }

    update_module_graph.add_module(module);
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
        .add_edge(
          &importer,
          module_id,
          ModuleGraphEdge {
            kind: resolve_param.kind.clone(),
            source: resolve_param.source.clone(),
            order,
          },
        )
        .expect("Both the importer and the module should be in the update module graph");
    }
  }
}

/// Similar to [crate::build::resolve_module], but the resolved module may be existing in both context and update_context
fn resolve_module(
  resolve_param: &PluginResolveHookParam,
  context: &Arc<CompilationContext>,
  update_context: &Arc<UpdateContext>,
  is_root: bool,
) -> Result<ResolveModuleResult> {
  let resolve_module_id_result = Compiler::resolve_module_id(resolve_param, context)?;

  // if this is the root module, we should always rebuild it
  if is_root {
    return Ok(ResolveModuleResult::Success(Box::new(ResolvedModuleInfo {
      module: Compiler::create_module(
        resolve_module_id_result.module_id.clone(),
        resolve_module_id_result.resolve_result.external,
      ),
      resolve_module_id_result,
    })));
  }

  let module_graph = context.module_graph.read();
  if module_graph.has_module(&resolve_module_id_result.module_id) {
    return Ok(ResolveModuleResult::ExistingBeforeUpdate(
      resolve_module_id_result.module_id,
    ));
  }

  let update_module_graph = update_context.module_graph.read();
  if update_module_graph.has_module(&resolve_module_id_result.module_id) {
    return Ok(ResolveModuleResult::ExistingWhenUpdate(
      resolve_module_id_result.module_id,
    ));
  }

  Ok(ResolveModuleResult::Success(Box::new(ResolvedModuleInfo {
    module: Compiler::create_module(
      resolve_module_id_result.module_id.clone(),
      resolve_module_id_result.resolve_result.external,
    ),
    resolve_module_id_result,
  })))
}
