#![feature(box_patterns)]

use std::{any::Any, collections::VecDeque, sync::Arc};

use farmfe_core::{
  config::{
    config_regex::ConfigRegex,
    external::{self, ExternalConfig},
    partial_bundling::PartialBundlingEnforceResourceConfig,
    AliasItem, Config, ModuleFormat, StringOrRegex, TargetEnv, FARM_MODULE_SYSTEM,
  },
  context::CompilationContext,
  enhanced_magic_string::types::{MappingsOptionHires, SourceMapOptions},
  error::CompilationError,
  module::{meta_data::script::feature_flag::FeatureFlag, Module, ModuleId, ModuleType},
  parking_lot::Mutex,
  plugin::{
    GeneratedResource, Plugin, PluginAnalyzeDepsHookResultEntry, PluginFinalizeResourcesHookParams,
    PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginResolveHookParam, PluginResolveHookResult,
    PluginTransformHookResult, ResolveKind,
  },
  resource::{
    meta_data::{js::JsResourcePotMetaData, ResourcePotMetaData},
    resource_pot::{ResourcePot, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  serde_json,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{Expr, ExprStmt, Module as SwcModule, ModuleItem, Stmt},
  HashMap, HashSet,
};
use farmfe_plugin_bundle::resource_pot_to_bundle::{BundleGroup, ShareBundleOptions, SharedBundle};
use farmfe_toolkit::{
  fs::read_file_utf8,
  html::get_farm_global_this,
  itertools::Itertools,
  script::{
    module_type_from_id, set_module_system_for_module_meta,
    sourcemap::{merge_comments, merge_sourcemap},
  },
};

use farmfe_utils::hash::base64_encode;
use insert_runtime_modules::insert_runtime_modules;
use insert_runtime_plugins::insert_runtime_plugins;
use render_resource_pot::{external::handle_external_modules, *};

mod handle_entry_resources;
mod insert_runtime_modules;
mod insert_runtime_plugins;
pub mod render_resource_pot;

const PLUGIN_NAME: &str = "FarmPluginRuntime";
pub const RUNTIME_INPUT_SCOPE: &str = "farm_internal_runtime";
pub const RUNTIME_PACKAGE: &str = "@farmfe/runtime";

/// FarmPluginRuntime is charge of:
/// * resolving, parsing and generating a executable runtime code and inject the code into the entries.
/// * merge module's ast and render the script module using farm runtime's specification, for example, wrap the module to something like `function(module, exports, require) { xxx }`, see [Farm Runtime RFC](https://github.com/farm-fe/rfcs/pull/1)
///
/// All runtime module (including the runtime core and its plugins) will be suffixed as `.farm-runtime` to distinguish with normal script modules.
pub struct FarmPluginRuntime {
  added_runtime_modules: Mutex<HashSet<String>>,
}

impl Plugin for FarmPluginRuntime {
  fn name(&self) -> &str {
    PLUGIN_NAME
  }

  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    if !config.runtime.swc_helpers_path.is_empty() {
      config.resolve.alias.push(AliasItem::Complex {
        find: StringOrRegex::String("@swc/helpers".to_string()),
        replacement: config.runtime.swc_helpers_path.clone(),
      });
    }

    config.define.insert(
      "'<@__farm_global_this__@>'".to_string(),
      serde_json::Value::String(format!(
        "{}",
        get_farm_global_this(&config.runtime.namespace, &config.output.target_env)
      )),
    );

    Ok(Some(()))
  }

  fn resolve(
    &self,
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginResolveHookResult>> {
    if param.source == RUNTIME_PACKAGE {
      return Ok(Some(PluginResolveHookResult {
        resolved_path: RUNTIME_PACKAGE.to_string(),
        ..Default::default()
      }));
    } else if param.source.starts_with(RUNTIME_PACKAGE) {
      if context.config.runtime.path.is_empty() {
        return Err(CompilationError::GenericError(
          "config.runtime.path is not set, please set or remove config.runtime.path in farm.config.ts. normally you should not set config.runtime.path manually".to_string(),
        ));
      }

      let rest_str = param.source.replace(RUNTIME_PACKAGE, "");

      return Ok(Some(PluginResolveHookResult {
        resolved_path: format!("{}{}.ts", context.config.runtime.path, rest_str),
        ..Default::default()
      }));
    }

    Ok(None)
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    // load farm runtime entry as a empty module, it will be filled later in freeze_module hook
    if param.resolved_path == RUNTIME_PACKAGE {
      return Ok(Some(PluginLoadHookResult {
        content: insert_runtime_plugins(context),
        module_type: ModuleType::Js,
        source_map: None,
      }));
    }

    Ok(None)
  }

  fn finalize_module(
    &self,
    param: &mut farmfe_core::plugin::PluginFinalizeModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !param.module.module_type.is_script() {
      return Ok(None);
    }

    let mut add_runtime_dynamic_input = |name: &str, dir: &str| {
      // add runtime module to the dynamic input if it's not added
      let mut added_runtime_modules = self.added_runtime_modules.lock();

      if added_runtime_modules.contains(name) {
        return;
      }

      let suffix = if name == "index" {
        "".to_string()
      } else {
        format!("/src/{dir}{name}")
      };

      param.deps.push(PluginAnalyzeDepsHookResultEntry {
        source: format!("{RUNTIME_PACKAGE}{suffix}"),
        kind: ResolveKind::DynamicEntry {
          name: format!("{RUNTIME_INPUT_SCOPE}_{}", name.replace("-", "_")),
          output_filename: None,
          no_importer: true,
        },
      });

      added_runtime_modules.insert(name.to_string());
    };

    // add runtime package entry file for the first entry module
    add_runtime_dynamic_input("index", "");

    // The goal of rendering runtime code is to make sure the runtime is as small as possible.
    // So we need to collect all the runtime related information in finalize_module hook,
    // for example, if a module uses dynamic import, we will append import '@farmfe/runtime/src/modules/dynamic-import' to the runtime entry module.
    let feature_flags = &param.module.meta.as_script().feature_flags;

    if feature_flags.contains(&FeatureFlag::DefaultImport) {
      add_runtime_dynamic_input("dynamic-import", "modules/");
    }

    if feature_flags.contains(&FeatureFlag::ModuleDecl) {
      add_runtime_dynamic_input("module-system-helper", "modules/");
    }

    // module system is always required
    add_runtime_dynamic_input("module-system", "");

    if context.config.mode.is_dev() {
      add_runtime_dynamic_input("module-helper", "modules/");
    }

    if context.config.runtime.plugins.len() > 0 {
      add_runtime_dynamic_input("plugin", "modules/");
    }

    Ok(Some(()))
  }

  fn module_graph_build_end(
    &self,
    module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    insert_runtime_modules(module_graph, context);
    Ok(Some(()))
  }

  fn process_resource_pots(
    &self,
    resource_pots: &mut Vec<&mut ResourcePot>,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // find runtime resource pot and set the resource pot type to Runtime
    for resource_pot in resource_pots {
      if resource_pot.name.starts_with(RUNTIME_INPUT_SCOPE) {
        resource_pot.resource_pot_type = ResourcePotType::Runtime;
      }
    }

    Ok(Some(()))
  }

  fn render_resource_pot(
    &self,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ResourcePotMetaData>> {
    // render runtime resource pot
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime) {
      let module_graph = context.module_graph.read();
      let bundle_group = BundleGroup::from(resource_pot);
      let bundle_group_id = bundle_group.id.clone();
      // concatenate all runtime modules, all runtime modules should be esm only
      let mut bundle = SharedBundle::new(vec![bundle_group], &module_graph, context, None)?;
      bundle.render()?;

      let result = bundle.codegen(&bundle_group_id)?;
      return Ok(Some(ResourcePotMetaData::Js(JsResourcePotMetaData {
        ast: result.ast,
        external_modules: Default::default(),
        rendered_modules: result.rendered_modules,
        comments: result.comments,
      })));
    }

    if resource_pot.resource_pot_type != ResourcePotType::Js {
      return Ok(None);
    }

    let module_graph = context.module_graph.read();

    let (mut rendered_modules, hoisted_map) =
      render_resource_pot_modules(resource_pot, &module_graph, context)?;

    let merged_sourcemap = merge_sourcemap(
      &resource_pot.id,
      &mut rendered_modules,
      &module_graph,
      context,
      &hoisted_map,
    );

    let comments = merge_comments(&mut rendered_modules, merged_sourcemap, &hoisted_map);

    let merged_ast = merge_rendered_module::merge_rendered_module(&mut rendered_modules, context);
    let wrapped_resource_pot_ast =
      merge_rendered_module::wrap_resource_pot_ast(merged_ast, &resource_pot.id, context);

    let external_modules = rendered_modules
      .iter()
      .flat_map(|m| m.external_modules.iter().map(|e| e.to_string()))
      .collect::<HashSet<_>>();

    let wrapped_resource_pot_ast = handle_external_modules(
      &resource_pot.id,
      wrapped_resource_pot_ast,
      &external_modules,
      context,
    )?;

    Ok(Some(ResourcePotMetaData::Js(JsResourcePotMetaData {
      ast: wrapped_resource_pot_ast,
      external_modules,
      rendered_modules: rendered_modules.into_iter().map(|m| m.module_id).collect(),
      comments: comments.into(),
    })))
  }

  // fn handle_entry_resource(
  //   &self,
  //   _resource: &mut farmfe_core::plugin::PluginHandleEntryResourceHookParams,
  //   _context: &Arc<CompilationContext>,
  // ) -> farmfe_core::error::Result<Option<()>> {
  //   // TODO handle runtime resource for entry
  //   Ok(None)
  // }

  fn finalize_resources(
    &self,
    param: &mut PluginFinalizeResourcesHookParams,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if context.config.output.target_env.is_library() {
      return Ok(None);
    }

    handle_entry_resources::handle_entry_resources(param.resources_map, context);

    Ok(Some(()))
  }
}

impl FarmPluginRuntime {
  pub fn new(_: &Config) -> Self {
    Self {
      added_runtime_modules: Mutex::new(HashSet::default()),
    }
  }
}
