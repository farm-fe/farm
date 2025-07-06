#![feature(box_patterns)]

use farmfe_core::{
  config::{
    config_regex::ConfigRegex, partial_bundling::PartialBundlingEnforceResourceConfig, Config,
    LibraryBundleType, ModuleFormatConfig,
  },
  error::CompilationError,
  module::{
    meta_data::script::{
      ModuleExportIdent, ModuleExportIdentType, FARM_RUNTIME_MODULE_HELPER_ID,
      FARM_RUNTIME_MODULE_SYSTEM_ID,
    },
    ModuleId, ModuleMetaData, ModuleSystem, ModuleType,
  },
  parking_lot::Mutex,
  plugin::{
    Plugin, PluginAnalyzeDepsHookResultEntry, PluginGenerateResourcesHookResult,
    PluginResolveHookResult, ResolveKind,
  },
  rayon::iter::{IntoParallelRefMutIterator, ParallelIterator},
  relative_path::RelativePath,
  resource::{
    meta_data::{js::JsResourcePotMetaData, ResourcePotMetaData},
    resource_pot::ResourcePotType,
  },
  swc_ecma_ast::Module,
  HashMap, HashSet,
};
use farmfe_toolkit::{
  fs::read_file_utf8,
  runtime::RuntimeFeatureGuardRemover,
  script::{
    concatenate_modules::{
      concatenate_modules_ast, ConcatenateModulesAstOptions, ConcatenateModulesAstResult,
      EXPORT_NAMESPACE,
    },
    create_export_namespace_ident, set_module_system_for_module_meta,
    swc_try_with::try_with,
  },
  swc_ecma_visit::VisitMutWith,
};
use transform_cjs::transform_cjs_to_esm;

use crate::{
  formats::{generate_library_format_resources, GenerateLibraryFormatResourcesOptions},
  transform_cjs::{replace_cjs_require, ReplaceCjsRequireResult},
  transform_hybrid::transform_hybrid_to_cjs,
};

mod formats;
mod handle_exports;
mod import_meta_visitor;
mod transform_cjs;
mod transform_hybrid;
mod utils;

const FARM_RUNTIME_PREFIX: &str = "@farm-runtime/";
const PLUGIN_NAME: &str = "FarmPluginLibrary";

#[derive(Default)]
pub struct FarmPluginLibrary {
  export_namespace_modules: Mutex<HashSet<ModuleId>>,
  cjs_require_map: Mutex<HashMap<(ModuleId, String), (ModuleId, ModuleSystem)>>,

  library_bundle_type: LibraryBundleType,

  runtime_module_helper_ast: Mutex<Option<Module>>,
  all_used_helper_idents: Mutex<HashSet<String>>,
  should_add_farm_node_require: Mutex<bool>,
}

impl FarmPluginLibrary {
  pub fn new(config: &Config) -> Self {
    Self {
      library_bundle_type: config.output.library_bundle_type,
      ..Self::default()
    }
  }
}

impl Plugin for FarmPluginLibrary {
  fn name(&self) -> &str {
    PLUGIN_NAME
  }

  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    if !config.partial_bundling.enforce_resources.is_empty() {
      println!("[Farm warn] Config `partial_bundling.enforce_resources` does not work under library mode, it will be ignored.");
      config.partial_bundling.enforce_resources = vec![];
    }

    match config.output.library_bundle_type {
      LibraryBundleType::SingleBundle => {
        if config.input.len() > 1 {
          panic!("When output.library_bundle_type is single-bundle, output.input should configure only one entry, currently there are {} inputs", config.input.len());
        }

        config
          .partial_bundling
          .enforce_resources
          .push(PartialBundlingEnforceResourceConfig {
            name: config.input.iter().next().unwrap().0.to_string(),
            test: vec![ConfigRegex::new(".+")],
          });
      }
      LibraryBundleType::MultipleBundle => {
        config.partial_bundling.target_concurrent_requests = 1;
        config.partial_bundling.target_min_size = usize::MAX;
      }
      LibraryBundleType::BundleLess => {
        config.partial_bundling.target_concurrent_requests = usize::MAX;
        config.partial_bundling.target_min_size = 0;
      }
    }

    // add runtime module helper as entry, it will be removed from the module graph later
    config.input.insert(
      FARM_RUNTIME_MODULE_HELPER_ID.to_string(),
      FARM_RUNTIME_MODULE_HELPER_ID.to_string(),
    );

    // add [format] place holder if there are multiple formats
    if matches!(config.output.format, ModuleFormatConfig::Multiple(_)) {
      if !config.output.filename.contains("[format]") {
        config.output.filename = format!("[format]/{}", config.output.filename);
      }

      if !config.output.entry_filename.contains("[format]") {
        config.output.entry_filename = format!("[format]/{}", config.output.entry_filename);
      }
    }

