#![deny(clippy::all)]
use farmfe_core::{
  config::{config_regex::ConfigRegex, Config, ResolveConfig},
  context::{CompilationContext, EmitFileParams},
  error::CompilationError,
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeModuleHookParam,
    PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam, PluginProcessModuleHookParam, ResolveKind,
  },
  resource::{Resource, ResourceOrigin, ResourceType},
  stats::Stats,
  swc_common::{comments::SingleThreadedComments, BytePos, FileName, Mark},
  swc_ecma_ast::{ImportDecl, Module as EcmaAstModule, ModuleDecl, ModuleItem, Program},
  swc_ecma_parser::{lexer::Lexer, EsSyntax as EsConfig, JscTarget, Parser, StringInput, Syntax},
};
use farmfe_plugin_resolve::resolver::{ResolveOptions, Resolver};

use farmfe_toolkit::{
  sourcemap::PathFilter,
  swc_ecma_codegen::{to_code, Node},
  swc_ecma_transforms::{helpers::inject_helpers, typescript},
  swc_ecma_visit::{VisitMut, VisitMutWith},
  swc_typescript::fast_dts::FastDts,
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
      include: vec![ConfigRegex::new(".(ts|tsx)$")],
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
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>, CompilationError> {
    let filter = PathFilter::new(&self.options.include, &self.options.exclude);
    if !filter.execute(param.module_id.relative_path()) {
      return Ok(None);
    }
    let path = param.module_id.relative_path();
    let start = std::time::Instant::now();

    let ast = &mut param.meta.as_script_mut().ast;

    let mut module: EcmaAstModule = ast.clone();

    module.visit_mut_with(&mut ImportPathRewriter {
      source_path: PathBuf::from(path),
      config: (*context.config).clone(),
      resolver: Resolver::new(),
    });
    let filename: Arc<FileName> = Arc::new(FileName::Real(
      param.module_id.relative_path().to_string().into(),
    ));

    let mut checker = FastDts::new(filename.clone());
    module.visit_mut_with(&mut ImportVariableRemover);
    let issues = checker.transform(&mut module);
    for issue in issues {
      let _range = issue.range();
    }

    let dts_path = if path.ends_with(".tsx") {
      path.replace(".tsx", ".d.ts")
    } else {
      path.replace(".ts", ".d.ts")
    };

    let dts_code = to_code(&module);
    *self.total_dts_time.lock().unwrap() += start.elapsed();

    context.emit_file(EmitFileParams {
      resolved_path: param.module_id.to_string(),
      name: dts_path,
      content: dts_code.as_bytes().to_vec(),
      resource_type: ResourceType::Custom("d.ts".to_string()),
    });
    Ok(Some(()))
  }

  fn finish(
    &self,
    _stat: &Stats,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>, CompilationError> {
    let total_time = *self.total_dts_time.lock().unwrap();
    println!(
      "\x1b[1m\x1b[38;2;113;26;95m[ Farm ]\x1b[39m\x1b[0m Dts Plugin Build completed in: \x1b[1m\x1b[32m{:.2}ms\x1b[0m",
      total_time.as_secs_f64() * 1000.0
    );
    Ok(None)
  }
}

struct ImportVariableRemover;

impl VisitMut for ImportVariableRemover {
  fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
    items.retain(|item| {
      !matches!(
        item,
        ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
          type_only: false,
          ..
        }))
      )
    });
    items.visit_mut_children_with(self);
  }
}

struct ImportPathRewriter {
  source_path: PathBuf,
  config: Config,
  resolver: Resolver,
}

// TODO 生成后缀
// 主要依据：根据 Rollup 的 outputOptions.entryFileNames 配置
// 如果输出是 .js -> 生成 .d.ts
// 如果输出是 .cjs -> 生成 .d.cts
// 如果输出是 .mjs -> 生成 .d.mts

// {
//   output: {
//     entryFileNames: '[name].cjs'  // 将生成 .d.cts
//     // 或
//     entryFileNames: '[name].mjs'  // 将生成 .d.mts
//     // 或
//     entryFileNames: '[name].js'   // 将生成 .d.ts
//   }
// }

// 然后这个先不跟 format 走吧 format 未来可能会有问题 还是跟 entryFileNames 走吧

// TODO emit_file baseDir
// extraOutdir
// 默认全放在根目录

// 如果设置了 extraOutdir 则需要根据 extraOutdir 来生成后缀

impl VisitMut for ImportPathRewriter {
  fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
    for item in items.iter_mut() {
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
        let src = &mut import.src;
        let alias_context = CompilationContext::new(
          Config {
            resolve: Box::new(ResolveConfig {
              alias: self.config.resolve.alias.clone(),
              ..Default::default()
            }),
            ..Default::default()
          },
          vec![],
        )
        .unwrap();
        let resolved = self.resolver.resolve(
          src.value.as_str(),
          PathBuf::from(self.config.root.clone()),
          &ResolveKind::Import,
          &ResolveOptions::default(),
          &Arc::new(alias_context),
        );
        if let Some(resolved_path) = resolved {
          let path = PathBuf::from(resolved_path.resolved_path);

          let base_path = path.with_extension("");
          let final_value = base_path
            .with_extension("d.ts")
            .to_string_lossy()
            .to_string();
          src.value = final_value.clone().into();
          src.raw = Some(format!("'{}'", final_value).into());
        }
      }
    }

    items.visit_mut_children_with(self);
  }
}
