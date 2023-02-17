#![feature(box_patterns)]

use std::sync::Arc;

use deps_analyzer::DepsAnalyzer;
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{ModuleMetaData, ModuleSystem, ScriptModuleMetaData},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeModuleHookParam, PluginHookContext,
    PluginLoadHookParam, PluginLoadHookResult, PluginParseHookParam, PluginProcessModuleHookParam,
    ResolveKind,
  },
  resource::{
    resource_pot::{ResourcePot, ResourcePotType},
    Resource, ResourceType,
  },
  swc_common::{comments::NoopComments, Mark, DUMMY_SP, GLOBALS},
  swc_ecma_ast::{
    CallExpr, Callee, Expr, ExprStmt, Ident, MemberExpr, MemberProp, MetaPropExpr, MetaPropKind,
    ModuleItem, Stmt,
  },
};
use farmfe_toolkit::{
  fs::read_file_utf8,
  script::{codegen_module, module_type_from_id, parse_module, syntax_from_module_type},
  swc_ecma_transforms::{
    react::{react, Options},
    resolver,
    typescript::{strip, strip_with_jsx},
  },
  swc_ecma_visit::VisitMutWith,
};

mod deps_analyzer;
/// ScriptPlugin is used to support compiling js/ts/jsx/tsx/... files, support loading, parse, analyze dependencies and code generation.
/// Note that we do not do transforms here, the transforms (e.g. strip types, jsx...) are handled in a separate plugin (farmfe_plugin_swc_transforms).
pub struct FarmPluginScript {}

impl Plugin for FarmPluginScript {
  fn name(&self) -> &str {
    "FarmPluginScript"
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
        syntax.clone(),
        context.config.script.target.clone(),
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
      GLOBALS.set(&context.meta.script.globals, || {
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
              Default::default(),
              NoopComments, // TODO parse comments
              top_level_mark,
            ));
          }
          _ => {}
        }
      });
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
        module_ast,
        Mark::from_u32(module.meta.as_script().unresolved_mark),
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

    if param.deps.len() > 0 {
      let module_system =
        self.module_system_from_deps(param.deps.iter().map(|d| d.kind.clone()).collect());
      param.module.meta.as_script_mut().module_system = module_system;
    } else {
      // default to es module
      param.module.meta.as_script_mut().module_system = ModuleSystem::EsModule;
    }

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
  ) -> Result<Option<Vec<Resource>>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Js) {
      let ast = &resource_pot.meta.as_js().ast;
      let buf = codegen_module(
        ast,
        context.config.script.target.clone(),
        context.meta.script.cm.clone(),
      )
      .map_err(|e| CompilationError::GenerateResourcesError {
        name: resource_pot.id.to_string(),
        ty: resource_pot.resource_pot_type.clone(),
        source: Some(Box::new(e)),
      })?;

      Ok(Some(vec![Resource {
        bytes: buf,
        name: resource_pot.id.to_string().replace("../", "") + ".js", // TODO generate file name based on config
        emitted: false,
        resource_type: ResourceType::Js,
        resource_pot: resource_pot.id.clone(),
      }]))
    } else {
      Ok(None)
    }
  }
}

impl FarmPluginScript {
  pub fn new(_config: &Config) -> Self {
    Self {}
  }

  pub fn module_system_from_deps(&self, deps: Vec<ResolveKind>) -> ModuleSystem {
    let mut module_system = ModuleSystem::Custom(String::from("unknown"));

    for resolve_kind in deps {
      if matches!(resolve_kind, ResolveKind::Import)
        || matches!(resolve_kind, ResolveKind::DynamicImport)
      {
        match module_system {
          ModuleSystem::EsModule => continue,
          ModuleSystem::CommonJs => {
            module_system = ModuleSystem::Hybrid;
            break;
          }
          _ => module_system = ModuleSystem::EsModule,
        }
      } else if matches!(resolve_kind, ResolveKind::Require) {
        match module_system {
          ModuleSystem::CommonJs => continue,
          ModuleSystem::EsModule => {
            module_system = ModuleSystem::Hybrid;
            break;
          }
          _ => module_system = ModuleSystem::CommonJs,
        }
      }
    }

    module_system
  }
}
