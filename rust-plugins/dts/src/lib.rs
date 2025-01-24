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
  stats::Stats,
  swc_common::{comments::SingleThreadedComments, BytePos, FileName, Mark},
  swc_ecma_ast::{Module as EcmaAstModule, Program},
  swc_ecma_parser::{lexer::Lexer, EsSyntax as EsConfig, JscTarget, Parser, StringInput, Syntax},
  swc_typescript::fast_dts::FastDts,
};
use farmfe_toolkit::{
  common::PathFilter,
  swc_ecma_codegen::{to_code_with_comments, Node},
  swc_ecma_transforms::{helpers::inject_helpers, typescript},
  swc_ecma_visit::{VisitMut, VisitMutWith},
};
use std::time::Duration;
use std::{
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use farmfe_macro_plugin::farm_plugin;

#[farm_plugin]
pub struct FarmPluginDts {
  options: FarmPluginDtsOptions,
  total_dts_time: Mutex<Duration>,
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
    Self {
      options,
      total_dts_time: Mutex::new(Duration::from_secs(0)),
    }
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
    let path = param.module_id.relative_path();
    if !path.ends_with(".ts") && !path.ends_with(".tsx") {
      return Ok(None);
    }
    let ast = &mut param.meta.as_script_mut().ast;
    let comments = SingleThreadedComments::default();

    // 克隆 AST
    let mut module = ast.clone();
    let filename = Arc::new(FileName::Real(
      param.module_id.relative_path().to_string().into(),
    ));
    // println!("filename: {:?}", filename);
    let start = std::time::Instant::now();

    let mut checker = FastDts::new(filename);
    let issues = checker.transform(&mut module);
    // 生成 d.ts 代码
    for issue in issues {
      let range = issue.range();
      // println!("DTS Issue: {:?}", issue.to_string());
    }
    let dts_code = to_code_with_comments(Some(&comments), &module);
    println!("dts_code: {:?}", start.elapsed());
    *self.total_dts_time.lock().unwrap() += start.elapsed();
    // println!("Original AST: {:#?}", ast); // 打印原始 AST
    // println!("Transformed AST: {:#?}", module); // 打印转换后的 AST
    // println!("DTS Code: {}", dts_code); // 打印生成的代码

    Ok(Some(()))
  }

  fn finish(
    &self,
    _stat: &Stats,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>, CompilationError> {
    let total_time = *self.total_dts_time.lock().unwrap();
    println!(
      "Total DTS generation time: {:?} ms",
      total_time.as_secs_f64() * 1000.0
    );
    Ok(None)
  }
}
