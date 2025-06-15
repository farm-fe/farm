#![feature(box_patterns)]

use farmfe_core::{
  config::Config,
  error::CompilationError,
  module::{
    meta_data::script::{ModuleExportIdent, ModuleExportIdentType},
    ModuleId, ModuleSystem, ModuleType,
  },
  parking_lot::Mutex,
  plugin::{Plugin, PluginAnalyzeDepsHookResultEntry, PluginResolveHookResult, ResolveKind},
  rayon::iter::{IntoParallelRefMutIterator, ParallelIterator},
  relative_path::RelativePath,
  resource::{
    meta_data::{js::JsResourcePotMetaData, ResourcePotMetaData},
    resource_pot::ResourcePotType,
  },
  HashMap, HashSet,
};
use farmfe_toolkit::{
  fs::read_file_utf8,
  runtime::RuntimeFeatureGuardRemover,
  script::{
    concatenate_modules::{concatenate_modules_ast, ConcatenateModulesAstResult, EXPORT_NAMESPACE},
    create_export_namespace_ident, set_module_system_for_module_meta,
    swc_try_with::try_with,
  },
  swc_ecma_visit::VisitMutWith,
};
use transform_cjs::{transform_cjs_to_esm, transform_hybrid_to_cjs};

use crate::transform_cjs::{FARM_MODULE_SYSTEM_MODULE_HELPER, FARM_MODULE_SYSTEM_SOURCE};

mod handle_exports;
mod transform_cjs;

const FARM_RUNTIME_PREFIX: &str = "@farmfe/runtime/";

#[derive(Default)]
pub struct FarmPluginLibrary {
  export_namespace_modules: Mutex<HashSet<ModuleId>>,
  cjs_require_map: Mutex<HashMap<(ModuleId, String), ModuleId>>,
  external_source_map: Mutex<HashMap<ModuleId, HashSet<String>>>,
}

impl FarmPluginLibrary {
  pub fn new(_: &Config) -> Self {
    Self::default()
  }
}

impl Plugin for FarmPluginLibrary {
  fn name(&self) -> &str {
    "FarmPluginLibrary"
  }

  /// Make sure this plugin is executed before all other internal plugins.
  fn priority(&self) -> i32 {
    101
  }

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginResolveHookResult>> {
    if param.source.starts_with(FARM_RUNTIME_PREFIX) {
      return Ok(Some(PluginResolveHookResult {
        resolved_path: param.source.clone(),
        external: false,
        side_effects: false,
        query: Default::default(),
        meta: Default::default(),
      }));
    }

    Ok(None)
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if let Some(rel_path) = param.resolved_path.strip_prefix(FARM_RUNTIME_PREFIX) {
      let abs_path = RelativePath::new(rel_path).to_logical_path(&context.config.runtime.path);
      let content = read_file_utf8(abs_path.to_string_lossy().to_string().as_str())?;

      return Ok(Some(farmfe_core::plugin::PluginLoadHookResult {
        content,
        module_type: ModuleType::Ts,
        source_map: None,
      }));
    }

    Ok(None)
  }

  /// 1. Handle runtime module
  fn finalize_module(
    &self,
    param: &mut farmfe_core::plugin::PluginFinalizeModuleHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if param.module.module_type.is_script() {
      // detect module system first
      set_module_system_for_module_meta(param, context);

      let script_meta_data = param.module.meta.as_script_mut();
      let cm = context.meta.get_module_source_map(&param.module.id);
      let globals = context.meta.get_globals(&param.module.id);

      try_with(cm, globals.value(), || {
        if param.module.id.to_string().starts_with(FARM_RUNTIME_PREFIX) {
          // remove unused runtime features
          let feature_flags = HashSet::default();
          let mut runtime_feature_remover =
            RuntimeFeatureGuardRemover::new(&feature_flags, context);
          script_meta_data
            .ast
            .visit_mut_with(&mut runtime_feature_remover);
        }

        if matches!(
          script_meta_data.module_system,
          ModuleSystem::CommonJs | ModuleSystem::Hybrid
        ) {
          param.deps.push(PluginAnalyzeDepsHookResultEntry {
            source: FARM_MODULE_SYSTEM_SOURCE.to_string(),
            kind: ResolveKind::Import,
          });
          param.deps.push(PluginAnalyzeDepsHookResultEntry {
            source: FARM_MODULE_SYSTEM_MODULE_HELPER.to_string(),
            kind: ResolveKind::Import,
          });
        }
      })?;
    }

    Ok(None)
  }

  // 2. Update ResolveKind to Import for cjs library module
  fn module_graph_build_end(
    &self,
    module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // Note that we update ResolveKind to Import here instead of in finalize_module because
    // when resolving dependencies ResolveKind::Require and ResolveKind::Import are different,
    // if we update ResolveKind::Require to ResolveKind::Import, it will break original dependency resolution
    let mut edges_to_update = vec![];

    for module in module_graph.modules() {
      for (dep_id, edge_info) in module_graph.dependencies(&module.id) {
        if let Some(dep_module) = module_graph.module(&dep_id) {
          if dep_module.external {
            let mut external_source_map = self.external_source_map.lock();
            external_source_map
              .entry(module.id.clone())
              .or_default()
              .extend(edge_info.items().iter().map(|item| item.source.clone()));
          }
        }

        if edge_info.contains_require() {
          edges_to_update.push((module.id.clone(), dep_id));
        }
      }
    }

    for (module_id, dep_id) in edges_to_update {
      if let Some(edge_info) = module_graph.edge_info(&module_id, &dep_id) {
        // find require edge item
        if let Some(dep_module) = module_graph.module(&dep_id) {
          if !dep_module.external && dep_module.module_type.is_script() {
            let meta = dep_module.meta.as_script();

            if meta.module_system == ModuleSystem::CommonJs {
              let require_edge_item = edge_info
                .items()
                .iter()
                .find(|item| item.kind == ResolveKind::Require);

              self.cjs_require_map.lock().insert(
                (module_id.clone(), require_edge_item.unwrap().source.clone()),
                dep_id.clone(),
              );
            }
          }
        }

        let mut edge_info = edge_info.clone();
        edge_info.update_kind(ResolveKind::Import);
        module_graph
          .update_edge(&module_id, &dep_id, edge_info)
          .unwrap();
      }

      self.export_namespace_modules.lock().insert(dep_id);
    }

    Ok(Some(()))
  }

