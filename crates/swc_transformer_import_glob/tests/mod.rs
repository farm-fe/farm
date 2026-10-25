use std::{path::Path, sync::Arc};

use farmfe_core::{
  config::{AliasItem, StringOrRegex},
  context::CompilationContext,
  module::{ModuleId, VIRTUAL_MODULE_PREFIX},
  swc_common::SourceMap,
  swc_ecma_ast::EsVersion,
  swc_ecma_parser::Syntax,
};
use farmfe_swc_transformer_import_glob::{
  ImportGlobVisitor, ImportMetaGlobResolver, ImportMetaGlobResolverParams,
  transform_import_meta_glob,
};
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::script::{ParseScriptModuleResult, codegen_module, parse_module};

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
fn root_relative_glob_matches_entries_without_current_directory_prefix() {
  use farmfe_core::wax::{Glob, Pattern};

  let context = create_context();
  let importer = ModuleId::new("src/index.js", "", &context.config.root);
  let visitor = create_visitor(&importer, &context);
  let (root, pattern) = visitor.find_rel_source("/src/*.js");

  assert_eq!(root, context.config.root);
  assert_eq!(pattern, "src/*.js");
  assert!(Glob::new(&pattern).unwrap().is_match("src/foo.js"));
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

    let mut context = CompilationContext::default();

    context.config.resolve.alias = vec![AliasItem {
      find: StringOrRegex::String("@".to_string()),
      replacement: file
        .parent()
        .unwrap()
        .to_path_buf()
        .join("dir")
        .to_string_lossy()
        .to_string(),
    }];
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

    let code = codegen_module(&ast, cm, None, Default::default(), None).unwrap();
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

fn transform_case(
  code: &str,
  root: &Path,
) -> farmfe_core::error::Result<(
  String,
  Vec<Vec<farmfe_swc_transformer_import_glob::GlobWatch>>,
)> {
  let file = root.join("index.js");
  let root = root.to_string_lossy().to_string();
  let importer = ModuleId::new(&file.to_string_lossy(), "", &root);
  let mut context = CompilationContext::default();
  context.config.root = root.clone();
  context.config.resolve.alias = vec![AliasItem {
    find: StringOrRegex::String("@".into()),
    replacement: format!("{root}/dir"),
  }];
  let context = Arc::new(context);
  let cm = Arc::new(SourceMap::default());
  let ParseScriptModuleResult { mut ast, .. } = parse_module(
    &file.to_string_lossy().to_string().as_str().into(),
    Arc::new(code.to_owned()),
    Syntax::Es(Default::default()),
    EsVersion::EsNext,
  )
  .unwrap();
  let watches = transform_import_meta_glob(
    &mut ast,
    root.clone(),
    &importer,
    root,
    &context.config.resolve.alias,
    ImportMetaGlobResolverImpl {
      context: context.clone(),
    },
  )?;
  let code = codegen_module(&ast, cm, None, Default::default(), None).unwrap();
  Ok((String::from_utf8(code).unwrap(), watches))
}

#[test]
fn negative_order_query_base_and_watch_profiles() {
  let root = tempfile::tempdir().unwrap();
  std::fs::create_dir(root.path().join("dir")).unwrap();
  for file in ["a.js", "a.test.js", ".hidden.js"] {
    std::fs::write(root.path().join("dir").join(file), "export default 1").unwrap();
  }
  let (first, watches) = transform_case(
    "const x = import.meta.glob(['!./dir/*.test.js','./dir/*.js'], {query:'?raw'});",
    root.path(),
  )
  .unwrap();
  let (last, _) = transform_case(
    "const x = import.meta.glob(['./dir/*.js','!./dir/*.test.js'], {query:'?raw'});",
    root.path(),
  )
  .unwrap();
  assert_eq!(first, last);
  assert!(first.contains("\"./dir/a.js\":"), "{first}");
  assert!(first.contains("import(\"./dir/a.js?raw\")"));
  assert!(!first.contains("a.test.js"));
  assert!(!first.contains(".hidden.js"));
  let new_path = root.path().join("dir/new.js").to_string_lossy().to_string();
  assert!(
    watches[0]
      .iter()
      .find(|watch| !watch.negative)
      .unwrap()
      .matches(&new_path)
      .unwrap()
  );
  let (base, _) = transform_case(
    "const x = import.meta.glob('./*.js', {base:'./dir', eager:true, import:'default'});",
    root.path(),
  )
  .unwrap();
  assert!(base.contains("\"./a.js\":"));
  assert!(base.contains("from \"./dir/a.js\""));
  let (all, _) = transform_case(
    "const x = import.meta.glob('./dir/*.js', {exhaustive:true});",
    root.path(),
  )
  .unwrap();
  assert!(all.contains(".hidden.js"));
  let (query, _) = transform_case(
    "const x = import.meta.glob('./dir/a.js', {query:{foo:'a&b', n:2}});",
    root.path(),
  )
  .unwrap();
  assert!(query.contains("foo=a%26b&n=2"));
  let (alias, _) = transform_case(
    "const x = import.meta.glob('@/a.js', {eager:true, import:'default'});",
    root.path(),
  )
  .unwrap();
  assert!(alias.contains("\"/dir/a.js\":"), "{alias}");
}

#[test]
fn invalid_glob_inputs_are_errors() {
  let root = tempfile::tempdir().unwrap();
  for code in [
    "import.meta.glob()",
    "import.meta.glob(pattern)",
    "import.meta.glob(['./x', pattern])",
    "import.meta.glob('./x', {eager:'yes'})",
    "import.meta.glob('./x', {unknown:true})",
  ] {
    assert!(transform_case(code, root.path()).is_err(), "{code}");
  }
}