    // update public path
    if config.output.public_path.starts_with("/") {
      config.output.public_path = format!("./{}", config.output.public_path);
    }

    Ok(Some(()))
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
      let rel_path = match rel_path {
        "module-system" => "src/module-system.ts",
        "module-helper" => "src/modules/module-helper.ts",
        _ => unreachable!("unsupported runtime path {rel_path}"),
      };
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
        let module_id_str = param.module.id.to_string();
        if module_id_str.starts_with(FARM_RUNTIME_PREFIX) {
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
            source: FARM_RUNTIME_MODULE_SYSTEM_ID.to_string(),
            kind: ResolveKind::Import,
          });
          param.deps.push(PluginAnalyzeDepsHookResultEntry {
            source: FARM_RUNTIME_MODULE_HELPER_ID.to_string(),
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
    // Remove module helper from module graph entry and clone it's ast
    let runtime_helper_id = FARM_RUNTIME_MODULE_HELPER_ID.into();
    module_graph.entries.remove(&runtime_helper_id);

    if let Some(helper_module) = module_graph.module(&runtime_helper_id) {
      let mut module_helper_ast = self.runtime_module_helper_ast.lock();
      *module_helper_ast = Some(helper_module.meta.as_script().ast.clone());
    }

    // Note that we update ResolveKind to Import here instead of in finalize_module because
    // when resolving dependencies ResolveKind::Require and ResolveKind::Import are different,
    // if we update ResolveKind::Require to ResolveKind::Import, it will break original dependency resolution
    let mut edges_to_update = vec![];

    for module in module_graph.modules() {
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

            self.cjs_require_map.lock().extend(cjs_require_map);

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

          self.export_namespace_modules.lock().insert(dep_id);
        }
      }
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
    let cjs_require_map: HashMap<(ModuleId, String), (ModuleId, ModuleSystem)> =
      cjs_require_map.drain().into_iter().collect();
    let cjs_required_only_modules: HashSet<&ModuleId> = cjs_require_map
      .values()
      .filter(|(_, module_system)| *module_system != ModuleSystem::EsModule)
      .map(|(module_id, _)| module_id)
      .collect();

    module_graph
      .modules_mut()
      .par_iter_mut()
      .filter(|module| module.module_type.is_script())
      .for_each(|module| {
        let meta = module.meta.as_script_mut();

        let cm = context.meta.get_module_source_map(&module.id);
        let globals = context.meta.get_globals(&module.id);
        let is_required_cjs_module = cjs_required_only_modules.contains(&module.id);
        let mut used_helper_idents = HashSet::default();

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

          if should_add_farm_node_require {
            *self.should_add_farm_node_require.lock() = true;
          }

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

        self
          .all_used_helper_idents
          .lock()
          .extend(used_helper_idents.into_iter().map(|s| s.to_string()));
      });

    let export_namespace_modules = self.export_namespace_modules.lock();

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

    let entry_module_id = if self.library_bundle_type == LibraryBundleType::BundleLess
      && resource_pot.modules().len() == 1
    {
      resource_pot.modules().first().unwrap()
    } else if let Some(entry) = resource_pot.entry_module.as_ref() {
      entry
    } else if let Some(entry) = resource_pot.dynamic_imported_entry_module.as_ref() {
      entry
    } else {
      panic!(
        "dynamic imported entry module not found for resource pot {:?}",
        resource_pot.id
      );
    };

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
      entry_module_id,
      &resource_pot.modules,
      &module_graph,
      ConcatenateModulesAstOptions { check_esm: true },
      context,
    )
    .map_err(|e| CompilationError::GenericError(e.to_string()))?;

    context
      .meta
      .set_resource_pot_source_map(&resource_pot.id, source_map);
    context
      .meta
      .set_resource_pot_globals(&resource_pot.id, globals);

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
    resource_pot: &mut farmfe_core::resource::resource_pot::ResourcePot,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginGenerateResourcesHookResult>> {
    if hook_context.contain_caller(self.name()) {
      return Ok(None);
    }

    if resource_pot.resource_pot_type != ResourcePotType::Js {
      return Ok(None);
    }

    let mut result = PluginGenerateResourcesHookResult { resources: vec![] };
    let hook_context = hook_context.clone_and_append_caller(self.name());
    let runtime_module_helper_ast = self.runtime_module_helper_ast.lock();
    let mut all_used_helper_idents = self.all_used_helper_idents.lock();
    let should_add_farm_node_require = *self.should_add_farm_node_require.lock();

    result.resources = generate_library_format_resources(
      resource_pot,
      runtime_module_helper_ast.as_ref().unwrap(),
      &mut all_used_helper_idents,
      &GenerateLibraryFormatResourcesOptions {
        should_add_farm_node_require,
      },
      context,
      &hook_context,
    )?;

    Ok(Some(result))
  }
}
