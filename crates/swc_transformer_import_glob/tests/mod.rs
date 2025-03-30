use std::{collections::HashMap, path::Path, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::{ModuleId, VIRTUAL_MODULE_PREFIX},
  swc_common::SourceMap,
  swc_ecma_ast::EsVersion,
  swc_ecma_parser::Syntax,
};
use farmfe_swc_transformer_import_glob::{
  transform_import_meta_glob, ImportGlobVisitor, ImportMetaGlobResolver,
  ImportMetaGlobResolverParams,
};
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::script::{codegen_module, parse_module, ParseScriptModuleResult};

use farmfe_core::plugin::{PluginResolveHookParam, ResolveKind};

struct ImportMetaGlobResolverImpl {
  context: Arc<CompilationContext>,
}

impl ImportMetaGlobResolver for ImportMetaGlobResolverImpl {
  fn resolve(&self, params: ImportMetaGlobResolverParams) -> Option<String> {
    self
      .context
      .plugin_driver
      .resolve(
        &PluginResolveHookParam {
          source: params.source,
          importer: Some(params.importer),
          kind: ResolveKind::Import,
        },
        &self.context,
        &Default::default(),
      )
      .ok()
      .flatten()
      .map(|v| v.resolved_path)
  }
}

fn create_context() -> Arc<CompilationContext> {
  let mut compilation = CompilationContext::default();
  compilation.config.root = if cfg!(windows) {
    "C:\\root1".to_string()
  } else {
    "/root1/root2".to_string()
  };
  Arc::new(compilation)
}

fn cur_dir_by_importer<'a>(importer: &'a ModuleId, context: &'a Arc<CompilationContext>) -> String {
  let resolved_path = importer.resolved_path(&context.config.root);

  if resolved_path.starts_with(VIRTUAL_MODULE_PREFIX) {
    context.config.root.clone()
  } else {
    Path::new(&resolved_path)
      .parent()
      .unwrap()
      .to_string_lossy()
      .to_string()
  }
}

#[inline]
fn create_visitor<'a>(
  importer: &'a ModuleId,
  context: &'a Arc<CompilationContext>,
) -> ImportGlobVisitor<'a, ImportMetaGlobResolverImpl> {
  let cur_dir = cur_dir_by_importer(importer, context);
  let visitor = ImportGlobVisitor::new(
    importer,
    context.config.root.clone(),
    cur_dir,
    &context.config.resolve.alias,
    ImportMetaGlobResolverImpl {
      context: Arc::clone(context),
    },
  );

  visitor
}

#[test]
fn find_rel_source_absolute_path() {
  let context = create_context();
  let importer = ModuleId::new("src/index.js", "", &context.config.root);
  let visitor = create_visitor(&importer, &context);

  let (_, s1) = visitor.find_rel_source("/src/foo.js");

  assert_eq!(s1, "src/foo.js");
}

#[test]
fn find_rel_source_relative_path() {
  let context = create_context();
  let importer = ModuleId::new("src/components/welcome/index.tsx", "", &context.config.root);
  let visitor = create_visitor(&importer, &context);

  let (_, s1) = visitor.find_rel_source("../../../assets/*.js");

  assert_eq!(s1, format!("assets/*.js"));

  let (_, s1) = visitor.find_rel_source("../../../src/foo.js");

  assert_eq!(s1, "src/foo.js");
}

#[test]
fn test_import_meta_glob() {
  fixture!("tests/fixtures/**/input.js", |file, _crate_path| {
    println!("Testing {file:?}...");
    let file_content = std::fs::read_to_string(&file).unwrap();
    let cm = Arc::new(SourceMap::default());
    let ParseScriptModuleResult { mut ast, .. } = parse_module(
      &file.to_string_lossy().to_string().as_str().into(),
      Arc::new(file_content),
      Syntax::Es(Default::default()),
      EsVersion::EsNext,
      // None,
    )
    .unwrap();
    let dir = file.parent().unwrap().to_str().unwrap();
    let root = if dir.contains("glob_embrace_url") {
      file.parent().unwrap().parent().unwrap().to_str().unwrap()
    } else {
      dir
    };

    let importer = ModuleId::new(&file.to_string_lossy(), "", root);

    let alias = HashMap::from([(
      "@".to_string(),
      file
        .parent()
        .unwrap()
        .to_path_buf()
        .join("dir")
        .to_string_lossy()
        .to_string(),
    )]);

    let mut context = CompilationContext::default();

    context.config.resolve.alias = alias;
    context.config.root = root.to_string();

    let context = Arc::new(context);

    transform_import_meta_glob(
      &mut ast,
      root.to_string(),
      &importer,
      cur_dir_by_importer(&importer, &context),
      &context.config.resolve.alias,
      ImportMetaGlobResolverImpl {
        context: context.clone(),
      },
    )
    .unwrap();

    let code = codegen_module(&ast, EsVersion::EsNext, cm, None, false, None).unwrap();
    let code = String::from_utf8(code).unwrap();

    let expected_file = file.with_extension("expected.js");
    // write to file if not exists
    if !expected_file.exists() {
      std::fs::write(&expected_file, code).unwrap();
    } else {
      let expected = std::fs::read_to_string(&expected_file).unwrap();
      assert_eq!(code, expected.replace("\r\n", "\n"));
    }
  });
}
