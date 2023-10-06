#![feature(box_patterns)]
#![feature(path_file_prefix)]

use std::sync::Arc;

use deps_analyzer::DepsAnalyzer;
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{ModuleMetaData, ModuleSystem, ModuleType, ScriptModuleMetaData},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeModuleHookParam,
    PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam, PluginProcessModuleHookParam,
  },
  resource::{
    resource_pot::{ResourcePot, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  swc_common::{comments::NoopComments, Mark, GLOBALS},
  swc_ecma_ast::{
    CallExpr, Callee, Expr, ExprStmt, Ident, MemberExpr, MemberProp, ModuleItem, Stmt,
  },
};
use farmfe_toolkit::{
  fs::read_file_utf8,
  script::{
    codegen_module, module_system_from_deps, module_type_from_id, parse_module,
    swc_try_with::try_with, syntax_from_module_type,
  },
  sourcemap::swc_gen::{build_source_map, AstModule},
  swc_ecma_transforms::{
    resolver,
    typescript::{strip, strip_with_jsx},
  },
  swc_ecma_visit::VisitMutWith,
};

use import_meta_vistor::ImportMetaVisitor;
use swc_plugins::{init_plugin_module_cache_once, transform_by_swc_plugins};

mod deps_analyzer;
mod handle_entry_resources;
mod import_meta_vistor;
mod swc_plugins;

const FARM_NODE_MODULE: &str = "__farmNodeModule";

/// ScriptPlugin is used to support compiling js/ts/jsx/tsx/... files, support loading, parse, analyze dependencies and code generation.
/// Note that we do not do transforms here, the transforms (e.g. strip types, jsx...) are handled in a separate plugin (farmfe_plugin_swc_transforms).
pub struct FarmPluginScript {}

impl Plugin for FarmPluginScript {
  fn name(&self) -> &str {
    "FarmPluginScript"
  }

  fn priority(&self) -> i32 {
    99
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    let module_type = module_type_from_id(param.resolved_path);

    if let Some(module_type) = module_type {
      if module_type.is_script() {
        let content = read_file_utf8(param.resolved_path)?;

        Ok(Some(PluginLoadHookResult {
          content,
          module_type,
        }))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  fn parse(
    &self,
    param: &PluginParseHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<ModuleMetaData>> {
    if let Some(syntax) =
      syntax_from_module_type(&param.module_type, context.config.script.parser.clone())
    {
      let mut swc_module = parse_module(
        &param.module_id.to_string(),
        &param.content,
        syntax,
        context.config.script.target,
        context.meta.script.cm.clone(),
      )?;

      GLOBALS.set(&context.meta.script.globals, || {
        let top_level_mark = Mark::new();
        let unresolved_mark = Mark::new();

        swc_module.visit_mut_with(&mut resolver(
          unresolved_mark,
          top_level_mark,
          param.module_type.is_typescript(),
        ));

        let meta = ScriptModuleMetaData {
          ast: swc_module,
          top_level_mark: top_level_mark.as_u32(),
          unresolved_mark: unresolved_mark.as_u32(),
          // set module_system to unknown, it will be detected in `finalize_module`
          module_system: ModuleSystem::Custom(String::from("unknown")),
          // set module_type to unknown, it will be detected in `finalize_module`
          hmr_accepted: false,
        };

        Ok(Some(ModuleMetaData::Script(meta)))
      })
    } else {
      Ok(None)
    }
  }

  fn process_module(
    &self,
    param: &mut PluginProcessModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if param.module_type.is_typescript() {
      try_with(
        context.meta.script.cm.clone(),
        &context.meta.script.globals,
        || {
          let top_level_mark = Mark::from_u32(param.meta.as_script().top_level_mark);
          let ast = &mut param.meta.as_script_mut().ast;

          match param.module_type {
            farmfe_core::module::ModuleType::Js => {
              // TODO downgrade syntax
            }
            farmfe_core::module::ModuleType::Jsx => {
              // Do nothing, jsx should be handled by other plugins
            }
            farmfe_core::module::ModuleType::Ts => {
              ast.visit_mut_with(&mut strip(top_level_mark));
            }
            farmfe_core::module::ModuleType::Tsx => {
              ast.visit_mut_with(&mut strip_with_jsx(
                context.meta.script.cm.clone(),
                // TODO make it configurable
                Default::default(),
                NoopComments, // TODO parse comments
                top_level_mark,
              ));
            }
            _ => {}
          }
        },
      )?;
    }

    // execute swc plugins
    if param.module_type.is_script() && !context.config.script.plugins.is_empty() {
      try_with(
        context.meta.script.cm.clone(),
        &context.meta.script.globals,
        || transform_by_swc_plugins(param, context).unwrap(),
      )?;
    }

    Ok(Some(()))
  }

  fn analyze_deps(
    &self,
    param: &mut PluginAnalyzeDepsHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    let module = param.module;

    if module.module_type.is_script() {
      let module_ast = &module.meta.as_script().ast;
      // TODO deal with dynamic import, when dynamic import and static import are mixed, using static import
      let mut analyzer = DepsAnalyzer::new(
        &module.id,
        module_ast,
        Mark::from_u32(module.meta.as_script().unresolved_mark),
        Mark::from_u32(module.meta.as_script().top_level_mark),
      );

      GLOBALS.set(&context.meta.script.globals, || {
        let deps = analyzer.analyze_deps();
        param.deps.extend(deps);
      });

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  /// detect [ModuleSystem] for a script module based on its dependencies' [ResolveKind] and detect hmr_accepted
  fn finalize_module(
    &self,
    param: &mut PluginFinalizeModuleHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if !param.module.module_type.is_script() {
      return Ok(None);
    }
    // all jsx, js, ts, tsx modules should be transformed to js module for now
    // cause the partial bundling is not support other module type yet
    param.module.module_type = ModuleType::Js;
    // default to commonjs
    let module_system = if !param.deps.is_empty() {
      module_system_from_deps(param.deps.iter().map(|d| d.kind.clone()).collect())
    } else {
      ModuleSystem::CommonJs
    };
    param.module.meta.as_script_mut().module_system = module_system.clone();

    let ast = &param.module.meta.as_script().ast;

    if module_system != ModuleSystem::Hybrid {
      // if the ast contains ModuleDecl, it's a esm module
      for item in ast.body.iter() {
        if let ModuleItem::ModuleDecl(_) = item {
          if module_system == ModuleSystem::CommonJs && !param.deps.is_empty() {
            param.module.meta.as_script_mut().module_system = ModuleSystem::Hybrid;
          } else {
            param.module.meta.as_script_mut().module_system = ModuleSystem::EsModule;
          }
          break;
        }
      }
    }

    // transform `import.meta.xxx` to `module.meta.xxx`
    let ast = &mut param.module.meta.as_script_mut().ast;
    ast.visit_mut_with(&mut ImportMetaVisitor {});

    let ast = &param.module.meta.as_script().ast;
    // detect hmr based on `module.meta.hot.accept()`
    for item in ast.body.iter() {
      if let ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        expr:
          box Expr::Call(CallExpr {
            callee:
              Callee::Expr(box Expr::Member(MemberExpr {
                obj:
                  box Expr::Member(MemberExpr {
                    obj:
                      box Expr::Member(MemberExpr {
                        obj: box Expr::Ident(Ident { sym: module, .. }),
                        prop: MemberProp::Ident(Ident { sym: meta, .. }),
                        ..
                      }),
                    prop: MemberProp::Ident(Ident { sym: hot, .. }),
                    ..
                  }),
                prop: MemberProp::Ident(Ident { sym: accept, .. }),
                ..
              })),
            ..
          }),
        ..
      })) = item
      {
        if &module.to_string() == "module"
          && &meta.to_string() == "meta"
          && &hot.to_string() == "hot"
          && &accept.to_string() == "accept"
        {
          param.module.meta.as_script_mut().hmr_accepted = true;
          break;
        }
      }
    }

    Ok(None)
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginGenerateResourcesHookResult>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Js) {
      // handle js entry resource pot
      // handle_entry_resource_pot::handle_entry_resource_pot(resource_pot, context)?;

      let ast = &resource_pot.meta.as_js().ast;
      let mut src_map_buf = vec![];

      let buf = codegen_module(
        ast,
        context.config.script.target,
        context.meta.script.cm.clone(),
        Some(&mut src_map_buf),
        context.config.minify,
      )
      .map_err(|e| CompilationError::GenerateResourcesError {
        name: resource_pot.id.to_string(),
        ty: resource_pot.resource_pot_type.clone(),
        source: Some(Box::new(e)),
      })?;

      let resource = Resource {
        bytes: buf,
        name: resource_pot.name.to_string(),
        emitted: false,
        resource_type: ResourceType::Js,
        origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
      };
      let mut source_map = None;

      if context.config.sourcemap.enabled()
        && (context.config.sourcemap.is_all() || !resource_pot.immutable)
      {
        let module_graph = context.module_graph.read();
        let src_map = build_source_map(
          &src_map_buf,
          context.meta.script.cm.clone(),
          AstModule::Script(ast),
          &module_graph,
        );

        source_map = Some(Resource {
          bytes: src_map,
          name: format!("{}.map", resource_pot.name.to_string()),
          emitted: false,
          resource_type: ResourceType::SourceMap(resource_pot.id.to_string()),
          origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
        });
      }

      Ok(Some(PluginGenerateResourcesHookResult {
        resource,
        source_map,
      }))
    } else {
      Ok(None)
    }
  }

  fn finalize_resources(
    &self,
    resources_map: &mut farmfe_core::hashbrown::HashMap<String, Resource>,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    handle_entry_resources::handle_entry_resources(resources_map, context);
    Ok(Some(()))
  }
}

impl FarmPluginScript {
  pub fn new(config: &Config) -> Self {
    init_plugin_module_cache_once(config);
    Self {}
  }
}
