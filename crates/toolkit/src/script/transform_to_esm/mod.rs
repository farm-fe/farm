use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{
    meta_data::script::{
      ModuleExportIdent, ModuleExportIdentType, EXPORT_NAMESPACE, FARM_RUNTIME_MODULE_HELPER_ID,
      FARM_RUNTIME_MODULE_SYSTEM_ID,
    },
    module_graph::ModuleGraph,
    Module, ModuleId, ModuleMetaData, ModuleSystem,
  },
  plugin::ResolveKind,
  HashMap, HashSet,
};

use crate::script::{
  create_export_namespace_ident,
  swc_try_with::try_with,
  transform_to_esm::{
    transform_cjs::{replace_cjs_require, transform_cjs_to_esm, ReplaceCjsRequireResult},
    transform_hybrid::transform_hybrid_to_cjs,
  },
};

pub mod handle_exports;
pub mod transform_cjs;
pub mod transform_hybrid;

pub fn transform_module_to_esm(
  module: &mut Module,
  cjs_require_map: &HashMap<(ModuleId, String), (ModuleId, ModuleSystem)>,
  cjs_required_only_modules: &HashSet<&ModuleId>,
  context: &Arc<CompilationContext>,
) -> (HashSet<&'static str>, bool) {
  let meta = module.meta.as_script_mut();

  let cm = context.meta.get_module_source_map(&module.id);
  let globals = context.meta.get_globals(&module.id);
  let is_required_cjs_module = cjs_required_only_modules.contains(&module.id);
  let mut used_helper_idents = HashSet::default();
  let mut should_add_farm_node_require_res = false;

  try_with(cm, globals.value(), || {
    // transform hybrid to cjs
    if meta.module_system == ModuleSystem::Hybrid {
      used_helper_idents.extend(transform_hybrid_to_cjs(meta));
      meta.module_system = ModuleSystem::CommonJs;
    }

    // replace cjs require
    let ReplaceCjsRequireResult {
      cjs_require_items,
      should_add_farm_node_require,
    } = replace_cjs_require(&module.id, &cjs_require_map, meta);
    should_add_farm_node_require_res = should_add_farm_node_require;

    // transform cjs to esm
    if meta.module_system == ModuleSystem::CommonJs {
      transform_cjs_to_esm(
        &module.id,
        cjs_require_items,
        meta,
        context,
        module.is_entry,
        is_required_cjs_module,
        &mut used_helper_idents,
      );
      meta.module_system = ModuleSystem::EsModule;
    }
  })
  .unwrap();

  (used_helper_idents, should_add_farm_node_require_res)
}

pub fn update_export_namespace_ident(
  export_namespace_modules: &HashSet<ModuleId>,
  cjs_required_only_modules: &HashSet<&ModuleId>,
  module_graph: &mut ModuleGraph,
) {
  for dep_id in export_namespace_modules.iter() {
    if let Some(dep_module) = module_graph.module_mut(&dep_id) {
      if dep_module.external || !dep_module.module_type.is_script() {
        continue;
      }

      let dep_module_meta = dep_module.meta.as_script_mut();
      let is_required_cjs_module = cjs_required_only_modules.contains(&dep_id);

      if !is_required_cjs_module
        && !dep_module_meta
          .export_ident_map
          .contains_key(EXPORT_NAMESPACE)
      {
        dep_module_meta.export_ident_map.insert(
          EXPORT_NAMESPACE.to_string(),
          ModuleExportIdent::new(
            dep_id.clone(),
            create_export_namespace_ident(&dep_id).to_id().into(),
            ModuleExportIdentType::VirtualNamespace,
          ),
        );
      }
    }
  }
}

pub fn update_module_graph_edges_of_cjs_modules(
  module_graph: &mut ModuleGraph,
  modules: Option<&HashSet<ModuleId>>,
) -> (
  HashMap<(ModuleId, String), (ModuleId, ModuleSystem)>,
  HashSet<ModuleId>,
) {
  let mut cjs_require_map_res = HashMap::default();
  let mut export_namespace_modules_res = HashSet::default();

  // Note that we update ResolveKind to Import here instead of in finalize_module because
  // when resolving dependencies ResolveKind::Require and ResolveKind::Import are different,
  // if we update ResolveKind::Require to ResolveKind::Import, it will break original dependency resolution
  let mut edges_to_update = vec![];

  for module in module_graph.modules() {
    if modules
      .as_ref()
      .map(|m| !m.contains(&module.id))
      .unwrap_or(false)
    {
      continue;
    }

    for (dep_id, edge_info) in module_graph.dependencies(&module.id) {
      // all hybrid modules will be transformed to cjs first, so we need to update all edges of hybrid modules
      let is_hybrid = if let box ModuleMetaData::Script(meta) = &module.meta {
        // Internal runtime modules are not hybrid modules
        meta.module_system == ModuleSystem::Hybrid
          && dep_id != FARM_RUNTIME_MODULE_HELPER_ID.into()
          && dep_id != FARM_RUNTIME_MODULE_SYSTEM_ID.into()
      } else {
        false
      };

      if is_hybrid || edge_info.contains_require() {
        edges_to_update.push((module.id.clone(), dep_id));
      }
    }
  }

  for (module_id, dep_id) in edges_to_update {
    let mut should_update_edge_kind = false;

    if let Some(edge_info) = module_graph.edge_info(&module_id, &dep_id) {
      // find require edge item
      if let Some(dep_module) = module_graph.module(&dep_id) {
        if !dep_module.external && dep_module.module_type.is_script() {
          let meta = dep_module.meta.as_script();

          let cjs_require_map =
            edge_info
              .items()
              .iter()
              .fold(HashMap::default(), |mut acc, item| {
                acc.insert(
                  (module_id.clone(), item.source.clone()),
                  (dep_id.clone(), meta.module_system.clone()),
                );
                acc
              });

          cjs_require_map_res.extend(cjs_require_map);

          should_update_edge_kind = true;
        }
      }

      if should_update_edge_kind {
        let mut edge_info = edge_info.clone();
        // update ResolveKind::require to ResolveKind::Import
        for item in edge_info.iter_mut() {
          if item.kind == ResolveKind::Require {
            item.kind = ResolveKind::Import;
          }
        }

        module_graph
          .update_edge(&module_id, &dep_id, edge_info)
          .unwrap();

        export_namespace_modules_res.insert(dep_id);
      }
    }
  }

  (cjs_require_map_res, export_namespace_modules_res)
}

pub fn get_cjs_require_only_modules(
  cjs_require_map: &HashMap<(ModuleId, String), (ModuleId, ModuleSystem)>,
) -> HashSet<&ModuleId> {
  cjs_require_map
    .values()
    .filter(|(_, module_system)| *module_system != ModuleSystem::EsModule)
    .map(|(module_id, _)| module_id)
    .collect()
}
