use std::{path::PathBuf, sync::Arc};

use farmfe_core::swc_common::{FilePathMapping, SourceMap};
use farmfe_toolkit::{
  fs::read_file_utf8,
  script::{codegen_module, module_type_from_id, parse_module, syntax_from_module_type},
  testing_helpers::fixture,
};

#[test]
fn parse_and_codegen_module() {
  fixture("tests/fixtures/script/**/index.*", |file: PathBuf| {
    let id = file.to_string_lossy().to_string();
    let content = read_file_utf8(&id).unwrap();

    let module_type = module_type_from_id(&id);
    let syntax = syntax_from_module_type(&module_type).unwrap();
    let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
    let ast = parse_module(&id, &content, syntax, cm.clone()).unwrap();

    assert_eq!(ast.body.len(), 3);

    let bytes = codegen_module(&ast, cm).unwrap();

    let code = String::from_utf8(bytes).unwrap();
    assert_eq!(
      code,
      "import a from \"./a\";\nimport b from \"./b\";\nconsole.log(a, b);\n"
    );
  });
}
