use std::sync::{mpsc::Sender, Arc};

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  hashbrown::HashMap,
  module::{module_graph::ModuleGraphEdge, Module, ModuleId},
  plugin::{PluginResolveHookParam, ResolveKind},
  rayon::ThreadPool,
};

use crate::{build::ResolvedModuleInfo, Compiler};
use farmfe_core::error::Result;

use self::{
  diff_and_patch_module_graph::{diff_module_graph, patch_module_graph, DiffResult},
  patch_module_group_map::patch_module_group_map,
  regenerate_resources::regenerate_resources,
  update_context::UpdateContext,
};

mod diff_and_patch_module_graph;
mod patch_module_group_map;
mod regenerate_resources;
mod update_context;

/// The output after the updating process
#[derive(Debug, Default)]
pub struct UpdateResult {
  pub added_module_ids: Vec<ModuleId>,
  pub updated_module_ids: Vec<ModuleId>,
  pub removed_module_ids: Vec<ModuleId>,
  /// Javascript module map string, the key is the module id, the value is the module function
  /// This code string should be returned to the client side as MIME type `application/javascript`
  pub resources: String,
}

#[derive(Debug, Clone)]
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
  /// Resolve Cache hit
  Cached(Box<ResolvedModuleInfo>),
  /// This module is a full new resolved module, and we need to do the full building process
  Success(Box<ResolvedModuleInfo>),
}

impl Compiler {
  pub fn update(&self, paths: Vec<(String, UpdateType)>) -> Result<UpdateResult> {
    let (thread_pool, err_sender, err_receiver) = Self::create_thread_pool();
    let update_context = Arc::new(UpdateContext::new());

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

    if let Ok(err) = err_receiver.recv() {
      return Err(err);
    }

    // we should optimize the module graph after the update, as tree shaking are called on this stage and may remove some modules
    {
      let mut update_module_graph = update_context.module_graph.write();

      self
        .context
        .plugin_driver
        .optimize_module_graph(&mut *update_module_graph, &self.context)?;
    }

    let (updated_module_ids, diff_result, removed_modules) =
      self.diff_and_patch_context(paths, &update_context);

    let resources =
      self.regenerate_resources(updated_module_ids.clone(), removed_modules, &diff_result)?;

    Ok(UpdateResult {
      added_module_ids: diff_result.added_modules.into_iter().collect(),
      updated_module_ids,
      removed_module_ids: diff_result.removed_modules.into_iter().collect(),
      resources, // TODO regenerate resources
    })
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

  fn diff_and_patch_context(
    &self,
    paths: Vec<(String, UpdateType)>,
    update_context: &Arc<UpdateContext>,
  ) -> (Vec<ModuleId>, DiffResult, HashMap<ModuleId, Module>) {
    let start_points: Vec<ModuleId> = paths
      .into_iter()
      .map(|path| ModuleId::new(&path.0, &self.context.config.root))
      .collect();
    let mut module_graph = self.context.module_graph.write();
    let mut update_module_graph = update_context.module_graph.write();
    let diff_result =
      diff_module_graph(start_points.clone(), &*module_graph, &*update_module_graph);

    let removed_modules = patch_module_graph(
      start_points.clone(),
      &diff_result,
      &mut *module_graph,
      &mut *update_module_graph,
    );

    let mut module_group_map = self.context.module_group_map.write();

    patch_module_group_map(
      start_points.clone(),
      &diff_result,
      &removed_modules,
      &mut *module_graph,
      &mut *module_group_map,
    );

    (start_points, diff_result, removed_modules)
  }

  fn regenerate_resources(
    &self,
    updated_module_ids: Vec<ModuleId>,
    removed_modules: HashMap<ModuleId, Module>,
    diff_result: &DiffResult,
  ) -> Result<String> {
    let mut module_graph = self.context.module_graph.write();
    let mut module_group_map = self.context.module_group_map.write();

    regenerate_resources(
      updated_module_ids,
      removed_modules,
      diff_result,
      &mut module_graph,
      &mut module_group_map,
    )
  }
}

/// Similar to [crate::build::resolve_module], but the resolved module may be existed in both context and update_context
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
