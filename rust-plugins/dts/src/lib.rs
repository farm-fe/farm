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
  swc_common::{comments::SingleThreadedComments, Mark},
  swc_ecma_ast::{Module as EcmaAstModule, ModuleItem},
  swc_ecma_parser::{lexer::Lexer, EsSyntax as EsConfig, Parser, StringInput, Syntax},
};
use farmfe_toolkit::swc_ecma_transforms::{helpers::inject_helpers, typescript::tsx};
use farmfe_toolkit::{
  common::PathFilter,
  swc_ecma_visit::{VisitMut, VisitMutWith},
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

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    // println!("resolve {:?} from {:?}", param.source, param.importer);
    Ok(None)
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if param.resolved_path.ends_with(".farm-runtime") {
      return Ok(None);
    }
    // println!("load path: {:#?}", param);
    Ok(None)
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    // module type guard is neccessary
    // return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
    //   content: compile_result.css,
    //   source_map: compile_result.source_map,
    //   // tell farm compiler that we have transformed this module to css
    //   module_type: Some(farmfe_core::module::ModuleType::Css),
    //   ignore_previous_source_map: false,
    // }));
    if param.resolved_path.ends_with(".farm-runtime") {
      return Ok(None);
    }
    // let transform_options = oxc_transformer::TransformOptions::default();
    // println!("transform_options : {:#?}", transform_options);
    println!("transform path: {:#?}", param);

    Ok(None)
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
    println!("param.module_id: {:#?}", param.module_id.relative_path());
    println!("param.content: {:#?}", param.content);
    Ok(Some(()))
  }
}
