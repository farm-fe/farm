#![feature(box_patterns)]

use std::{
  any::Any,
  collections::{HashMap, HashSet, VecDeque},
  sync::Arc,
};

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
  module::{ModuleId, ModuleType},
  plugin::{
    Plugin, PluginFinalizeResourcesHookParams, PluginGenerateResourcesHookResult,
    PluginHookContext, PluginLoadHookParam, PluginLoadHookResult, PluginResolveHookParam,
    PluginResolveHookResult, PluginTransformHookResult,
  },
  resource::{
    resource_pot::{ResourcePot, ResourcePotMetaData, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  serde_json,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{Expr, ExprStmt, Module as SwcModule, ModuleItem, Stmt},
};
use farmfe_toolkit::{
  fs::read_file_utf8,
  html::get_farm_global_this,
  script::{module_type_from_id, set_module_system_for_module_meta},
};

use farmfe_utils::hash::base64_encode;
use insert_runtime_plugins::insert_runtime_plugins;
use merge_rendered_module::RenderResourcePotAstResult;
use render_resource_pot::*;

pub use farmfe_toolkit::script::constant::RUNTIME_SUFFIX;
pub const ASYNC_MODULES: &str = "async_modules";

mod find_async_modules;
mod handle_entry_resources;
mod insert_runtime_plugins;
pub mod render_resource_pot;

const PLUGIN_NAME: &str = "FarmPluginRuntime";
/// FarmPluginRuntime is charge of:
/// * resolving, parsing and generating a executable runtime code and inject the code into the entries.
/// * merge module's ast and render the script module using farm runtime's specification, for example, wrap the module to something like `function(module, exports, require) { xxx }`, see [Farm Runtime RFC](https://github.com/farm-fe/rfcs/pull/1)
///
/// The runtime supports html entry and script(js/jsx/ts/tsx) entry, when entry is html, the runtime will be injected as a inline <script /> tag in the <head /> tag;
/// when entry is script, the runtime will be injected into the entry module's head, makes sure the runtime execute before all other code.
///
/// All runtime module (including the runtime core and its plugins) will be suffixed as `.farm-runtime` to distinguish with normal script modules.
pub struct FarmPluginRuntime {}

impl Plugin for FarmPluginRuntime {
  fn name(&self) -> &str {
    PLUGIN_NAME
  }

  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
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

  fn resolve(
    &self,
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginResolveHookResult>> {
    // avoid cyclic resolve
    if hook_context.contain_caller(PLUGIN_NAME) {
      Ok(None)
    } else if param.source.ends_with(RUNTIME_SUFFIX) // if the source is a runtime module or its importer is a runtime module, then resolve it to the runtime module
      || (param.importer.is_some()
        && param
          .importer
          .as_ref()
          .unwrap()
          .relative_path()
          .ends_with(RUNTIME_SUFFIX))
    {
      let ori_source = param.source.replace(RUNTIME_SUFFIX, "");
      let resolve_result = context.plugin_driver.resolve(
        &PluginResolveHookParam {
          source: ori_source,
          ..param.clone()
        },
        context,
        &PluginHookContext {
          caller: hook_context.add_caller(PLUGIN_NAME),
          meta: HashMap::new(),
        },
      )?;

      if let Some(mut res) = resolve_result {
        res.resolved_path = format!("{}{}", res.resolved_path, RUNTIME_SUFFIX);
        Ok(Some(res))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    if param.resolved_path.ends_with(RUNTIME_SUFFIX) {
      let real_file_path = param.resolved_path.replace(RUNTIME_SUFFIX, "");
      let content = read_file_utf8(&real_file_path)?;

      if let Some(module_type) = module_type_from_id(&real_file_path) {
        Ok(Some(PluginLoadHookResult {
          content,
          module_type,
          source_map: None,
        }))
      } else {
        panic!("unknown module type for {real_file_path}");
      }
    } else {
      Ok(None)
    }
  }

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

  fn generate_start(
    &self,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // detect async module like top level await when start rendering
    // render start is only called once when the compilation start
    context.custom.insert(
      ASYNC_MODULES.to_string(),
      Box::new(find_async_modules::find_async_modules(context)),
    );

    Ok(Some(()))
  }

  fn module_graph_updated(
    &self,
    param: &farmfe_core::plugin::PluginModuleGraphUpdatedHookParams,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // detect async module like top level await when module graph updated
    // module graph updated is called during compiler.update
    let mut async_modules = context.custom.get_mut(ASYNC_MODULES).unwrap();
    let async_modules = async_modules.downcast_mut::<HashSet<ModuleId>>().unwrap();

    for remove in &param.removed_modules_ids {
      async_modules.remove(remove);
    }

    let module_graph = context.module_graph.read();
    let mut added_async_modules = vec![];
    // find added modules that contains top level await
    let mut analyze_top_level_await = |module_id: &ModuleId| {
      let module = module_graph.module(module_id).unwrap();

      if module.module_type.is_script() {
        let ast = &module.meta.as_script().ast;
        let dependencies = module_graph.dependencies(module_id);
        let is_deps_async = dependencies
          .iter()
          .any(|(dep, edge)| async_modules.contains(dep) && !edge.is_dynamic());
        if is_deps_async || farmfe_toolkit::swc_ecma_utils::contains_top_level_await(ast) {
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

    while !queue.is_empty() {
      let module_id = queue.pop_front().unwrap();
      async_modules.insert(module_id.clone());

      for (dept, edge) in module_graph.dependents(&module_id) {
        if !async_modules.contains(&dept) && !edge.is_dynamic() {
          queue.push_back(dept);
        }
      }
    }

    Ok(Some(()))
  }

  fn render_resource_pot_modules(
    &self,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ResourcePotMetaData>> {
    if !context.config.output.target_env.is_library()
      && matches!(resource_pot.resource_pot_type, ResourcePotType::Js)
    {
      let async_modules = self.get_async_modules(context);
      let async_modules = async_modules.downcast_ref::<HashSet<ModuleId>>().unwrap();
      let module_graph = context.module_graph.read();
      let external_config = ExternalConfig::from(&*context.config);
      let (mut code, map, external_modules) =
        resource_pot_to_runtime_object(resource_pot, &module_graph, async_modules, context)?;

      let mut external_modules_str = None;

      let farm_global_this = get_farm_global_this(
        &context.config.runtime.namespace,
        &context.config.output.target_env,
      );

      let external_modules = external_modules
        .into_iter()
        .map(|id| id.to_string())
        .collect::<HashSet<_>>();
      let mut external_modules = external_modules.into_iter().collect::<Vec<_>>();
      external_modules.sort();

      // inject global externals
      if !external_modules.is_empty() && context.config.output.target_env == TargetEnv::Node {
        let mut import_strings = vec![];
        let mut source_to_names = vec![];

        for external_module in external_modules {
          // replace all invalid characters with `_`
          let mut name = external_module
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>();
          name = format!("__farm_external_module_{name}");

          let import_str = if context.config.output.format == ModuleFormat::EsModule {
            format!("import * as {name} from {external_module:?};")
          } else {
            format!("var {name} = require({external_module:?});")
          };
          import_strings.push(import_str);
          source_to_names.push((name, external_module));
        }

        let mut prepend_str = import_strings.join("");
        prepend_str.push_str(&format!(
          "{farm_global_this}.{FARM_MODULE_SYSTEM}.setExternalModules({{{}}});",
          source_to_names
            .into_iter()
            .map(
              |(name, source)| if context.config.output.format == ModuleFormat::EsModule {
                format!("{source:?}: {name} && {name}.default && !{name}.__esModule ? {{...{name},__esModule:true}} : {{...{name}}}")
              } else {
                format!("{source:?}: {name}")
              }
            )
            .collect::<Vec<_>>()
            .join(",")
        ));

        external_modules_str = Some(prepend_str);
      } else if !external_modules.is_empty()
        && context.config.output.target_env == TargetEnv::Browser
      {
        let mut external_objs = Vec::new();

        for source in external_modules {
          let replace_source = external_config
            .find_match(&source.to_string())
            .map(|v| v.source(&source.to_string()))
            // it's maybe from plugin
            .unwrap_or(source.to_string());

          let source_obj = format!("window['{replace_source}']||{{}}");
          external_objs.push(if context.config.output.format == ModuleFormat::EsModule {
            format!("{source:?}: ({source_obj}).default && !({source_obj}).__esModule ? {{...({source_obj}),__esModule:true}} : {source_obj}")
          } else {
            format!("{source:?}: {source_obj}")
          });
        }

        let prepend_str = format!(
          "{farm_global_this}.{FARM_MODULE_SYSTEM}.setExternalModules({{{}}});",
          external_objs.join(",")
        );
        external_modules_str = Some(prepend_str);
      }

      if let Some(external_modules_str) = external_modules_str {
        code = external_modules_str + code.as_str();
      }

      return Ok(Some(ResourcePotMetaData {
        rendered_modules: HashMap::new(),
        rendered_content: Arc::new(code),
        rendered_map_chain: map.map(|m| vec![m]).unwrap_or(vec![]),
        // rendered_map_chain: if context.config.sourcemap.enabled(resource_pot.immutable) {
        //   let root = context.config.root.clone();
        //   let map = bundle
        //     .generate_map(SourceMapOptions {
        //       include_content: Some(true),
        //       remap_source: Some(Box::new(move |src| {
        //         format!("/{}", farmfe_utils::relative(&root, src))
        //       })),
        //       hires: if context.config.minify.enabled() {
        //         Some(MappingsOptionHires::Boundary)
        //       } else {
        //         None
        //       },
        //       ..Default::default()
        //     })
        //     .map_err(|_| CompilationError::GenerateSourceMapError {
        //       id: resource_pot.id.to_string(),
        //     })?;
        //   let mut buf = vec![];
        //   map
        //     .to_writer(&mut buf)
        //     .map_err(|e| CompilationError::RenderScriptModuleError {
        //       id: resource_pot.id.to_string(),
        //       source: Some(Box::new(e)),
        //     })?;

        //   vec![Arc::new(String::from_utf8(buf).unwrap())]
        // } else {
        //   vec![]
        // },
        ..Default::default()
      }));
    }

    Ok(None)
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    _context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginGenerateResourcesHookResult>> {
    if matches!(&hook_context.caller, Some(c) if c == self.name()) {
      return Ok(None);
    }

    // only handle runtime resource pot
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime) {
      Ok(Some(PluginGenerateResourcesHookResult {
        resource: Resource {
          name: resource_pot.id.to_string(),
          bytes: resource_pot.meta.rendered_content.as_bytes().to_vec(),
          emitted: true, // do not emit runtime resource by default. The runtime will be injected into the html or script entry.
          resource_type: ResourceType::Runtime,
          origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
          info: None,
        },
        source_map: None,
      }))
    } else {
      Ok(None)
    }
  }

  fn render_update_resource_pot(
    &self,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginRenderResourcePotHookResult>>
  {
    println!(
      "render update resource pot {} {}",
      resource_pot.id,
      resource_pot.resource_pot_type != ResourcePotType::Js || resource_pot.modules().is_empty()
    );
    // The hmr result should alway be a js resource
    if resource_pot.resource_pot_type != ResourcePotType::Js || resource_pot.modules().is_empty() {
      return Ok(None);
    }

    let async_modules = self.get_async_modules(context);
    let async_modules = async_modules.downcast_ref::<HashSet<ModuleId>>().unwrap();
    let module_graph = context.module_graph.read();
    println!("start");
    let modules = render_resource_pot_modules(resource_pot, &module_graph, async_modules, context)?;
    println!("rendered modules");
    let RenderResourcePotAstResult {
      rendered_resource_pot_ast,
      merged_comments,
      merged_sourcemap,
      external_modules,
    } = merge_rendered_module::merge_rendered_module(modules, context);
    println!("merged modules");
    let (mut code, map) = generate_code_and_sourcemap(
      resource_pot,
      &module_graph,
      &SwcModule {
        shebang: None,
        span: DUMMY_SP,
        body: vec![ModuleItem::Stmt(Stmt::Expr(ExprStmt {
          span: DUMMY_SP,
          expr: Box::new(Expr::Object(rendered_resource_pot_ast)),
        }))],
      },
      merged_sourcemap,
      merged_comments,
      context,
    )?;
    println!("generated code");

    if let Some(map) = &map {
      // inline source map
      code = format!(
        "{}\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,{}",
        code,
        base64_encode(map.as_bytes())
      );
    }

    Ok(Some(
      farmfe_core::plugin::PluginRenderResourcePotHookResult {
        content: code,
        source_map: map,
      },
    ))
  }

  fn finalize_resources(
    &self,
    param: &mut PluginFinalizeResourcesHookParams,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if context.config.output.target_env.is_library() {
      return Ok(None);
    }

    let async_modules = self.get_async_modules(context);
    let async_modules = async_modules.downcast_ref::<HashSet<ModuleId>>().unwrap();
    handle_entry_resources::handle_entry_resources(param.resources_map, context, async_modules);

    Ok(Some(()))
  }
}

impl FarmPluginRuntime {
  pub fn new(_: &Config) -> Self {
    Self {}
  }

  pub(crate) fn get_async_modules<'a>(
    &'a self,
    context: &'a Arc<CompilationContext>,
  ) -> farmfe_core::dashmap::mapref::one::Ref<'_, String, Box<dyn Any + Send + Sync>> {
    context.custom.get(ASYNC_MODULES).unwrap()
  }
}
