use farmfe_ecosystem_tailwindcss::{CompilerOptions, compile};

#[test]
fn aurora_effect_utilities_emit_composable_filter_content_and_blending() {
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let candidates = [
    "filter",
    "blur-[10px]",
    "invert",
    "dark:invert-0",
    "after:content-[\"\"]",
    "after:mix-blend-difference",
    "will-change-transform",
  ];
  let css = compiler.build(&candidates.map(str::to_owned));
  for required in [
    "content: \"\"",
    "--tw-blur: blur(10px)",
    "--tw-invert: invert(100%)",
    "--tw-invert: invert(0%)",
    "mix-blend-mode: difference",
    "will-change: transform",
  ] {
    assert!(css.contains(required), "missing {required}: {css}");
  }
  assert!(css.contains("var(--tw-blur,)") && css.contains("var(--tw-invert,)"));
  assert!(
    css.contains("::before, ::after") || css.contains("::before,\n"),
    "filter defaults must reset per element/pseudo rather than inherit parent filters: {css}"
  );
}
