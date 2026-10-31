use farmfe_ecosystem_glob::{compile_matcher, CompileOptions, Glob, WalkOptions};
use serde_json::Value;

#[test]
fn fixed_picomatch_reference_profiles() {
  let corpus: Value = serde_json::from_str(include_str!("profiles.json")).unwrap();
  let cases = corpus["cases"].as_array().unwrap();
  assert_eq!(cases.len(), 3352);
  for row in cases {
    let options: CompileOptions = serde_json::from_value(row["options"].clone()).unwrap();
    let matcher = compile_matcher(row["pattern"].as_str().unwrap(), &options);
    if row["expected"]["status"] == "error" {
      assert!(matcher.is_err(), "{}: expected compile error", row["id"]);
    } else {
      let actual = matcher
        .unwrap_or_else(|error| panic!("{}: {error:?}", row["id"]))
        .is_match(row["input"].as_str().unwrap())
        .unwrap();
      assert_eq!(
        actual,
        row["expected"]["matches"].as_bool().unwrap(),
        "{}",
        row["id"]
      );
    }
  }
}

#[test]
fn walker_dot_dependencies_and_parenthesized_pattern() {
  let root = tempfile::tempdir().unwrap();
  std::fs::create_dir_all(root.path().join("src")).unwrap();
  std::fs::create_dir_all(root.path().join("node_modules/dep")).unwrap();
  for file in ["src/a.js", "src/.hidden.js", "node_modules/dep/a.js"] {
    std::fs::write(root.path().join(file), "export default 1").unwrap();
  }
  let files = |pattern, options| {
    Glob::with_options(pattern, options)
      .unwrap()
      .walk(root.path())
      .map(|entry| {
        entry
          .unwrap()
          .path()
          .strip_prefix(root.path())
          .unwrap()
          .to_string_lossy()
          .replace('\\', "/")
      })
      .collect::<Vec<_>>()
  };
  assert_eq!(files("**/*.js", WalkOptions::default()), vec!["src/a.js"]);
  assert!(files("node_modules/dep/*.js", WalkOptions::default()).is_empty());
  let mut all = files(
    "**/*.js",
    WalkOptions {
      exhaustive: true,
      ..Default::default()
    },
  );
  all.sort();
  assert_eq!(
    all,
    vec!["node_modules/dep/a.js", "src/.hidden.js", "src/a.js"]
  );
  let grouped = files("src/(*).js", WalkOptions::default());
  assert_eq!(grouped.len(), 2);
}
