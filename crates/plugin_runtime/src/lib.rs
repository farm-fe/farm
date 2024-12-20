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
  module::{Module, ModuleId, ModuleType},
  plugin::{
    Plugin, PluginFinalizeResourcesHookParams, PluginGenerateResourcesHookResult,
    PluginHookContext, PluginLoadHookParam, PluginLoadHookResult, PluginResolveHookParam,
    PluginResolveHookResult, PluginTransformHookResult,
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
  script::{
    module_type_from_id, set_module_system_for_module_meta,
    sourcemap::{merge_comments, merge_sourcemap},
  },
};

use farmfe_utils::hash::base64_encode;
use insert_runtime_plugins::insert_runtime_plugins;
use render_resource_pot::{external::handle_external_modules, *};

pub use farmfe_toolkit::script::constant::RUNTIME_SUFFIX;

mod find_async_modules;
mod handle_entry_resources;
mod insert_runtime_plugins;
pub mod render_resource_pot;

const PLUGIN_NAME: &str = "FarmPluginRuntime";
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
    println!("config runtime");
    if config.output.target_env.is_library() {
      return Ok(None);
    }
    // runtime package entry file
    if !config.runtime.path.is_empty() {
      config.input.insert(
        "runtime".to_string(),
        format!("{}{}", config.runtime.path, RUNTIME_SUFFIX),
      );
    }

    if !config.runtime.swc_helpers_path.is_empty() {
      config.resolve.alias.push(AliasItem::Complex {
        find: StringOrRegex::String("@swc/helpers".to_string()),
        replacement: config.runtime.swc_helpers_path.clone(),
      });
    }

    config.partial_bundling.enforce_resources.insert(
      0,
      PartialBundlingEnforceResourceConfig {
        name: "FARM_RUNTIME".to_string(),
        test: vec![ConfigRegex::new(&format!(".+{RUNTIME_SUFFIX}"))],
      },
    );

    config.define.insert(
      "'<@__farm_global_this__@>'".to_string(),
      serde_json::Value::String(format!(
        "{}",
        get_farm_global_this(&config.runtime.namespace, &config.output.target_env)
      )),
    );

    Ok(Some(()))
  }

  // fn resolve(
  //   &self,
  //   param: &PluginResolveHookParam,
  //   context: &Arc<CompilationContext>,
  //   hook_context: &PluginHookContext,
  // ) -> farmfe_core::error::Result<Option<PluginResolveHookResult>> {
  //   // avoid cyclic resolve
  //   if hook_context.contain_caller(PLUGIN_NAME) {
  //     Ok(None)
  //   } else if param.source.ends_with(RUNTIME_SUFFIX) // if the source is a runtime module or its importer is a runtime module, then resolve it to the runtime module
  //     || (param.importer.is_some()
  //       && param
  //         .importer
  //         .as_ref()
  //         .unwrap()
  //         .relative_path()
  //         .ends_with(RUNTIME_SUFFIX))
  //   {
  //     let ori_source = param.source.replace(RUNTIME_SUFFIX, "");
  //     let resolve_result = context.plugin_driver.resolve(
  //       &PluginResolveHookParam {
  //         source: ori_source,
  //         ..param.clone()
  //       },
  //       context,
  //       &PluginHookContext {
  //         caller: hook_context.add_caller(PLUGIN_NAME),
  //         meta: HashMap::default(),
  //       },
  //     )?;

  //     if let Some(mut res) = resolve_result {
  //       res.resolved_path = format!("{}{}", res.resolved_path, RUNTIME_SUFFIX);
  //       Ok(Some(res))
  //     } else {
  //       Ok(None)
  //     }
  //   } else {
  //     Ok(None)
  //   }
  // }

  // fn load(
  //   &self,
  //   param: &PluginLoadHookParam,
  //   _context: &Arc<CompilationContext>,
  //   _hook_context: &PluginHookContext,
  // ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
  //   if param.resolved_path.ends_with(RUNTIME_SUFFIX) {
  //     let real_file_path = param.resolved_path.replace(RUNTIME_SUFFIX, "");
  //     let content = read_file_utf8(&real_file_path)?;

  //     if let Some(module_type) = module_type_from_id(&real_file_path) {
  //       Ok(Some(PluginLoadHookResult {
  //         content,
  //         module_type,
  //         source_map: None,
  //       }))
  //     } else {
  //       panic!("unknown module type for {real_file_path}");
  //     }
  //   } else {
  //     Ok(None)
  //   }
  // }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    let farm_runtime_module_id = format!("{}{}", context.config.runtime.path, RUNTIME_SUFFIX);
    // if the module is runtime entry, then inject runtime plugins
    if farm_runtime_module_id == param.resolved_path {
      return Ok(Some(PluginTransformHookResult {
        content: insert_runtime_plugins(param.content.clone(), context),
        module_type: Some(param.module_type.clone()),
        source_map: None,
        ignore_previous_source_map: false,
      }));
    }

    Ok(None)
  }

  fn finalize_module(
    &self,
    param: &mut farmfe_core::plugin::PluginFinalizeModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if param.module.id.relative_path().ends_with(RUNTIME_SUFFIX) {
      param.module.module_type = ModuleType::Runtime;

      set_module_system_for_module_meta(param, context);

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn freeze_module(
    &self,
    _param: &mut Module,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // The goal of rendering runtime code is to make sure the runtime is as small as possible.
    // So we need to collect all the runtime related information in finalize_module hook,
    // and use these information to append more runtime abilities in freeze_module hook.
    // for example, if a module uses dynamic import, we will append import '@farmfe/runtime/modules/dynamic-import' to the runtime entry module.
    Ok(None)
  }

  fn generate_start(
    &self,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // detect async module like top level await when start rendering
    // render start is only called once when the compilation start
    // context.custom.insert(
    //   ASYNC_MODULES.to_string(),
    //   Box::new(find_async_modules::find_async_modules(context)),
    // );

    let async_modules = find_async_modules::find_async_modules(context);
    let mut module_graph = context.module_graph.write();

    for module_id in async_modules {
      let module = module_graph.module_mut(&module_id).unwrap();
      module.meta.as_script_mut().is_async = true;
    }

    Ok(Some(()))
  }

  fn module_graph_updated(
    &self,
    param: &farmfe_core::plugin::PluginModuleGraphUpdatedHookParams,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // detect async module like top level await when module graph updated
    // module graph updated is called during compiler.update
    let module_graph = context.module_graph.read();
    let mut added_async_modules = vec![];
    // find added modules that contains top level await
    let mut analyze_top_level_await = |module_id: &ModuleId| {
      let module = module_graph.module(module_id).unwrap();

      if module.module_type.is_script() {
        let is_async = module.meta.as_script().is_async;
        let dependencies = module_graph.dependencies(module_id);
        let is_deps_async = dependencies.iter().any(|(dep_id, edge)| {
          let dep = module_graph.module(dep_id).unwrap();
          dep.meta.as_script().is_async && !edge.is_dynamic()
        });
        if is_deps_async || is_async {
          added_async_modules.push(module_id.clone());
        }
      }
    };
    for added in &param.added_modules_ids {
      analyze_top_level_await(added);
    }
    for updated in &param.updated_modules_ids {
      analyze_top_level_await(updated);
    }

    let mut queue = VecDeque::from(added_async_modules.into_iter().collect::<Vec<_>>());
    let mut async_modules = HashSet::default();

    while !queue.is_empty() {
      let module_id = queue.pop_front().unwrap();
      async_modules.insert(module_id.clone());

      for (dept, edge) in module_graph.dependents(&module_id) {
        if !async_modules.contains(&dept) && !edge.is_dynamic() {
          queue.push_back(dept);
        }
      }
    }

    drop(module_graph);

    // update async modules
    let mut module_graph = context.module_graph.write();

    for module_id in async_modules {
      let module = module_graph.module_mut(&module_id).unwrap();
      module.meta.as_script_mut().is_async = true;
    }

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

    // let merged_sourcemap = merge_sourcemap(
    //   &resource_pot.id,
    //   &mut rendered_modules,
    //   &module_graph,
    //   context,
    //   &hoisted_map,
    // );

    // let comments = merge_comments(&mut rendered_modules, merged_sourcemap, &hoisted_map);

    Ok(Some(ResourcePotMetaData::Js(JsResourcePotMetaData {
      ast: wrapped_resource_pot_ast,
      external_modules,
      rendered_modules: rendered_modules.into_iter().map(|m| m.module_id).collect(),
      // comments: comments.into(),
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
