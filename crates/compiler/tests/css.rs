use std::collections::HashMap;

use farmfe_core::config::{CssConfig, CssModulesConfig, CssPrefixerConfig};
use farmfe_testing_helpers::fixture;
mod common;

use crate::common::{assert_compiler_result, create_css_compiler};

#[test]
fn css_modules() {
  fixture!("tests/fixtures/css/modules/**/*.ts", |file, crate_path| {
    let cwd = file.parent().unwrap();

    let entry_name = "index".to_string();

    let compiler = create_css_compiler(
      HashMap::from([(entry_name.clone(), "./index.ts".into())]),
      cwd.to_path_buf(),
      crate_path,
      CssConfig {
        modules: Some(CssModulesConfig {
          indent_name: "farm-[name]".into(),
          paths: vec![".+".to_string()],
        }),
        ..Default::default()
      },
    );

    compiler.compile().unwrap();

    assert_compiler_result(&compiler, Some(&entry_name));
  });
}

#[test]
fn css_prefixer() {
  fixture!("tests/fixtures/css/prefixer/**/*.ts", |file, crate_path| {
    let cwd = file.parent().unwrap();

    let entry_name = "index".to_string();

    let compiler = create_css_compiler(
      HashMap::from([(entry_name.clone(), "./index.ts".into())]),
      cwd.to_path_buf(),
      crate_path,
      CssConfig {
        prefixer: Some(CssPrefixerConfig {
          targets: farmfe_toolkit::swc_css_prefixer::options::Options::default().env,
        }),
        ..Default::default()
      },
    );

    compiler.compile().unwrap();

    assert_compiler_result(&compiler, Some(&entry_name));
  });
}
