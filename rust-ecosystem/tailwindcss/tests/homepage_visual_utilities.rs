use farmfe_ecosystem_tailwindcss::{compile, parse_candidate, CompilerOptions};
use farmfe_ecosystem_tailwindcss::{theme::Theme, utilities::UtilityRegistry};

const ACTUAL_CLASSES: &[&str] = &[
  "backdrop-blur-sm",
  "bg-no-repeat",
  "dark:shadow-none",
  "duration-200",
  "duration-300",
  "duration-500",
  "ease-in",
  "ease-in-out",
  "ease-out",
  "group-hover/bento:translate-x-2",
  "group-hover:translate-x-0.5",
  "hover:duration-300",
  "hover:shadow-[inset_0_-5px_10px_#8fdfff3f]",
  "hover:shadow-xl",
  "max-w-7xl",
  "max-w-lg",
  "max-w-md",
  "self-start",
  "shadow-[0_0_20px_-12px_rgba(0,0,0,0.8)]",
  "shadow-[inset_0_-8px_10px_#8fdfff1f]",
  "space-x-2",
  "space-y-2",
  "space-y-4",
  "tracking-wide",
  "transform-gpu",
  "transition",
  "transition-all",
  "transition-shadow",
  "transition-transform",
  "container",
];
#[test]
fn real_homepage_utilities_have_generator_coverage() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::with_defaults();
  let missing: Vec<_> = ACTUAL_CLASSES
    .iter()
    .filter(|name| parse_candidate(name).is_none_or(|c| registry.generate(&c, &theme).is_empty()))
    .collect();
  assert!(
    missing.is_empty(),
    "missing generator coverage: {missing:?}"
  );
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let css = compiler.build(
    &ACTUAL_CLASSES
      .iter()
      .map(|s| s.to_string())
      .collect::<Vec<_>>(),
  );
  assert!(css.contains("background-repeat: no-repeat"), "{css}");
}

#[test]
fn transitions_remain_composable_regardless_of_candidate_order() {
  let candidates = [
    "transition-transform",
    "duration-300",
    "ease-out",
    "group-hover/bento:translate-x-2",
    "transform-gpu",
  ];
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let css = compiler.build(&candidates.map(str::to_owned));
  assert!(
    css.contains("transition-duration: var(--tw-duration, 150ms)"),
    "default must not overwrite duration utility: {css}"
  );
  assert!(css.contains("--tw-duration: 300ms"), "{css}");
  assert!(
    css.contains("transition-timing-function: var(--tw-ease,"),
    "{css}"
  );
  assert!(
    css.contains(":where(.group\\/bento):hover"),
    "named group must scope to its marker: {css}"
  );
  assert!(
    css.contains("--tw-translate-x: calc(var(--spacing) * 2)"),
    "{css}"
  );
  assert!(
    css.contains("var(--tw-transform-gpu,) translate("),
    "GPU and translation must compose: {css}"
  );
  let mut reversed = candidates.map(str::to_owned);
  reversed.reverse();
  assert_eq!(compiler.build(&reversed), css);
}
#[test]
fn spacing_targets_only_visible_following_siblings_and_preserves_variants() {
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let css = compiler.build(&["space-x-2", "hover:space-y-4!"].map(str::to_owned));
  assert!(
    css.contains(".space-x-2>:not([hidden])~:not([hidden])"),
    "{css}"
  );
  assert!(
    css.contains(":hover>:not([hidden])~:not([hidden])"),
    "{css}"
  );
  assert!(
    css.contains(
      "margin-top: calc(calc(var(--spacing) * 4) * (1 - var(--tw-space-y-reverse))) !important"
    ),
    "{css}"
  );
  assert!(css.contains("margin-left:"), "{css}");
}
#[test]
fn sizing_and_container_use_theme_tokens_and_responsive_breakpoints() {
  let mut compiler = compile(
    "@theme { --container-lg: 35rem; --breakpoint-sm: 42rem; } @tailwind utilities;",
    CompilerOptions::default(),
  )
  .unwrap();
  let css = compiler.build(&["container", "max-w-lg", "max-w-md", "max-w-7xl"].map(str::to_owned));
  assert!(css.contains("width: 100%"), "{css}");
  assert!(css.contains("(min-width: 42rem)"), "{css}");
  assert!(css.contains("max-width: 42rem"), "{css}");
  assert!(
    css.contains("--container-lg: 35rem") && css.contains("max-width: var(--container-lg)"),
    "{css}"
  );
  assert!(
    css.contains("--container-md: 28rem") && css.contains("--container-7xl: 80rem"),
    "{css}"
  );
  assert!(
    !css.contains("margin: 0 auto"),
    "container centering is authored Infima styling, not implicit Tailwind: {css}"
  );
}
#[test]
fn effects_keep_arbitrary_values_ring_composition_and_per_element_resets() {
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let css = compiler.build(
    &[
      "backdrop-blur-sm",
      "shadow-[inset_0_-8px_10px_#8fdfff1f]",
      "dark:shadow-none",
    ]
    .map(str::to_owned),
  );
  assert!(
    css.contains("-webkit-backdrop-filter: var(--tw-backdrop-blur,)"),
    "{css}"
  );
  assert!(
    css.contains("--blur-sm: 4px"),
    "legacy homepage blur must match reference: {css}"
  );
  assert!(
    css.contains("--tw-shadow: inset 0 -8px 10px #8fdfff1f"),
    "{css}"
  );
  assert!(
    css.contains("var(--tw-ring-offset-shadow, 0 0 #0000), var(--tw-ring-shadow, 0 0 #0000)"),
    "{css}"
  );
  assert!(css.contains("--tw-shadow: 0 0 #0000"), "{css}");
  for variable in [
    "--tw-shadow",
    "--tw-duration",
    "--tw-translate-x",
    "--tw-backdrop-blur",
  ] {
    assert!(
      css.contains(&format!("{variable}: initial")),
      "nested elements must not inherit effect variables: {css}"
    );
  }
}
