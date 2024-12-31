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
    Plugin, PluginAnalyzeDepsHookResultEntry, PluginFinalizeResourcesHookParams,
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
pub struct FarmPluginRuntime {}

impl Plugin for FarmPluginRuntime {
  fn name(&self) -> &str {
    PLUGIN_NAME
  }

  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    // runtime package entry file
    if !config.runtime.path.is_empty() {
      config
        .input
        .insert(RUNTIME_INPUT_SCOPE.to_string(), RUNTIME_PACKAGE.to_string());
    }

    if !config.runtime.swc_helpers_path.is_empty() {
      config.resolve.alias.push(AliasItem::Complex {
        find: StringOrRegex::String("@swc/helpers".to_string()),
        replacement: config.runtime.swc_helpers_path.clone(),
      });
    }

    // config.partial_bundling.enforce_resources.insert(
    //   0,
    //   PartialBundlingEnforceResourceConfig {
    //     name: RUNTIME_INPUT_SCOPE.to_string(),
    //     test: vec![ConfigRegex::new(&format!(".+{RUNTIME_INPUT_SCOPE}"))],
    //   },
    // );

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
        content: insert_runtime_plugins("", context),
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

    let mut add_runtime_dynamic_input = |name: &str| {
      param.deps.push(PluginAnalyzeDepsHookResultEntry {
        source: format!("@farmfe/runtime/src/modules/{name}"),
        kind: ResolveKind::DynamicEntry {
          name: format!("{RUNTIME_INPUT_SCOPE}_{}", name.replace("-", "_")),
          output_filename: None,
        },
      });
    };

    // The goal of rendering runtime code is to make sure the runtime is as small as possible.
    // So we need to collect all the runtime related information in finalize_module hook,
    // for example, if a module uses dynamic import, we will append import '@farmfe/runtime/modules/dynamic-import' to the runtime entry module.
    let feature_flags = &param.module.meta.as_script().feature_flags;

    if feature_flags.contains(&FeatureFlag::DefaultImport) {
      add_runtime_dynamic_input("dynamic-import");
    }

    if feature_flags.contains(&FeatureFlag::ModuleDecl) {
      add_runtime_dynamic_input("module-system-helper");
    }

    // module system is always required
    add_runtime_dynamic_input("module-system");

    if context.config.mode.is_dev() {
      add_runtime_dynamic_input("module-helper");
    }

    if context.config.runtime.plugins.len() > 0 {
      add_runtime_dynamic_input("plugin");
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

  fn render_resource_pot(
    &self,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ResourcePotMetaData>> {
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
    Self {}
  }
}
