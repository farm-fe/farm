use std::{
  collections::HashMap,
  sync::{mpsc::Sender, Arc},
};

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  hashbrown::HashSet,
  module::{
    module_graph::ModuleGraphEdge, module_group::ModuleGroupId, Module, ModuleId, ModuleType,
  },
  plugin::{PluginResolveHookParam, ResolveKind},
  rayon::ThreadPool,
  resource::ResourceType,
};
use farmfe_plugin_html::get_dynamic_resources_map;
use farmfe_toolkit::tracing;

use crate::{
  build::ResolvedModuleInfo, generate::finalize_resources::finalize_resources, Compiler,
};
use farmfe_core::error::Result;

use self::{
  diff_and_patch_module_graph::{diff_module_graph, patch_module_graph, DiffResult},
  patch_module_group_graph::patch_module_group_graph,
  regenerate_resources::{
    regenerate_resources_for_affected_module_groups, render_and_generate_update_resource,
  },
  update_context::UpdateContext,
};

mod diff_and_patch_module_graph;
mod find_hmr_boundaries;
mod patch_module_group_graph;
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
  pub boundaries: HashMap<String, Vec<Vec<String>>>,
  pub dynamic_resources_map: Option<HashMap<ModuleId, Vec<(String, ResourceType)>>>,
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

    self.optimize_update_module_graph(&update_context).unwrap();

    let previous_module_groups = {
      let module_group_graph = self.context.module_group_graph.read();
      module_group_graph
        .module_groups()
        .into_iter()
        .map(|m| m.id.clone())
        .collect::<HashSet<_>>()
    };

    let (affected_module_groups, updated_module_ids, diff_result) =
      self.diff_and_patch_context(paths, &update_context);

    let dynamic_resources_map = self.regenerate_resources(
      affected_module_groups,
      previous_module_groups,
      &updated_module_ids,
    );
    // TODO1: only regenerate the resources for script modules.
    // TODO2: should reload when html change
    // TODO3: cover it with tests
    let resources =
      render_and_generate_update_resource(&updated_module_ids, &diff_result, &self.context)?;

    // find the boundaries. TODO: detect the boundaries in the client side.
    let boundaries = find_hmr_boundaries::find_hmr_boundaries(&updated_module_ids, &self.context);

    // TODO: support sourcemap for hmr. and should generate the hmr update response body in rust side.
    Ok(UpdateResult {
      added_module_ids: diff_result.added_modules.into_iter().collect(),
      updated_module_ids,
      removed_module_ids: diff_result.removed_modules.into_iter().collect(),
      resources,
      boundaries,
      dynamic_resources_map,
    })
  }

  fn optimize_update_module_graph(&self, update_context: &Arc<UpdateContext>) -> Result<()> {
    // we should optimize the module graph after the update, as tree shaking are called on this stage and may remove some modules
    let mut update_module_graph = update_context.module_graph.write();

    self
      .context
      .plugin_driver
      .optimize_module_graph(&mut *update_module_graph, &self.context)
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

  #[tracing::instrument(skip_all)]
  fn diff_and_patch_context(
    &self,
    paths: Vec<(String, UpdateType)>,
    update_context: &Arc<UpdateContext>,
  ) -> (HashSet<ModuleId>, Vec<ModuleId>, DiffResult) {
    let start_points: Vec<ModuleId> = paths
      .into_iter()
      // Note: HMR does not support the module with query
      .map(|path| ModuleId::new(&path.0, "", &self.context.config.root))
      .collect();
    let mut module_graph = self.context.module_graph.write();
    let mut update_module_graph = update_context.module_graph.write();

    tracing::trace!("Diffing module graph start from {:?}...", start_points);
    let diff_result =
      diff_module_graph(start_points.clone(), &*module_graph, &*update_module_graph);
    tracing::trace!("Diff result: {:?}", diff_result);

    tracing::trace!("Patching module graph start from {:?}...", start_points);
    let removed_modules = patch_module_graph(
      start_points.clone(),
      &diff_result,
      &mut *module_graph,
      &mut *update_module_graph,
    );
    tracing::trace!(
      "Patched module graph, removed modules: {:?}",
      removed_modules
        .iter()
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>()
    );

    let mut module_group_graph = self.context.module_group_graph.write();

    tracing::trace!("Patching module group map start from {:?}...", start_points);
    let affected_module_groups = patch_module_group_graph(
      start_points.clone(),
      &diff_result,
      &removed_modules,
      &mut *module_graph,
      &mut *module_group_graph,
    );
    tracing::trace!(
      "Patched module group map, affected module groups: {:?}",
      affected_module_groups
    );

    (affected_module_groups, start_points, diff_result)
  }

  fn regenerate_resources(
    &self,
    affected_module_groups: HashSet<ModuleGroupId>,
    previous_module_groups: HashSet<ModuleGroupId>,
    updated_module_ids: &Vec<ModuleId>,
  ) -> Option<HashMap<ModuleId, Vec<(String, ResourceType)>>> {
    let mut dynamic_resources_map = None;
    let cloned_updated_module_ids = updated_module_ids.clone();

    let cloned_context = self.context.clone();

    // if there are new module groups, we should run the tasks synchronously
    if affected_module_groups
      .iter()
      .any(|ag| !previous_module_groups.contains(ag))
    {
      regenerate_resources_for_affected_module_groups(
        affected_module_groups,
        &cloned_updated_module_ids,
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
        .filter(|m| {
          let module = module_graph.module(m).unwrap();
          matches!(module.module_type, ModuleType::Html)
        })
        .collect::<Vec<_>>();
      let mut dynamic_resources = HashMap::new();

      for html_entry_id in html_entries_ids {
        dynamic_resources.extend(get_dynamic_resources_map(
          &*module_group_graph,
          &html_entry_id,
          &*resource_pot_map,
          &*resources_map,
        ));
      }

      dynamic_resources_map = Some(dynamic_resources);
    } else {
      std::thread::spawn(move || {
        // TODO: manage a task queue, and run the tasks in sequence
        regenerate_resources_for_affected_module_groups(
          affected_module_groups,
          &cloned_updated_module_ids,
          &cloned_context,
        )
        .unwrap();

        finalize_resources(&cloned_context).unwrap();
      });
    }

    dynamic_resources_map
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
  let module_id = &resolve_module_id_result.module_id;
  // if this is the root module, we should always rebuild it
  if is_root {
    return Ok(ResolveModuleResult::Success(Box::new(ResolvedModuleInfo {
      module: Compiler::create_module(
        resolve_module_id_result.module_id.clone(),
        resolve_module_id_result.resolve_result.external,
        // TODO: make it configurable
        module_id.to_string().contains("/node_modules/"),
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

  let mut update_module_graph = update_context.module_graph.write();
  if update_module_graph.has_module(&resolve_module_id_result.module_id) {
    return Ok(ResolveModuleResult::ExistingWhenUpdate(
      resolve_module_id_result.module_id,
    ));
  }
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
      module_id.to_string().contains("/node_modules/"),
    ),
    resolve_module_id_result,
  })))
}
