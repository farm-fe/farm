#![deny(clippy::all)]
use farmfe_core::{
  config::{Config, ResolveConfig, config_regex::ConfigRegex},
  context::{CompilationContext, EmitFileParams},
  error::CompilationError,
  plugin::{Plugin, PluginProcessModuleHookParam, ResolveKind},
  resource::ResourceType,
  stats::Stats,
  swc_common::{FileName, Mark},
  swc_ecma_ast::{ImportDecl, Module as EcmaAstModule, ModuleDecl, ModuleItem, Program},
};
use farmfe_plugin_resolve::resolver::{ResolveOptions, Resolver};

use farmfe_toolkit::{
  plugin_utils::path_filter::PathFilter,
  swc_ecma_codegen::to_code,
  swc_ecma_visit::{VisitMut, VisitMutWith},
  swc_typescript::fast_dts::FastDts,
};
use std::time::Duration;
use std::{
  path::PathBuf,
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

    let meta = &mut param.meta.as_script_mut();

    let mut module: EcmaAstModule = meta.ast.clone();

    emit_type_dependencies(&module, param.module_id, &self.options, context)?;

    module.visit_mut_with(&mut ImportPathRewriter {
      config: (*context.config).clone(),
      resolver: Resolver::new(),
    });
    let filename: Arc<FileName> = Arc::new(FileName::Real(
      param.module_id.relative_path().to_string().into(),
    ));

    let unresolved_mark = Mark::from_u32(meta.unresolved_mark);
    let mut checker = FastDts::new(filename.clone(), unresolved_mark, Default::default());
    module.visit_mut_with(&mut ImportVariableRemover);

    let mut program = Program::Module(module);
    let issues = checker.transform(&mut program);

    for issue in issues {
      let _range = issue.range;
    }

    let dts_path = if path.ends_with(".tsx") {
      path.replace(".tsx", ".d.ts")
    } else {
      path.replace(".ts", ".d.ts")
    };

    let dts_code = to_code(&program.expect_module());
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

fn emit_type_dependencies(
  entry: &EcmaAstModule,
  owner: &farmfe_core::module::ModuleId,
  options: &FarmPluginDtsOptions,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  use farmfe_core::{
    module::ModuleId,
    swc_common::{GLOBALS, Globals},
    swc_ecma_parser::{Syntax, TsSyntax},
  };
  use farmfe_toolkit::script::{parse_module, swc_try_with::resolve_module_mark};
  let entry_path = PathBuf::from(owner.resolved_path(&context.config.root));
  let mut visited =
    std::collections::HashSet::from([entry_path.canonicalize().unwrap_or(entry_path)]);
  let mut queue = vec![(owner.clone(), entry.clone())];
  let resolver = Resolver::new();
  while let Some((importer, module)) = queue.pop() {
    for item in &module.body {
      let source = match item {
        ModuleItem::ModuleDecl(ModuleDecl::Import(import)) => Some(&import.src),
        ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(export)) => export.src.as_ref(),
        ModuleItem::ModuleDecl(ModuleDecl::ExportAll(export)) => Some(&export.src),
        _ => None,
      };
      let Some(source) = source.and_then(|source| source.value.as_str()) else {
        continue;
      };
      let base = PathBuf::from(importer.resolved_path(&context.config.root));
      let Some(base) = base.parent() else { continue };
      let resolved = resolver.resolve(
        source,
        base.to_path_buf(),
        &ResolveKind::Import,
        &ResolveOptions::default(),
        context,
      );
      let path = resolved
        .map(|resolved| PathBuf::from(resolved.resolved_path))
        .or_else(|| {
          // NodeNext source imports use the eventual .js suffix for TypeScript.
          if !source.starts_with('.') {
            return None;
          }
          let candidate = base.join(source);
          [
            candidate.with_extension("ts"),
            candidate.with_extension("tsx"),
          ]
          .into_iter()
          .find(|path| path.is_file())
        });
      let Some(path) = path else { continue };
      if !matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts" | "tsx")
      ) {
        continue;
      }
      let path = path.to_string_lossy().to_string();
      let id = ModuleId::new(&path, "", &context.config.root);
      if !PathFilter::new(&options.include, &options.exclude).execute(id.relative_path())
        || id.relative_path().starts_with("../")
      {
        continue;
      }
      let identity = std::fs::canonicalize(&path).map_err(|error| {
        CompilationError::GenericError(format!("DTS dependency identity {path}: {error}"))
      })?;
      if !visited.insert(identity) {
        continue;
      }
      let content = std::fs::read_to_string(&path).map_err(|error| {
        CompilationError::GenericError(format!("DTS dependency {path}: {error}"))
      })?;
      let mut ast = parse_module(
        &id,
        Arc::new(content),
        Syntax::Typescript(TsSyntax {
          tsx: path.ends_with(".tsx"),
          ..Default::default()
        }),
        context.config.script.target,
      )?
      .ast;
      queue.push((id.clone(), ast.clone()));
      context.add_watch_files(owner.clone(), vec![id.clone()])?;
      let globals = Globals::new();
      let (unresolved_mark, _) = resolve_module_mark(&mut ast, true, &globals);
      let filename = Arc::new(FileName::Real(path.into()));
      let mut program = Program::Module(ast);
      GLOBALS.set(&globals, || {
        program.visit_mut_with(&mut ImportVariableRemover);
        FastDts::new(filename, unresolved_mark, Default::default()).transform(&mut program);
      });
      let name = PathBuf::from(id.relative_path())
        .with_extension("d.ts")
        .to_string_lossy()
        .replace('\\', "/");
      context.emit_file(EmitFileParams {
        resolved_path: owner.to_string(),
        name,
        content: to_code(&program.expect_module()).into_bytes(),
        resource_type: ResourceType::Custom("d.ts".into()),
      });
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests;

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
          src.value.as_str().expect("non-UTF-8 string literal"),
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
          src.raw = Some(format!("'{final_value}'").into());
        }
      }
    }

    items.visit_mut_children_with(self);
  }
}
