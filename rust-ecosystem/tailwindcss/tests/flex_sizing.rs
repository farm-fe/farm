use farmfe_ecosystem_tailwindcss::{CompilerOptions, compile};

#[test]
fn standard_flex_sizing_keeps_percentage_basis_for_indefinite_column_layouts() {
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let css = compiler.build(&[
    "flex-1".into(),
    "flex-auto".into(),
    "flex-initial".into(),
    "flex-none".into(),
  ]);
  for (class, value) in [
    ("flex-1", "1 1 0%"),
    ("flex-auto", "1 1 auto"),
    ("flex-initial", "0 1 auto"),
    ("flex-none", "none"),
  ] {
    assert!(css.contains(&format!(".{class}")), "{class} missing: {css}");
    assert!(
      css.contains(&format!("flex: {value};")),
      "{class} value missing: {css}"
    );
  }
}

#[test]
fn flex_sizing_works_in_responsive_variants_and_apply() {
  let mut compiler = compile(
    "@tailwind utilities; .panel { @apply flex-1; }",
    CompilerOptions::default(),
  )
  .unwrap();
  let css = compiler.build(&["md:flex-1".into()]);
  assert!(css.contains(".md\\:flex-1"), "{css}");
  assert!(css.contains("@media"), "{css}");
  assert!(css.contains(".panel"), "{css}");
  assert_eq!(css.matches("flex: 1 1 0%;").count(), 2, "{css}");
}
