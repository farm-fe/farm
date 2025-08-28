#![feature(box_patterns)]

use farmfe_core::{
  config::{
    config_regex::ConfigRegex, partial_bundling::PartialBundlingEnforceResourceConfig, Config,
    LibraryBundleType, ModuleFormatConfig,
  },
  error::CompilationError,
  module::{
    meta_data::script::{
      FARM_RUNTIME_MODULE_HELPER_ID, FARM_RUNTIME_MODULE_SYSTEM_ID, FARM_RUNTIME_SUFFIX,
    },
    ModuleId, ModuleSystem, ModuleType,
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
    },
    set_module_system_for_module_meta,
    swc_try_with::{try_with, ResetSpanVisitMut},
    transform_to_esm::{
      self, get_cjs_require_only_modules, update_module_graph_edges_of_cjs_modules,
    },
  },
  swc_ecma_visit::VisitMutWith,
};

use crate::formats::{generate_library_format_resources, GenerateLibraryFormatResourcesOptions};

mod formats;
mod import_meta_visitor;
mod utils;

const FARM_RUNTIME_PREFIX: &str = "@farmfe/runtime/";
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
    if let Some(rel_path) = param
      .resolved_path
      .strip_prefix(FARM_RUNTIME_PREFIX)
      .and_then(|rel_path| rel_path.strip_suffix(FARM_RUNTIME_SUFFIX))
    {
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
      let mut helper_ast = helper_module.meta.as_script().ast.clone();
      // reset span for helper ast
      helper_ast.visit_mut_with(&mut ResetSpanVisitMut);
      let mut module_helper_ast = self.runtime_module_helper_ast.lock();
      *module_helper_ast = Some(helper_ast);
    }

    let (cjs_require_map, export_namespace_modules) =
      update_module_graph_edges_of_cjs_modules(module_graph, None);

    self.cjs_require_map.lock().extend(cjs_require_map);
    self
      .export_namespace_modules
      .lock()
      .extend(export_namespace_modules);

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
    let cjs_required_only_modules: HashSet<&ModuleId> =
      get_cjs_require_only_modules(&cjs_require_map);

    module_graph
      .modules_mut()
      .par_iter_mut()
      .filter(|module| module.module_type.is_script())
      .for_each(|module| {
        let (used_helper_idents, should_add_farm_node_require) =
          transform_to_esm::transform_module_to_esm(
            module,
            &cjs_require_map,
            &cjs_required_only_modules,
            context,
          );

        if should_add_farm_node_require {
          *self.should_add_farm_node_require.lock() = true;
        }

        self
          .all_used_helper_idents
          .lock()
          .extend(used_helper_idents.into_iter().map(|s| s.to_string()));
      });

    let export_namespace_modules = self.export_namespace_modules.lock();

    transform_to_esm::update_export_namespace_ident(
      &export_namespace_modules,
      &cjs_required_only_modules,
      module_graph,
    );

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
      custom: Default::default(),
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
