#![feature(box_patterns)]

use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  config::{
    config_regex::ConfigRegex, partial_bundling::PartialBundlingEnforceResourceConfig, Config,
  },
  context::CompilationContext,
  error::CompilationError,
  module::{ModuleMetaData, ModuleSystem, ModuleType},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginAnalyzeDepsHookResultEntry, PluginHookContext,
    PluginLoadHookParam, PluginLoadHookResult, PluginResolveHookParam, PluginResolveHookResult,
    PluginTransformHookResult, ResolveKind,
  },
  regex::Regex,
  resource::{
    resource_pot::{JsResourcePotMetaData, ResourcePot, ResourcePotMetaData, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    CallExpr, ExportAll, Expr, ExprOrSpread, ExprStmt, ImportDecl, ImportSpecifier, Lit,
    ModuleDecl, ModuleItem, Stmt, Str,
  },
};
use farmfe_toolkit::{
  fs::read_file_utf8,
  script::{codegen_module, module_system_from_deps, module_type_from_id, parse_module},
  swc_ecma_parser::Syntax,
};

use insert_runtime_plugins::insert_runtime_plugins;
use render_resource_pot::*;

const RUNTIME_SUFFIX: &str = ".farm-runtime";

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
pub struct FarmPluginRuntime {}

impl Plugin for FarmPluginRuntime {
  fn name(&self) -> &str {
    "FarmPluginRuntime"
  }

  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    // runtime package entry file
    config.input.insert(
      "runtime".to_string(),
      format!("{}{}", config.runtime.path, RUNTIME_SUFFIX),
    );
    config.resolve.alias.insert(
      "@swc/helpers".to_string(),
      config.runtime.swc_helpers_path.clone(),
    );

    config.partial_bundling.enforce_resources.insert(
      0,
      PartialBundlingEnforceResourceConfig {
        name: "FARM_RUNTIME".to_string(),
        test: vec![ConfigRegex(
          Regex::new(&format!(".+{}", RUNTIME_SUFFIX)).unwrap(),
        )],
      },
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

    if let ModuleMetaData::Script(script) = &param.module.meta {
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

      if !param.deps.is_empty() {
        let module_system =
          module_system_from_deps(param.deps.iter().map(|d| d.kind.clone()).collect());
        param.module.meta.as_script_mut().module_system = module_system;
      } else {
        // default to es module
        param.module.meta.as_script_mut().module_system = ModuleSystem::EsModule;
      }

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn process_resource_pots(
    &self,
    resource_pots: &mut Vec<&mut ResourcePot>,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if context.meta.script.runtime_ast.read().is_some() {
      return Ok(None);
    }

    let module_graph = context.module_graph.write();

    for resource_pot in resource_pots {
      if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime) {
        let rendered_resource_pot_ast =
          resource_pot_to_runtime_object_lit(resource_pot, &module_graph, context)?;

        #[cfg(not(windows))]
        let minimal_runtime = include_str!("./js-runtime/minimal-runtime.js");
        #[cfg(windows)]
        let minimal_runtime = include_str!(".\\js-runtime\\minimal-runtime.js");

        let mut runtime_ast = parse_module(
          "farm_internal_minimal_runtime",
          minimal_runtime,
          Syntax::Es(context.config.script.parser.es_config),
          context.config.script.target,
          context.meta.script.cm.clone(),
        )?;

        if let ModuleItem::Stmt(Stmt::Expr(ExprStmt {
          expr: box Expr::Call(CallExpr { args, .. }),
          ..
        })) = &mut runtime_ast.body[0]
        {
          args[0] = ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Object(rendered_resource_pot_ast)),
          };
          args[1] = ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Lit(Lit::Str(Str {
              span: DUMMY_SP,
              value: resource_pot
                .entry_module
                .as_ref()
                .unwrap()
                .id(context.config.mode.clone())
                .into(),
              raw: None,
            }))),
          };
        }

        // TODO: minify runtime when minify is enabled

        // TODO transform async function if target is lower than es2017, should not externalize swc helpers
        // This may cause async generator duplicated but it's ok for now. We can fix it later.

        context.meta.script.runtime_ast.write().replace(runtime_ast);
        break;
      }
    }

    Ok(Some(()))
  }

  fn render_resource_pot(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // the runtime module and its plugins should be in the same resource pot
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Js) {
      let module_graph = context.module_graph.read();
      let rendered_resource_pot_ast =
        resource_pot_to_runtime_object_lit(resource_pot, &module_graph, context)?;

      #[cfg(not(windows))]
      let wrapper = include_str!("./js-runtime/resource-wrapper.js");
      #[cfg(windows)]
      let wrapper = include_str!(".\\js-runtime\\resource-wrapper.js");

      let mut wrapper_ast = parse_module(
        "farm_internal_resource_wrapper",
        wrapper,
        Syntax::Es(context.config.script.parser.es_config),
        context.config.script.target,
        context.meta.script.cm.clone(),
      )?;

      if let ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        expr: box Expr::Call(CallExpr { args, .. }),
        ..
      })) = &mut wrapper_ast.body[0]
      {
        args[0] = ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Object(rendered_resource_pot_ast)),
        };
      }

      resource_pot.meta = ResourcePotMetaData::Js(JsResourcePotMetaData { ast: wrapper_ast });

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<Resource>>> {
    if matches!(&hook_context.caller, Some(c) if c == self.name()) {
      return Ok(None);
    }

    // only handle runtime resource pot
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime) {
      let runtime_ast = context.meta.script.runtime_ast.read();
      let runtime_ast = runtime_ast.as_ref().unwrap();
      let bytes = codegen_module(
        runtime_ast,
        context.config.script.target,
        context.meta.script.cm.clone(),
        None,
        context.config.minify,
      )
      .map_err(|e| CompilationError::GenerateResourcesError {
        name: resource_pot.id.to_string(),
        ty: resource_pot.resource_pot_type.clone(),
        source: Some(Box::new(e)),
      })?;
      // set emitted property of Runtime to true by default, as it will be generated and injected when generating entry resources,
      // other plugins wants to modify this behavior in write_resources hook.
      Ok(Some(vec![Resource {
        name: resource_pot.id.to_string(),
        bytes,
        emitted: true, // do not emit runtime resource by default
        resource_type: ResourceType::Runtime,
        origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
      }]))
    } else {
      Ok(None)
    }
  }
}

impl FarmPluginRuntime {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}
