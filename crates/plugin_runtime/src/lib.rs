#![feature(box_patterns)]

use std::sync::Arc;

use farmfe_core::{
  config::{AliasItem, Config, StringOrRegex},
  context::CompilationContext,
  error::CompilationError,
  module::ModuleType,
  plugin::{
    Plugin, PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginResolveHookParam, PluginResolveHookResult,
  },
  relative_path::RelativePath,
  resource::{
    meta_data::{js::JsResourcePotMetaData, ResourcePotMetaData},
    resource_pot::{ResourcePot, ResourcePotType},
    ResourceType,
  },
  serde_json, HashSet,
};

use farmfe_toolkit::{
  fs::read_file_utf8,
  html::get_farm_global_this,
  script::merge_swc_globals::{merge_comments, merge_sourcemap},
};

use handle_entry_resources::handle_entry_resources;
use handle_runtime_modules::{
  get_all_feature_flags, insert_runtime_modules, remove_unused_runtime_features,
  transform_normal_runtime_inputs_to_dynamic_entries,
};
use handle_runtime_plugins::insert_runtime_plugins;
use render_resource_pot::{external::handle_external_modules, *};

mod handle_entry_resources;
mod handle_runtime_modules;
mod handle_runtime_plugins;
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
    if !config.runtime.swc_helpers_path.is_empty() {
      config.resolve.alias.push(AliasItem::Complex {
        find: StringOrRegex::String("@swc/helpers".to_string()),
        replacement: config.runtime.swc_helpers_path.clone(),
      });
    }

    // DO NOT use Dynamic entry to compile runtime module, use normal input instead
    // the normal runtime input will be transformed into dynamic entry in module_graph_build_end hook to make sure only one runtime bundle is generated
    let mut add_runtime_dynamic_input = |name: &str, dir: &str| {
      let suffix = if name == "index" {
        "".to_string()
      } else {
        format!("/src/{dir}{name}")
      };

      config.input.insert(
        format!("{RUNTIME_INPUT_SCOPE}_{}", name.replace("-", "_")),
        format!("{RUNTIME_PACKAGE}{suffix}"),
      );
    };

    // Note that unused inputs will be removed in module_graph_build_end hook
    // add runtime package entry file for the first entry module
    add_runtime_dynamic_input("index", "");
    // module system is always required
    add_runtime_dynamic_input("module-system", "");
    add_runtime_dynamic_input("dynamic-import", "modules/");
    add_runtime_dynamic_input("module-helper", "modules/");
    add_runtime_dynamic_input("module-system-helper", "modules/");
    add_runtime_dynamic_input("plugin", "modules/");

    config.define.insert(
      "$__farm_global_this__$".to_string(),
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
    if param.source.starts_with(RUNTIME_PACKAGE) {
      if context.config.runtime.path.is_empty() {
        return Err(CompilationError::GenericError(
          "config.runtime.path is not set, please set or remove config.runtime.path in farm.config.ts. normally you should not set config.runtime.path manually".to_string(),
        ));
      }

      return Ok(Some(PluginResolveHookResult {
        resolved_path: param.source.to_string(),
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
    } else if param.resolved_path.starts_with(RUNTIME_PACKAGE) {
      let rest_str = param.resolved_path.replace(RUNTIME_PACKAGE, "") + ".ts";
      let relative_path = RelativePath::new(&rest_str);
      let resolved_path = relative_path
        .to_logical_path(&context.config.runtime.path)
        .to_string_lossy()
        .to_string();
      let code = read_file_utf8(&resolved_path)?;

      return Ok(Some(PluginLoadHookResult {
        content: code,
        module_type: ModuleType::Ts,
        source_map: None,
      }));
    }

    Ok(None)
  }

  fn module_graph_build_end(
    &self,
    module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let all_feature_flags = get_all_feature_flags(module_graph);
    transform_normal_runtime_inputs_to_dynamic_entries(module_graph, &all_feature_flags, context);
    // remove unused runtime features that controlled by feature guard like `if (__FARM_TARGET_ENV__)`
    // note that this must be called before insert_runtime_modules cause insert_runtime_modules will remove dynamic entries
    remove_unused_runtime_features(module_graph, &all_feature_flags, context);

    // find all runtime dynamic entries and insert them into runtime entry module
    insert_runtime_modules(module_graph, context);

    Ok(Some(()))
  }

  fn render_resource_pot(
    &self,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ResourcePotMetaData>> {
    // render normal script resource pot
    if resource_pot.resource_pot_type != ResourcePotType::Js {
      return Ok(None);
    }

    let module_graph = context.module_graph.read();

    let (rendered_modules, source_maps) =
      render_resource_pot_modules(resource_pot, &module_graph, context)?;

    let mut external_modules = HashSet::default();
    let mut module_asts = vec![];
    let mut comments = vec![];
    let mut sorted_modules = vec![];

    for rendered_module in rendered_modules {
      external_modules.extend(
        rendered_module
          .external_modules
          .into_iter()
          .map(|e| e.to_string()),
      );
      module_asts.push((
        rendered_module.module_id.clone(),
        rendered_module.rendered_ast,
      ));
      comments.push((rendered_module.module_id, rendered_module.comments));
      sorted_modules.extend(rendered_module.hoisted_module_ids);
    }

    let merged_sourcemap = merge_sourcemap(&mut module_asts, source_maps, context);
    // update the source map for the resource pot in the global meta so that it can be used in the next step
    context
      .meta
      .set_resource_pot_source_map(&resource_pot.id, merged_sourcemap.clone());

    let comments = merge_comments(&mut comments, merged_sourcemap);

    let merged_ast = merge_rendered_module::merge_rendered_module(&mut module_asts, context);
    let wrapped_resource_pot_ast =
      merge_rendered_module::wrap_resource_pot_ast(merged_ast, &resource_pot.id, context);

    let wrapped_resource_pot_ast = handle_external_modules(
      &resource_pot.id,
      wrapped_resource_pot_ast,
      &external_modules,
      context,
    )?;

    Ok(Some(ResourcePotMetaData::Js(JsResourcePotMetaData {
      ast: wrapped_resource_pot_ast,
      external_modules,
      rendered_modules: sorted_modules,
      comments: comments.into(),
    })))
  }

  /// Generate runtime resources
  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginGenerateResourcesHookResult>> {
    if hook_context.contain_caller(self.name()) || !Self::is_runtime_resource_pot(resource_pot) {
      return Ok(None);
    }

    let res = context
      .plugin_driver
      .generate_resources(
        resource_pot,
        context,
        &PluginHookContext {
          caller: hook_context.add_caller(self.name()),
          meta: hook_context.meta.clone(),
        },
      )?
      .map(|mut res| {
        for resource in &mut res.resources {
          resource.resource.resource_type = ResourceType::Runtime;
          // do not emit a
          resource.resource.emitted = true;
          // ignore source map for runtime
          resource.source_map = None;
        }
        res
      });

    Ok(res)
  }

  fn handle_entry_resource(
    &self,
    params: &mut farmfe_core::plugin::PluginHandleEntryResourceHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    handle_entry_resources(params, context);

    Ok(None)
  }
}

impl FarmPluginRuntime {
  pub fn new(_: &Config) -> Self {
    Self {}
  }

  fn is_runtime_resource_pot(resource_pot: &ResourcePot) -> bool {
    resource_pot.resource_pot_type == ResourcePotType::DynamicEntryJs
      && resource_pot.name.starts_with(RUNTIME_INPUT_SCOPE)
  }
}
