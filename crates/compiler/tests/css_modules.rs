use std::collections::HashMap;

use farmfe_core::config::Mode;
use farmfe_testing_helpers::fixture;
mod common;

use crate::common::{assert_compiler_result, create_css_modules_compiler};

#[test]
fn css_module() {
  fixture!("tests/fixtures/css/modules/**/*.ts", |file, crate_path| {
    let cwd = file.parent().unwrap();

    let entry_name = "index".to_string();

    let compiler = create_css_modules_compiler(
      HashMap::from([(entry_name.clone(), "./index.ts".into())]),
      cwd.to_path_buf(),
      crate_path,
      Mode::Production,
    );

    println!("{}", file.to_string_lossy());

    compiler.compile().unwrap();

    assert_compiler_result(&compiler, Some(&entry_name));
  });
}
