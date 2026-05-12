use farmfe_ecosystem_tailwindcss::{compile, CompilerOptions, Features, Polyfills};

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
  let compiler = compile(
    ".foo { @apply flex; }",
    CompilerOptions::default(),
  )
  .unwrap();
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
