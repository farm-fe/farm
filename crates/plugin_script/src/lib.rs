use std::{path::PathBuf, sync::Arc};

use deps_analyzer::DepsAnalyzer;
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{Module, ModuleId, ModuleMetaData, ModuleScriptMetaData, ModuleType},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginLoadHookParam, PluginLoadHookResult,
    PluginParseHookParam,
  },
  resource::{
    resource_pot::{ResourcePot, ResourcePotType},
    Resource,
  },
  swc_common::FileName,
};
use swc_ecma_codegen::{
  text_writer::{JsWriter, WriteJs},
  Emitter, Node,
};
use swc_ecma_parser::{EsConfig, Parser, StringInput, Syntax, TsConfig};

mod deps_analyzer;
/// ScriptPlugin is used to support compiling js/ts/jsx/tsx/... files, support loading, parse, analyze dependencies and code generation.
/// Note that we do not do transforms here, the transforms (e.g. strip types, jsx...) are handled in a separate plugin (farmfe_plugin_swc_transforms).
pub struct FarmScriptPlugin {}

impl Plugin for FarmScriptPlugin {
  fn name(&self) -> &str {
    "FarmScriptPlugin"
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginLoadHookResult>> {
    let id = param.id;

    // only deal with known extension
    if let Some(module_type) = self.module_type_from_id(id) {
      let raw =
        std::fs::read(id).map_err(|e| CompilationError::GenericError(format!("{:?}", e)))?;
      let content = String::from_utf8(raw).map_err(|e| {
        CompilationError::GenericError(format!(
          "File `{}` is not utf8! Detailed Error: {:?}",
          id, e
        ))
      })?;

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
  ) -> Result<Option<Module>> {
    if let Some(syntax) = self.syntax_from_module_type(&param.module_type) {
      let source_file = context.meta.script.cm.new_source_file(
        FileName::Real(PathBuf::from(&param.id)),
        param.content.clone(),
      );
      let input = StringInput::from(&*source_file);
      // TODO support parsing comments
      let mut parser = Parser::new(syntax, input, None);
      let swc_module = parser
        .parse_module()
        .map_err(|e| CompilationError::ParseError {
          id: param.id.clone(),
          source: Some(Box::new(CompilationError::GenericError(format!("{:?}", e))) as _),
        })?;

      let mut module = Module::new(
        ModuleId::new(&param.id, &context.config.root),
        param.module_type.clone(),
      );
      let meta = ModuleScriptMetaData { ast: swc_module };
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
    let module_ast = &module.meta.as_script().ast;
    let mut analyzer = DepsAnalyzer::new(module_ast);

    param.deps.extend(analyzer.analyze_deps());

    Ok(Some(()))
  }

  fn generate_resources(
    &self,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<Vec<Resource>>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Js) {
      let ast = &resource_pot.meta.as_js().ast;
      let mut buf = vec![];

      {
        // TODO support source map
        let wr = Box::new(JsWriter::new(
          context.meta.script.cm.clone(),
          "\n",
          &mut buf,
          None,
        )) as Box<dyn WriteJs>;

        let mut emitter = Emitter {
          cfg: swc_ecma_codegen::Config {
            target: Default::default(),
            ascii_only: false,
            minify: false,
          },
          comments: None,
          cm: context.meta.script.cm.clone(),
          wr,
        };

        ast
          .emit_with(&mut emitter)
          .map_err(|_| CompilationError::GenerateResourcesError {
            name: resource_pot.name.clone(),
            ty: resource_pot.resource_pot_type.clone(),
          })?;
      }

      Ok(Some(vec![Resource { bytes: buf }]))
    } else {
      Ok(None)
    }
  }
}

impl FarmScriptPlugin {
  pub fn new(config: &Config) -> Self {
    Self {}
  }

  /// Get [ModuleType] from the resolved id's extension, return [None] if the extension is not supported
  /// TODO support configuring parser config and extra extension
  fn module_type_from_id(&self, id: &str) -> Option<ModuleType> {
    if id.ends_with(".ts") {
      Some(ModuleType::Ts)
    } else if id.ends_with(".tsx") {
      Some(ModuleType::Tsx)
    } else if id.ends_with(".js") || id.ends_with(".mjs") || id.ends_with(".cjs") {
      Some(ModuleType::Js)
    } else if id.ends_with(".jsx") {
      Some(ModuleType::Jsx)
    } else {
      None
    }
  }

  /// TODO support custom [EsConfig] and [TsConfig]
  fn syntax_from_module_type(&self, module_type: &ModuleType) -> Option<Syntax> {
    match module_type {
      ModuleType::Js => Some(Syntax::Es(Default::default())),
      ModuleType::Jsx => Some(Syntax::Es(EsConfig {
        jsx: true,
        ..Default::default()
      })),
      ModuleType::Ts => Some(Syntax::Typescript(Default::default())),
      ModuleType::Tsx => Some(Syntax::Typescript(TsConfig {
        tsx: true,
        ..Default::default()
      })),
      _ => None,
    }
  }
}
