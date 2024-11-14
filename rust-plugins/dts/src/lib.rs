#![deny(clippy::all)]
use farmfe_toolkit::swc_ecma_transforms::{helpers::inject_helpers, typescript::tsx};

use farmfe_core::{
  config::Config,
  plugin::Plugin,
  swc_common::{comments::SingleThreadedComments, Mark},
  swc_ecma_ast::{Module as EcmaAstModule, ModuleItem},
  swc_ecma_parser::{lexer::Lexer, EsSyntax as EsConfig, Parser, StringInput, Syntax},
};

use farmfe_macro_plugin::farm_plugin;

#[derive(serde::Deserialize)]
pub struct DtsOptions {
  pub my_option: Option<String>,
}

#[farm_plugin]
pub struct FarmPluginDts {}

impl FarmPluginDts {
  fn new(config: &Config, options: String) -> Self {
    let opts: DtsOptions = serde_json::from_str(&options).unwrap();
    Self {}
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
}
