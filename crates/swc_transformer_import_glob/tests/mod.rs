use std::sync::Arc;

use farmfe_core::{swc_common::SourceMap, swc_ecma_ast::EsVersion, swc_ecma_parser::Syntax};
use farmfe_swc_transformer_import_glob::transform_import_meta_glob;
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::script::{codegen_module, parse_module};

#[test]
fn test_import_meta_glob() {
  fixture!("tests/fixtures/**/input.js", |file, _crate_path| {
    let file_content = std::fs::read_to_string(&file).unwrap();
    let cm = Arc::new(SourceMap::default());
    let mut ast = parse_module(
      file.to_string_lossy().to_string().as_str(),
      &file_content,
      Syntax::Es(Default::default()),
      EsVersion::EsNext,
      cm.clone(),
    )
    .unwrap();
    let dir = file.parent().unwrap().to_str().unwrap();
    transform_import_meta_glob(&mut ast, dir.to_string(), dir.to_string()).unwrap();

    let code = codegen_module(&ast, EsVersion::EsNext, cm, None, false).unwrap();
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
