use std::sync::Arc;

use deps_analyzer::DepsAnalyzer;
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{Module, ModuleId, ModuleMetaData, ScriptModuleMetaData},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam,
  },
  resource::{
    resource_pot::{ResourcePot, ResourcePotType},
    Resource, ResourceType,
  },
  swc_common::{Mark, GLOBALS},
};
use farmfe_toolkit::{
  fs::read_file_utf8,
  script::{codegen_module, module_type_from_id, parse_module, syntax_from_module_type},
  swc_ecma_transforms::{resolver, typescript::strip},
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
    let id = param.id;

    let module_type = module_type_from_id(id);

    if module_type.is_script() {
      let content = read_file_utf8(id)?;

      Ok(Some(PluginLoadHookResult {
        content,
        module_type,
      }))
    } else {
      Ok(None)
    }
  }

  fn parse(
    &self,
    param: &PluginParseHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<Module>> {
    if let Some(syntax) = syntax_from_module_type(&param.module_type) {
      let swc_module = parse_module(
        &param.id,
        &param.content,
        syntax.clone(),
        context.meta.script.cm.clone(),
      )?;

      let mut module = Module::new(
        ModuleId::new(&param.id, &context.config.root),
        param.module_type.clone(),
      );

      let meta = ScriptModuleMetaData { ast: swc_module };
      module.meta = ModuleMetaData::Script(meta);

      Ok(Some(module))
    } else {
      Ok(None)
    }
  }

  fn analyze_deps(
    &self,
    param: &mut PluginAnalyzeDepsHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    let module = param.module;

    if module.module_type.is_script() {
      let module_ast = &module.meta.as_script().ast;
      let mut analyzer = DepsAnalyzer::new(module_ast);

      param.deps.extend(analyzer.analyze_deps());
      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn process_module(
    &self,
    module: &mut Module,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if module.module_type.is_typescript() {
      GLOBALS.set(&context.meta.script.globals, || {
        let top_level_mark = Mark::new();
        let unresolved_mark = Mark::new();

        let ast = &mut module.meta.as_script_mut().ast;
        ast.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, true));
        ast.visit_mut_with(&mut strip(top_level_mark));
      });
    }

    Ok(Some(()))
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<Vec<Resource>>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Js) {
      let ast = &resource_pot.meta.as_js().ast;
      let buf = codegen_module(ast, context.meta.script.cm.clone()).map_err(|e| {
        CompilationError::GenerateResourcesError {
          name: resource_pot.id.to_string(),
          ty: resource_pot.resource_pot_type.clone(),
          source: Some(Box::new(e)),
        }
      })?;

      Ok(Some(vec![Resource {
        bytes: buf,
        name: resource_pot.id.to_string() + ".js",
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
}
