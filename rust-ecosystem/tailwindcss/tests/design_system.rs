use farmfe_ecosystem_tailwindcss::{compile, CompilerOptions, Features};

#[test]
fn test_compiler_build_processes_candidates() {
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let output = compiler.build(&["flex".to_string(), "block".to_string()]);
  assert!(!output.is_empty());
  assert!(output.contains("display: flex") || output.contains("display:flex"));
}

#[test]
fn test_compiler_detects_utilities_feature() {
  let compiler = compile(
    "@import \"tailwindcss\";\n@tailwind utilities;",
    CompilerOptions::default(),
  )
  .unwrap();
  assert!(compiler.features.contains(Features::UTILITIES));
}

#[test]
fn test_compiler_detects_apply_feature() {
  let compiler = compile(".foo { @apply flex; }", CompilerOptions::default()).unwrap();
  assert!(compiler.features.contains(Features::AT_APPLY));
}

#[test]
fn test_compiler_detects_theme_function_feature() {
  let compiler = compile(
    ".foo { color: theme(colors.red.500); }",
    CompilerOptions::default(),
  )
  .unwrap();
  assert!(compiler.features.contains(Features::THEME_FUNCTION));
}

#[test]
fn test_compiler_build_empty_candidates() {
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let output = compiler.build(&[]);
  // Empty candidates → empty or minimal CSS
  assert!(output.is_empty() || !output.contains("display"));
}

// ── Phase 21: @source directive ──────────────────────────────────────────────

use farmfe_ecosystem_tailwindcss::design_system::SourceDirective;

#[test]
fn test_at_source_include_glob() {
  let compiler = compile("@source \"./src/**/*.html\";", CompilerOptions::default()).unwrap();
  assert_eq!(
    compiler.sources(),
    &[SourceDirective::Include("./src/**/*.html".to_string())]
  );
}

#[test]
fn test_at_source_exclude_glob() {
  let compiler = compile("@source not 'node_modules/**';", CompilerOptions::default()).unwrap();
  assert_eq!(
    compiler.sources(),
    &[SourceDirective::Exclude("node_modules/**".to_string())]
  );
}

#[test]
fn test_at_source_inline_and_not_inline() {
  let compiler = compile(
    "@source inline(\"flex block\");\n@source not inline(\"unused-*\");",
    CompilerOptions::default(),
  )
  .unwrap();
  assert_eq!(
    compiler.sources(),
    &[
      SourceDirective::Inline("flex block".to_string()),
      SourceDirective::NotInline("unused-*".to_string()),
    ]
  );
}

#[test]
fn test_at_source_stripped_from_output() {
  let mut compiler = compile(
    "@source \"./src/**/*.html\";\n@tailwind utilities;",
    CompilerOptions::default(),
  )
  .unwrap();
  let output = compiler.build(&["flex".to_string()]);
  assert!(
    !output.contains("@source"),
    "@source must not appear in compiled CSS, got:\n{output}"
  );
  // Sanity: the surrounding pipeline still runs.
  assert!(output.contains("display: flex") || output.contains("display:flex"));
}

#[test]
fn test_at_source_malformed_dropped() {
  // Missing quotes / unrecognised form is silently dropped.
  let compiler = compile("@source ;", CompilerOptions::default()).unwrap();
  assert!(compiler.sources().is_empty());
}
