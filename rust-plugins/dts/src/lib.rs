#![deny(clippy::all)]
use farmfe_core::{
  config::{config_regex::ConfigRegex, Config},
  context::CompilationContext,
  error::CompilationError,
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeModuleHookParam,
    PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam, PluginProcessModuleHookParam,
  },
  swc_common::syntax_pos::FileName,
  swc_common::{comments::SingleThreadedComments, BytePos, Mark},
  swc_ecma_ast::{Module as EcmaAstModule, ModuleItem, Program},
  swc_ecma_parser::{lexer::Lexer, EsSyntax as EsConfig, JscTarget, Parser, StringInput, Syntax},
};
use farmfe_toolkit::swc_ecma_transforms::{helpers::inject_helpers, typescript::tsx};
use farmfe_toolkit::{
  common::PathFilter,
  swc_ecma_codegen::{to_code_with_comments, Node},
  swc_ecma_visit::{VisitMut, VisitMutWith},
  swc_typescript::fast_dts::FastDts,
};
use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use farmfe_macro_plugin::farm_plugin;

#[farm_plugin]
pub struct FarmPluginDts {
  options: FarmPluginDtsOptions,
}

#[derive(serde::Deserialize)]
pub struct FarmPluginDtsOptions {
  exclude: Vec<ConfigRegex>,
  include: Vec<ConfigRegex>,
}

impl Default for FarmPluginDtsOptions {
  fn default() -> Self {
    Self {
      exclude: vec![ConfigRegex::new("node_modules/")],
      include: vec![],
    }
  }
}

impl FarmPluginDts {
  fn new(_: &Config, options: String) -> Self {
    let options: FarmPluginDtsOptions = serde_json::from_str(&options).unwrap_or_default();
    Self { options }
  }
}

impl Plugin for FarmPluginDts {
  fn name(&self) -> &str {
    "FarmPluginDts"
  }

  fn priority(&self) -> i32 {
    101
  }

  fn process_module(
    &self,
    param: &mut PluginProcessModuleHookParam,
    _: &Arc<CompilationContext>,
  ) -> Result<Option<()>, CompilationError> {
    let filter = PathFilter::new(&self.options.include, &self.options.exclude);
    if param.module_id.relative_path().ends_with(".farm-runtime") {
      return Ok(None);
    }
    if !filter.execute(param.module_id.relative_path())
      && param.module_id.relative_path().ends_with(".farm-runtime")
    {
      return Ok(None);
    }
    let ast: &mut EcmaAstModule = &mut param.meta.as_script_mut().ast;
    let comments = SingleThreadedComments::default();
    let fm = Arc::new(FileName::Real(param.module_id.relative_path().into()));
    let mut checker = FastDts::new(fm.clone());
    let mut program = Program::Module(ast.clone());

    let issues = checker.transform(&mut program);
    for issue in issues {
      let range = issue.range();
      println!("DTS Issue: {:?}", issue.to_string());
    }

    let dts_code = to_code_with_comments(Some(&comments), &program);
    println!("Generated DTS: {:?}", dts_code);

    Ok(Some(()))
  }
}
