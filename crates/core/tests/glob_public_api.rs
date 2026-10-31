use farmfe_core::glob::{compile_matcher, CompileOptions, Glob, WalkOptions};

#[test]
fn public_glob_namespace_uses_scoped_matching_and_fallible_results() {
  let pattern = Glob::new("src/*.{js,ts}").unwrap();
  assert!(pattern.is_match("src/main.js").unwrap());
  assert!(pattern.is_match("src/main.ts").unwrap());
  assert!(!pattern.is_match("src/main.css").unwrap());
  assert!(!pattern.is_match("src/.hidden.js").unwrap());
  assert!(Glob::with_options(
    "src/*.js",
    WalkOptions {
      exhaustive: true,
      ..WalkOptions::default()
    }
  )
  .unwrap()
  .is_match("src/.hidden.js")
  .unwrap());
}

#[test]
fn public_glob_errors_remain_distinct_from_non_matches() {
  assert!(Glob::new("").is_err());
  assert!(Glob::new(&"x".repeat(65_537)).is_err());
  assert!(compile_matcher(
    "[",
    &CompileOptions {
      strict_brackets: true,
      ..CompileOptions::default()
    }
  )
  .is_err());
  assert!(!Glob::new("src/*.js")
    .unwrap()
    .is_match("other/main.js")
    .unwrap());
}
