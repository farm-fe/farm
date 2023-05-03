#![feature(box_patterns)]
#![feature(path_file_prefix)]

use std::sync::Arc;

use deps_analyzer::DepsAnalyzer;
use farmfe_core::{
  config::{Config, TargetEnv, FARM_GLOBAL_THIS, FARM_MODULE_SYSTEM},
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{ModuleId, ModuleMetaData, ModuleSystem, ModuleType, ScriptModuleMetaData},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeModuleHookParam, PluginHookContext,
    PluginLoadHookParam, PluginLoadHookResult, PluginParseHookParam, PluginProcessModuleHookParam,
  },
  relative_path::RelativePath,
  resource::{
    resource_pot::{self, ResourcePot, ResourcePotType},
    Resource, ResourceType,
  },
  swc_common::{comments::NoopComments, FilePathMapping, Mark, SourceMap, GLOBALS},
  swc_ecma_ast::{
    CallExpr, Callee, Expr, ExprStmt, Ident, MemberExpr, MemberProp, ModuleDecl, ModuleItem, Stmt,
  },
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::{
  fs::{read_file_utf8, transform_output_filename},
  script::{
    codegen_module, module_system_from_deps, module_type_from_id, parse_module,
    swc_try_with::try_with, syntax_from_module_type,
  },
  sourcemap::swc_gen::{build_source_map, AstModule},
  swc_ecma_minifier::{
    optimize,
    option::{ExtraOptions, MinifyOptions},
  },
  swc_ecma_transforms::{
    fixer::fixer,
    resolver,
    typescript::{strip, strip_with_jsx},
  },
  swc_ecma_visit::FoldWith,
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
                Default::default(),
                NoopComments, // TODO parse comments
                top_level_mark,
              ));
            }
            _ => {}
          }
        },
      )?;

      return Ok(Some(()));
    }

    Ok(None)
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

    param.module.module_type = ModuleType::Js;

    if param.deps.len() > 0 {
      let module_system =
        module_system_from_deps(param.deps.iter().map(|d| d.kind.clone()).collect());
      param.module.meta.as_script_mut().module_system = module_system;
    } else {
      // default to commonjs
      param.module.meta.as_script_mut().module_system = ModuleSystem::CommonJs;

      let ast = &param.module.meta.as_script().ast;

      // if the ast contains ModuleDecl, it's a esm module
      for item in ast.body.iter() {
        if let ModuleItem::ModuleDecl(_) = item {
          param.module.meta.as_script_mut().module_system = ModuleSystem::EsModule;
          break;
        }
      }
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
      // handle js entry resource pot
      self.handle_entry_resource_pot(resource_pot, context)?;

      let ast = &resource_pot.meta.as_js().ast;
      let mut src_map_buf = vec![];

      let mut buf = codegen_module(
        ast,
        context.config.script.target.clone(),
        context.meta.script.cm.clone(),
        Some(&mut src_map_buf),
        context.config.minify,
      )
      .map_err(|e| CompilationError::GenerateResourcesError {
        name: resource_pot.id.to_string(),
        ty: resource_pot.resource_pot_type.clone(),
        source: Some(Box::new(e)),
      })?;

      let filename = resource_pot
        .entry_module
        .as_ref()
        .map(|module_id| {
          let entry_filename =
            RelativePath::new(&module_id.relative_path().to_string()).normalize();
          let entry_name = context
            .config
            .input
            .iter()
            .find(|(_, val)| RelativePath::new(val).normalize() == entry_filename);

          if let Some((entry_name, _)) = entry_name {
            entry_name.to_string()
          } else {
            resource_pot.id.to_string()
          }
        })
        .unwrap_or(resource_pot.id.to_string());

      let sourcemap_filename = transform_output_filename(
        context.config.output.filename.clone(),
        &filename,
        &buf,
        &ResourceType::SourceMap.to_ext(),
      );

      if context.config.sourcemap.enabled()
        && (context.config.sourcemap.is_all() || !resource_pot.immutable)
      {
        // TODO: support inline sourcemap
        let source_mapping_url = format!("\n//# sourceMappingURL={}", sourcemap_filename);
        buf.append(&mut source_mapping_url.as_bytes().to_vec());
      }

      let mut resources = vec![Resource {
        bytes: buf,
        name: filename,
        emitted: false,
        resource_type: ResourceType::Js,
        resource_pot: resource_pot.id.clone(),
        preserve_name: false,
      }];

      if context.config.sourcemap.enabled()
        && (context.config.sourcemap.is_all() || !resource_pot.immutable)
      {
        let src_map = build_source_map(
          &src_map_buf,
          context.meta.script.cm.clone(),
          AstModule::Script(ast),
        );

        resources.push(Resource {
          bytes: src_map,
          name: sourcemap_filename,
          emitted: false,
          resource_type: ResourceType::SourceMap,
          resource_pot: resource_pot.id.clone(),
          preserve_name: true,
        });
      }

      Ok(Some(resources))
    } else {
      Ok(None)
    }
  }
}