  /// 3. Transform cjs to esm, update export_ident_map and append export decl for cjs module
  fn optimize_module_graph(
    &self,
    module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let mut cjs_require_map = self.cjs_require_map.lock();
    let cjs_require_map: HashMap<(ModuleId, String), ModuleId> =
      cjs_require_map.drain().into_iter().collect();
    let cjs_required_modules: HashSet<&ModuleId> = cjs_require_map.values().collect();

    let external_source_map = self.external_source_map.lock();

    module_graph
      .modules_mut()
      .par_iter_mut()
      .filter(|module| module.module_type.is_script())
      .for_each(|module| {
        let meta = module.meta.as_script_mut();

        let cm = context.meta.get_module_source_map(&module.id);
        let globals = context.meta.get_globals(&module.id);
        let is_required_cjs_module = cjs_required_modules.contains(&module.id);

        try_with(cm, globals.value(), || {
          if meta.module_system == ModuleSystem::Hybrid {
            // TODO
            transform_hybrid_to_cjs();
            meta.module_system = ModuleSystem::CommonJs;
          }

          // transform cjs to esm
          if meta.module_system == ModuleSystem::CommonJs {
            transform_cjs_to_esm(
              &module.id,
              &cjs_require_map,
              &external_source_map,
              meta,
              context,
              is_required_cjs_module,
            );
            meta.module_system = ModuleSystem::EsModule;
          }
        })
        .unwrap();
      });

    let export_namespace_modules = self.export_namespace_modules.lock();

    for dep_id in export_namespace_modules.iter() {
      if let Some(dep_module) = module_graph.module_mut(&dep_id) {
        if dep_module.external || !dep_module.module_type.is_script() {
          continue;
        }

        let dep_module_meta = dep_module.meta.as_script_mut();
        let is_required_cjs_module = cjs_required_modules.contains(&dep_id);

        if !is_required_cjs_module
          && !dep_module_meta
            .export_ident_map
            .contains_key(EXPORT_NAMESPACE)
        {
          dep_module_meta.export_ident_map.insert(
            EXPORT_NAMESPACE.to_string(),
            ModuleExportIdent {
              module_id: dep_id.clone(),
              ident: create_export_namespace_ident(&dep_id).to_id().into(),
              export_type: ModuleExportIdentType::VirtualNamespace,
            },
          );
        }
      }
    }

    Ok(None)
  }

  fn partial_bundling(
    &self,
    _modules: &Vec<farmfe_core::module::ModuleId>,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<farmfe_core::resource::resource_pot::ResourcePot>>> {
    // TODO: disable partial bundling for library bundle and implement normal bundling.
    // The algorithm: Merge all modules in the same module group into one resource pot.
    // Note: farm runtime modules should always be bundled into one single resource pot.

    Ok(None)
  }

  // TODO: add a hook collect resource pot import/export info before render resource pot

  fn render_resource_pot(
    &self,
    resource_pot: &farmfe_core::resource::resource_pot::ResourcePot,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ResourcePotMetaData>> {
    if resource_pot.resource_pot_type != ResourcePotType::Js {
      return Ok(None);
    }

    let module_graph = context.module_graph.read();

    let ConcatenateModulesAstResult {
      ast,
      module_ids,
      external_modules,
      source_map,
      comments,
      globals,
      unresolved_mark,
      top_level_mark,
    } = concatenate_modules_ast(
      resource_pot.entry_module.as_ref().unwrap(), // TODO: support dynamic imported entry module for multiple library bundle
      &resource_pot.modules,
      &module_graph,
      context,
    )
    .map_err(|e| CompilationError::GenericError(e.to_string()))?;

    context
      .meta
      .set_resource_pot_source_map(&resource_pot.id, source_map);
    context
      .meta
      .set_resource_pot_globals(&resource_pot.id, globals);

    // handle import/export between resource pots
    // if let Some(entry) = &resource_pot.entry_module {
    //   let entry_module = module_graph.module(entry).unwrap();
    //   let script_meta = entry_module.meta.as_script();

    //   if !script_meta.export_ident_map.is_empty() {
    //     let export_item = script_meta.get_export_module_item();
    //     ast.body.push(export_item);
    //   }
    // }

    // TODO find exports in this resource pot

    Ok(Some(ResourcePotMetaData::Js(JsResourcePotMetaData {
      ast,
      external_modules: external_modules
        .into_iter()
        .map(|(_, id)| id.to_string())
        .collect(),
      rendered_modules: module_ids,
      comments,
      top_level_mark: top_level_mark.as_u32(),
      unresolved_mark: unresolved_mark.as_u32(),
    })))
  }

  fn generate_resources(
    &self,
    _resource_pot: &mut farmfe_core::resource::resource_pot::ResourcePot,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginGenerateResourcesHookResult>>
  {
    // TODO: render the resource according to ModuleFormat and then call generate_resources hook to
    Ok(None)
  }
}
