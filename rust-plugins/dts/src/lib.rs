#![deny(clippy::all)]
use farmfe_core::{
  config::{config_regex::ConfigRegex, Config},
  context::{CompilationContext, EmitFileParams},
  error::CompilationError,
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeModuleHookParam,
    PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam, PluginProcessModuleHookParam,
  },
  resource::{Resource, ResourceOrigin, ResourceType},
  stats::Stats,
  swc_common::{comments::SingleThreadedComments, BytePos, FileName, Mark},
  swc_ecma_ast::{ImportDecl, Module as EcmaAstModule, ModuleDecl, ModuleItem, Program},
  swc_ecma_parser::{lexer::Lexer, EsSyntax as EsConfig, JscTarget, Parser, StringInput, Syntax},
  swc_typescript::fast_dts::FastDts,
};
use farmfe_toolkit::{
  common::PathFilter,
  swc_ecma_codegen::{to_code, Node},
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
}

impl VisitMut for ImportPathRewriter {
  fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
    for item in items.iter_mut() {
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
        let src = &mut import.src;
        if src.value.starts_with('.') {
          let import_path = PathBuf::from(&*src.value);
          let source_dir = self.source_path.parent().unwrap_or_else(|| Path::new(""));

          let full_path = source_dir.join(&import_path);

          let new_path = if full_path.to_string_lossy().ends_with(".ts") {
            full_path.with_extension("d.ts")
          } else if full_path.to_string_lossy().ends_with(".tsx") {
            PathBuf::from(full_path.to_string_lossy().replace(".tsx", ".d.ts"))
          } else {
            PathBuf::from(format!("{}.d.ts", full_path.to_string_lossy()))
          };

          if let Some(rel_path) = pathdiff::diff_paths(&new_path, source_dir) {
            let new_value = rel_path.to_string_lossy().replace('\\', "/");

            src.value = if !new_value.ends_with(".d.ts") {
              format!("{}.d.ts", new_value).into()
            } else {
              new_value.into()
            };
          }
        }
      }
    }

    items.visit_mut_children_with(self);
  }
}