impl FarmPluginScript {
  pub fn new(_config: &Config) -> Self {
    Self {}
  }

  pub fn get_export_info_of_entry_module(
    &self,
    entry_module_id: &ModuleId,
    context: &Arc<CompilationContext>,
  ) -> Vec<String> {
    let module_graph = context.module_graph.read();
    let entry_module = module_graph
      .module(entry_module_id)
      .expect("entry module is not found");

    let ast = &entry_module.meta.as_script().ast;
    let mut export_info = vec![];

    for item in ast.body.iter() {
      match item {
        ModuleItem::ModuleDecl(module_decl) => match module_decl {
          // TODO: support more export syntax

          // ModuleDecl::ExportDecl(export_decl) => {
          //   if let Decl::Class(class_decl) = &export_decl.decl {
          //     export_info.push(class_decl.ident.sym.to_string());
          //   }
          // }
          // ModuleDecl::ExportNamed(named_export) => {
          //   for specifier in named_export.specifiers.iter() {
          //     match specifier {
          //       ExportSpecifier::Named(named_specifier) => {
          //         export_info.push(named_specifier.orig.sym.to_string());
          //       }
          //       _ => {}
          //     }
          //   }
          // }
          ModuleDecl::ExportDefaultDecl(_) | ModuleDecl::ExportDefaultExpr(_) => {
            export_info.push("default".to_string());
          }
          _ => {}
        },
        _ => {}
      }
    }

    export_info
  }

  fn handle_entry_resource_pot(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<()> {
    if let Some(entry_module_id) = &resource_pot.entry_module {
      // modify the ast according to the type,
      // if js, insert the runtime ast in the front
      match resource_pot.resource_pot_type {
        ResourcePotType::Js => {
          let runtime_ast = context.meta.script.runtime_ast.read();
          let runtime_ast = runtime_ast.as_ref().unwrap_or_else(|| {
            panic!(
              "runtime ast is not found when generating resources for {:?}",
              resource_pot.id
            )
          });

          let resource_pot_ast = &mut resource_pot.meta.as_js_mut().ast;
          resource_pot_ast
            .body
            .insert(0, runtime_ast.body.to_vec().remove(0));

          let export_info = self.get_export_info_of_entry_module(entry_module_id, context);
          let export_str = if context.config.output.target_env == TargetEnv::Node {
            export_info
              .iter()
              .map(|export| {
                if export == "default" {
                  "export default entry.default;".to_string()
                } else {
                  format!("export {{ {}: entry.{} }};", export, export)
                }
              })
              .collect::<Vec<String>>()
              .join("\n")
          } else {
            "".to_string()
          };

          // TODO support top level await, and only support reexport default export now, should support more export type in the future
          // call the entry module
          let call_entry = parse_module(
            "farm-internal-call-entry-module",
            &format!(
              r#"var {} = globalThis || window || global || self;
                var farmModuleSystem = {}.{};
                farmModuleSystem.bootstrap();
                var entry = farmModuleSystem.require("{}");
                {}"#,
              FARM_GLOBAL_THIS,
              FARM_GLOBAL_THIS,
              FARM_MODULE_SYSTEM,
              entry_module_id.id(context.config.mode.clone()),
              export_str
            ),
            Syntax::Es(context.config.script.parser.es_config.clone()),
            context.config.script.target.clone(),
            context.meta.script.cm.clone(),
          )?;
          // insert node specific code.
          // TODO: support async module for node, using dynamic require to load external module instead of createRequire. createRequire does not support load ESM module.
          if context.config.output.target_env == TargetEnv::Node {
            let global_var = parse_module(
              "farm-global-var",
              r#"import module from 'node:module';
                global.__farmNodeRequire = module.createRequire(import.meta.url);
                global.__farmNodeBuiltinModules = module.builtinModules;"#,
              Syntax::Es(context.config.script.parser.es_config.clone()),
              context.config.script.target.clone(),
              context.meta.script.cm.clone(),
            )?;
            resource_pot_ast.body.splice(0..0, global_var.body);
          }

          resource_pot_ast.body.extend(call_entry.body);
        }
        _ => unreachable!("Entry resource pot should be js type in FarmPluginScript"),
      }
    }

    Ok(())
  }
}
