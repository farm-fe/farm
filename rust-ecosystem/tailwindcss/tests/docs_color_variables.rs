use farmfe_ecosystem_tailwindcss::{CompilerOptions, TailwindConfig, compile};

#[test]
fn optional_color_variables_are_flattened_sorted_and_layered() {
  let config = serde_json::json!({"theme": {"extend": {"colors": {
    "blue": {"500": "#3b82f6"},
    "brand": {"DEFAULT": "#fea7df", "foreground": "#111"}
  }}}});
  let mut compiler = compile(
    "@tailwind utilities;",
    CompilerOptions {
      config: Some(TailwindConfig::new(config)),
      ..Default::default()
    },
  )
  .unwrap();
  let plain = compiler.build(&[]);
  assert!(!plain.contains("--blue-500:"));
  compiler.set_color_variables(true);
  let css = compiler.build(&[]);
  assert!(
    css.contains("@layer base"),
    "aliases must not override unlayered authored variables: {css}"
  );
  for expected in [
    "--blue-500: #3b82f6",
    "--brand: #fea7df",
    "--brand-foreground: #111",
    "--black: #000",
    "--white: #fff",
    "--transparent: transparent",
  ] {
    assert!(css.contains(expected), "{expected}: {css}");
  }
  assert!(!css.contains("--spacing:"));
  assert!(css.find("--black:").unwrap() < css.find("--blue-500:").unwrap());
  assert_eq!(compiler.build(&[]), css);
  compiler.set_color_variables(false);
  assert_eq!(compiler.build(&[]), plain);
}
