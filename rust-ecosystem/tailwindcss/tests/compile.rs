mod support;

#[allow(dead_code)]
mod normalize_path {
  include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/normalize_path.rs"));
}

#[allow(dead_code)]
mod urls {
  include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/urls.rs"));
}

#[allow(dead_code)]
mod resolve {
  include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/resolve.rs"));
}

#[allow(dead_code)]
mod compile {
  include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/compile.rs"));

  #[cfg(test)]
  mod moved_tests {
    use super::*;
    use crate::support::{fixture_path, manifest_path};
    use farmfe_testing_helpers::assert_snapshot;
    use std::fs;
    use std::sync::{Arc, Mutex};

    #[test]
    fn compile_fixture_outputs_are_snapshotted() {
      let simple_input = fs::read_to_string(fixture_path("compile/simple/input.css")).unwrap();
      let simple_dir = fixture_path("compile/simple");
      let mut simple = compile(
        &simple_input,
        CompileOptions {
          base: simple_dir.to_str().unwrap().to_string(),
          ..Default::default()
        },
      )
      .unwrap();

      let imports_input = fs::read_to_string(fixture_path("compile/imports/input.css")).unwrap();
      let imports_dir = fixture_path("compile/imports");
      let deps: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
      let deps_clone = deps.clone();
      let mut imports = compile(
        &imports_input,
        CompileOptions {
          base: imports_dir.to_str().unwrap().to_string(),
          on_dependency: Box::new(move |dep| deps_clone.lock().unwrap().push(dep.to_string())),
          ..Default::default()
        },
      )
      .unwrap();

      let rewrite_input = fs::read_to_string(fixture_path("compile/rewrite/input.css")).unwrap();
      let rewrite_dir = fixture_path("compile/rewrite");
      let mut rewrite = compile(
        &rewrite_input,
        CompileOptions {
          base: rewrite_dir.to_str().unwrap().to_string(),
          should_rewrite_urls: true,
          ..Default::default()
        },
      )
      .unwrap();

      let output = format!(
        "simple:\n-- input --\n{simple_input}\n-- output --\n{}\n\nimports:\n-- input --\n{imports_input}\n-- output --\n{}\n-- deps --\n{}\n\nrewrite:\n-- input --\n{rewrite_input}\n-- output --\n{}",
        simple.build(&[]),
        imports.build(&[]),
        deps.lock().unwrap().iter().map(|dep| manifest_path(std::path::Path::new(dep))).collect::<Vec<_>>().join("\n"),
        rewrite.build(&[]),
      );

      assert_snapshot!(output);
    }

    #[test]
    fn compile_feature_detection_and_metadata() {
      let fixture_dir = fixture_path("compile/simple");

      let at_apply = compile(
        ".foo { @apply text-red-500; }",
        CompileOptions {
          base: fixture_dir.to_str().unwrap().to_string(),
          ..Default::default()
        },
      )
      .unwrap();
      assert!(at_apply.features.contains(Features::AT_APPLY));

      let theme = compile(
        ".foo { color: theme(colors.red.500); }",
        CompileOptions {
          base: fixture_dir.to_str().unwrap().to_string(),
          ..Default::default()
        },
      )
      .unwrap();
      assert!(theme.features.contains(Features::THEME_FUNCTION));

      let utilities = compile(
        "@import \"tailwindcss\";",
        CompileOptions {
          base: fixture_dir.to_str().unwrap().to_string(),
          ..Default::default()
        },
      )
      .unwrap();
      assert!(utilities.features.contains(Features::UTILITIES));
      assert!(utilities.build_source_map().is_none());

      let source_maps = compile(
        ".foo { color: red; }",
        CompileOptions {
          base: fixture_dir.to_str().unwrap().to_string(),
          from: Some("input.css".to_string()),
          ..Default::default()
        },
      )
      .unwrap();
      assert!(source_maps.build_source_map().is_some());

      let polyfills = compile(
        ".foo { color: red; }",
        CompileOptions {
          base: fixture_dir.to_str().unwrap().to_string(),
          polyfills: Polyfills::AT_MEDIA_HOVER,
          ..Default::default()
        },
      )
      .unwrap();
      assert!(polyfills.polyfills.contains(Polyfills::AT_MEDIA_HOVER));

      let mut ast_compiler = compile_ast(
        &[
          AstNode::Rule {
            selector: ".foo".to_string(),
            nodes: vec![AstNode::Declaration {
              property: "color".to_string(),
              value: "red".to_string(),
            }],
          },
          AstNode::AtRule {
            name: "import".to_string(),
            params: "\"tailwindcss\"".to_string(),
            nodes: vec![],
          },
        ],
        CompileOptions {
          base: fixture_dir.to_str().unwrap().to_string(),
          ..Default::default()
        },
      )
      .unwrap();
      assert!(ast_compiler.features.contains(Features::UTILITIES));
      assert!(ast_compiler.build(&[]).contains(".foo"));

      let node = AstNode::Comment("test comment".to_string());
      assert_eq!(node.to_css(), "/* test comment */");
      assert!(ensure_source_detection_root_exists(&None).is_ok());
      assert!(ensure_source_detection_root_exists(&Some(SourceRoot {
        pattern: "**/*.css".to_string(),
        base: fixture_dir.to_str().unwrap().to_string(),
      }))
      .is_ok());
      assert!(ensure_source_detection_root_exists(&Some(SourceRoot {
        pattern: "missing/**/*.css".to_string(),
        base: fixture_dir.join("missing").to_string_lossy().to_string(),
      }))
      .is_err());
    }

    #[test]
    fn compile_loaders_and_missing_imports_behave() {
      let fixture_dir = fixture_path("compile/imports");
      let missing = compile(
        "@import \"./nonexistent.css\";\n.foo { color: red; }",
        CompileOptions {
          base: fixture_dir.to_str().unwrap().to_string(),
          ..Default::default()
        },
      )
      .unwrap();
      let mut missing = missing;
      let output = missing.build(&[]);
      assert!(output.contains("nonexistent.css"));
      assert!(output.contains(".foo { color: red; }"));

      let deps: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
      let deps_clone = deps.clone();
      let stylesheet = load_stylesheet(
        "./sub.css",
        fixture_dir.to_str().unwrap(),
        &move |dep| deps_clone.lock().unwrap().push(dep.to_string()),
        None,
      )
      .unwrap();
      assert!(stylesheet.content.contains("body { margin: 0; }"));
      assert_eq!(manifest_path(&stylesheet.path), "tests/fixtures/compile/imports/sub.css");
      assert!(!deps.lock().unwrap().is_empty());

      let module = load_module(
        "./config.js",
        fixture_path("compile/module").to_str().unwrap(),
        &|_| {},
        None,
      )
      .unwrap();
      assert_eq!(manifest_path(&module.path), "tests/fixtures/compile/module/config.js");

      let mut design_system =
        load_design_system(".foo { color: red; }", fixture_dir.to_str().unwrap()).unwrap();
      assert!(design_system.build(&[]).contains(".foo"));

      let features = Features::AT_APPLY | Features::UTILITIES;
      assert!(features.contains(Features::AT_APPLY));
      assert!(features.contains(Features::UTILITIES));
      assert!(!features.contains(Features::THEME_FUNCTION));
      assert!(features.has_any_output_feature());
      assert!(!Features::NONE.has_any_output_feature());

      let polyfills = Polyfills::NONE | Polyfills::AT_MEDIA_HOVER;
      assert!(polyfills.contains(Polyfills::AT_MEDIA_HOVER));
    }
  }
}
