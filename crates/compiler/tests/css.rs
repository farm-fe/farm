use std::path::PathBuf;

use farmfe_core::config::{AdapterType, Config, CssConfig, CssModulesConfig, CssPrefixerConfig};
use farmfe_core::HashMap;
use farmfe_testing_helpers::fixture;
mod common;

use crate::common::{
  assert_compiler_result, create_css_compiler, test_builder, TestBuilderOptions,
};

#[test]
fn css_modules() {
  fixture!(
    "tests/fixtures/css/modules/normal/**/*.ts",
    |file, crate_path| {
      test_builder(
        TestBuilderOptions::new(file.to_string_lossy().to_string(), crate_path).with_config(
          r#"
          {
            "css": {
              "modules": {
                "indentName": "farm-[name]",
                "paths": [".+"]
              }
            }
          }
          "#,
        ),
      );

      // let cwd = file.parent().unwrap();
      // println!("cwd: {cwd:?}");

      // let entry_name = "index".to_string();

      // let compiler = create_css_compiler(
      //   HashMap::from_iter([(entry_name.clone(), "./index.ts".into())]),
      //   cwd.to_path_buf(),
      //   crate_path,
      //   CssConfig {
      //     modules: Some(CssModulesConfig {
      //       indent_name: "farm-[name]".into(),
      //       paths: vec![".+".to_string()],
      //     }),
      //     ..Default::default()
      //   },
      // );

      // compiler.compile().unwrap();

      // assert_compiler_result(&compiler, Some(&entry_name));
    }
  );
}

#[test]
fn css_prefixer() {
  fixture!("tests/fixtures/css/prefixer/**/*.ts", |file, crate_path| {
    let cwd = file.parent().unwrap();

    let entry_name = "index".to_string();

    let compiler = create_css_compiler(
      HashMap::from_iter([(entry_name.clone(), "./index.ts".into())]),
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

#[test]
fn css_url_replacer() {
  fixture!(
    "tests/fixtures/css/url_replacer/**/*.ts",
    |file, crate_path| {
      let cwd = file.parent().unwrap();

      let entry_name = "index".to_string();

      let compiler = create_css_compiler(
        HashMap::from_iter([(entry_name.clone(), "./index.ts".into())]),
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
    }
  );
}
