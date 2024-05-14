#![feature(box_patterns)]

use std::{
  any::Any,
  collections::{HashMap, HashSet, VecDeque},
  sync::Arc,
};

use farmfe_core::{
  config::{
    config_regex::ConfigRegex, external::ExternalConfig,
    partial_bundling::PartialBundlingEnforceResourceConfig, Config, ModuleFormat, TargetEnv,
    FARM_MODULE_SYSTEM,
  },
  context::CompilationContext,
  enhanced_magic_string::types::SourceMapOptions,
  error::CompilationError,
  module::{ModuleId, ModuleMetaData, ModuleType},
  parking_lot::Mutex,
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginAnalyzeDepsHookResultEntry,
    PluginFinalizeResourcesHookParams, PluginGenerateResourcesHookResult, PluginHookContext,
    PluginLoadHookParam, PluginLoadHookResult, PluginResolveHookParam, PluginResolveHookResult,
    PluginTransformHookResult, ResolveKind,
  },
  resource::{
    resource_pot::{ResourcePot, ResourcePotMetaData, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  serde_json,
  swc_ecma_ast::{ExportAll, ImportDecl, ImportSpecifier, ModuleDecl, ModuleItem},
};
use farmfe_toolkit::{
  fs::read_file_utf8,
  html::get_farm_global_this,
  script::{module_type_from_id, set_module_system_for_module_meta},
};

use insert_runtime_plugins::insert_runtime_plugins;
use render_resource_pot::*;

pub const RUNTIME_SUFFIX: &str = ".farm-runtime";
pub const ASYNC_MODULES: &str = "async_modules";

mod find_async_modules;
mod handle_entry_resources;
mod insert_runtime_plugins;
pub mod render_resource_pot;

/// FarmPluginRuntime is charge of:
/// * resolving, parsing and generating a executable runtime code and inject the code into the entries.
/// * merge module's ast and render the script module using farm runtime's specification, for example, wrap the module to something like `function(module, exports, require) { xxx }`, see [Farm Runtime RFC](https://github.com/farm-fe/rfcs/pull/1)
///
/// The runtime supports html entry and script(js/jsx/ts/tsx) entry, when entry is html, the runtime will be injected as a inline <script /> tag in the <head /> tag;
/// when entry is script, the runtime will be injected into the entry module's head, makes sure the runtime execute before all other code.
///
/// All runtime module (including the runtime core and its plugins) will be suffixed as `.farm-runtime` to distinguish with normal script modules.
pub struct FarmPluginRuntime {
  runtime_code: Mutex<Arc<String>>,
}

impl Plugin for FarmPluginRuntime {
  fn name(&self) -> &str {
    "FarmPluginRuntime"
  }

  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    // runtime package entry file
    if !config.runtime.path.is_empty() {
      config.input.insert(
        "runtime".to_string(),
        format!("{}{}", config.runtime.path, RUNTIME_SUFFIX),
      );
    }

    if !config.runtime.swc_helpers_path.is_empty() {
      config.resolve.alias.insert(
        "@swc/helpers".to_string(),
        config.runtime.swc_helpers_path.clone(),
      );
    }

    config.partial_bundling.enforce_resources.insert(
      0,
      PartialBundlingEnforceResourceConfig {
        name: "FARM_RUNTIME".to_string(),
        test: vec![ConfigRegex::new(&format!(".+{}", RUNTIME_SUFFIX))],
      },
    );

    config.define.insert(
      "'<@__farm_global_this__@>'".to_string(),
      serde_json::Value::String(format!(
        "{}",
        get_farm_global_this(&config.runtime.namespace)
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
    if matches!(&hook_context.caller, Some(c) if c == "FarmPluginRuntime") {
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
          caller: Some(String::from("FarmPluginRuntime")),
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
        panic!("unknown module type for {}", real_file_path);
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

  fn analyze_deps(
    &self,
    param: &mut PluginAnalyzeDepsHookParam,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !param.module.id.relative_path().ends_with(RUNTIME_SUFFIX) {
      return Ok(None);
    }

    if let ModuleMetaData::Script(script) = &*param.module.meta {
      let mut has_import_star = false;
      let mut has_import_default = false;
      let mut has_export_star = false;

      // insert swc cjs module helper as soon as it has esm import
      for stmt in &script.ast.body {
        if let ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl { specifiers, .. })) = stmt {
          has_import_star = has_export_star
            || specifiers
              .iter()
              .any(|sp| matches!(sp, ImportSpecifier::Namespace(_)));
          has_import_default = has_import_default
            || specifiers
              .iter()
              .any(|specifier| matches!(specifier, ImportSpecifier::Default(_)));
        } else if let ModuleItem::ModuleDecl(ModuleDecl::ExportAll(ExportAll { .. })) = stmt {
          has_export_star = true;
        }
      }

      let exists = |source: &str, param: &mut PluginAnalyzeDepsHookParam| {
        param.deps.iter().any(|dep| dep.source == source)
      };
      let insert_import =
        |source: &str, kind: ResolveKind, param: &mut PluginAnalyzeDepsHookParam| {
          param.deps.push(PluginAnalyzeDepsHookResultEntry {
            kind,
            source: source.to_string(),
          });
        };

      if has_import_star && !exists("@swc/helpers/_/_interop_require_wildcard", param) {
        insert_import(
          "@swc/helpers/_/_interop_require_wildcard",
          ResolveKind::Import,
          param,
        );
      }

      if has_import_default && !exists("@swc/helpers/_/_interop_require_default", param) {
        insert_import(
          "@swc/helpers/_/_interop_require_default",
          ResolveKind::Import,
          param,
        );
      }

      if has_export_star && !exists("@swc/helpers/_/_export_star", param) {
        insert_import("@swc/helpers/_/_export_star", ResolveKind::Import, param);
      }
    } else {
      return Ok(None);
    }

    Ok(Some(()))
  }

  fn finalize_module(
    &self,
    param: &mut farmfe_core::plugin::PluginFinalizeModuleHookParam,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if param.module.id.relative_path().ends_with(RUNTIME_SUFFIX) {
      param.module.module_type = ModuleType::Runtime;

      set_module_system_for_module_meta(param);

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
    // module graph updated is called when the module graph is updated
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

        if farmfe_toolkit::swc_ecma_utils::contains_top_level_await(ast) {
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

  fn process_resource_pots(
    &self,
    resource_pots: &mut Vec<&mut ResourcePot>,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !self.runtime_code.lock().is_empty() {
      return Ok(None);
    }
    let async_modules = self.get_async_modules(context);
    let async_modules = async_modules.downcast_ref::<HashSet<ModuleId>>().unwrap();
    let module_graph = context.module_graph.read();

    for resource_pot in resource_pots {
      if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime) {
        let RenderedJsResourcePot { mut bundle, .. } =
          resource_pot_to_runtime_object(resource_pot, &module_graph, async_modules, context)?;

        bundle.prepend(
          r#"(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};t[e]=i;r[e](i,i.exports,o,n);return i.exports}o(e)})("#,
        );

        bundle.append(
          &format!(
            ",{:?});",
            resource_pot
              .entry_module
              .as_ref()
              .unwrap()
              .id(context.config.mode.clone())
          ),
          None,
        );

        *self.runtime_code.lock() = Arc::new(bundle.to_string());
        break;
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
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime) {
      return Ok(Some(ResourcePotMetaData {
        rendered_modules: HashMap::new(),
        rendered_content: self.runtime_code.lock().clone(),
        rendered_map_chain: vec![],
        ..Default::default()
      }));
    } else if matches!(resource_pot.resource_pot_type, ResourcePotType::Js) {
      let async_modules = self.get_async_modules(context);
      let async_modules = async_modules.downcast_ref::<HashSet<ModuleId>>().unwrap();
      let module_graph = context.module_graph.read();
      let external_config = ExternalConfig::from(&*context.config);
      let RenderedJsResourcePot {
        mut bundle,
        rendered_modules,
        external_modules,
      } = resource_pot_to_runtime_object(resource_pot, &module_graph, async_modules, context)?;

      let mut external_modules_str = None;

      let farm_global_this = get_farm_global_this(&context.config.runtime.namespace);

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
          name = format!("__farm_external_module_{}", name);

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
                format!("{source:?}: {{...{name},__esModule:true}}")
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
            .find_match(&source)
            .map(|v| v.source(&source))
            // it's maybe from plugin
            .unwrap_or(source.clone());

          let source_obj = format!("(globalThis||window||{{}})['{}']||{{}}", replace_source);
          external_objs.push(if context.config.output.format == ModuleFormat::EsModule {
            format!("{source:?}: {{...({source_obj}),__esModule:true}}")
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

      let is_target_node_and_cjs = context.config.output.target_env == TargetEnv::Node
        && context.config.output.format == ModuleFormat::CommonJs;

      let str = format!(
        r#"(function(_){{for(var r in _){{_[r].__farm_resource_pot__={};{farm_global_this}.{FARM_MODULE_SYSTEM}.register(r,_[r])}}}})("#,
        if is_target_node_and_cjs {
          "'file://'+__filename".to_string()
        } else {
          // TODO make it final output file name
          format!("'{}'", resource_pot.name.to_string() + ".js")
        },
      );

      bundle.prepend(&str);
      bundle.append(");", None);

      if let Some(external_modules_str) = external_modules_str {
        bundle.prepend(&external_modules_str);
      }

      return Ok(Some(ResourcePotMetaData {
        rendered_modules,
        rendered_content: Arc::new(bundle.to_string()),
        rendered_map_chain: if context.config.sourcemap.enabled(resource_pot.immutable) {
          let root = context.config.root.clone();
          let map = bundle
            .generate_map(SourceMapOptions {
              include_content: Some(true),
              remap_source: Some(Box::new(move |src| {
                format!("/{}", farmfe_utils::relative(&root, src))
              })),
              ..Default::default()
            })
            .map_err(|_| CompilationError::GenerateSourceMapError {
              id: resource_pot.id.to_string(),
            })?;
          let mut buf = vec![];
          map
            .to_writer(&mut buf)
            .map_err(|e| CompilationError::RenderScriptModuleError {
              id: resource_pot.id.to_string(),
              source: Some(Box::new(e)),
            })?;

          vec![Arc::new(String::from_utf8(buf).unwrap())]
        } else {
          vec![]
        },
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

  fn finalize_resources(
    &self,
    param: &mut PluginFinalizeResourcesHookParams,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    handle_entry_resources::handle_entry_resources(param.resources_map, context);

    Ok(Some(()))
  }
}

impl FarmPluginRuntime {
  pub fn new(_: &Config) -> Self {
    Self {
      runtime_code: Mutex::new(Arc::new(String::new())),
    }
  }

  pub(crate) fn get_async_modules<'a>(
    &'a self,
    context: &'a Arc<CompilationContext>,
  ) -> farmfe_core::dashmap::mapref::one::Ref<'_, String, Box<dyn Any + Send + Sync>> {
    context.custom.get(ASYNC_MODULES).unwrap()
  }
}
