use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  context::CompilationContext, module::ModuleId, swc_common::SourceMap, swc_ecma_ast::EsVersion,
  swc_ecma_parser::Syntax,
};
use farmfe_swc_transformer_import_glob::transform_import_meta_glob;
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::script::{codegen_module, parse_module, ParseScriptModuleResult};

#[test]
fn test_import_meta_glob() {
  fixture!("tests/fixtures/**/input.js", |file, _crate_path| {
    println!("Testing {file:?}...");
    let file_content = std::fs::read_to_string(&file).unwrap();
    let cm = Arc::new(SourceMap::default());
    let ParseScriptModuleResult { mut ast, .. } = parse_module(
      file.to_string_lossy().to_string().as_str(),
      &file_content,
      Syntax::Es(Default::default()),
      EsVersion::EsNext,
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

    transform_import_meta_glob(&mut ast, root.to_string(), &importer, &context).unwrap();

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
