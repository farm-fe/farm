use farmfe_ecosystem_tailwindcss::compiler::{CompilerOptions, TailwindConfig, compile};

#[test]
fn website_class_dark_mode_uses_dom_selector() {
  let config = serde_json::json!({"darkMode": ["class"], "theme": {"extend": {"colors": {"background": "hsl(var(--background))"}}}});
  let mut compiler = compile(
    "@tailwind utilities;",
    CompilerOptions {
      config: Some(TailwindConfig::new(config)),
      ..Default::default()
    },
  )
  .unwrap();
  let css = compiler.build(&["dark:bg-background".into()]);
  assert!(css.contains(":where(.dark, .dark *)"), "{css}");
  assert!(!css.contains("prefers-color-scheme"), "{css}");
}

#[test]
fn website_animation_config_generates_utility_and_keyframes() {
  let config = serde_json::json!({"theme": {"extend": {
    "animation": {"shimmer": "shimmer 8s infinite"},
    "keyframes": {"shimmer": {"from": {"backgroundPosition": "-100% 0", "opacity": "0"}, "to": {"backgroundPosition": "100% 0", "opacity": "1"}}}
  }}});
  let mut compiler = compile(
    "@tailwind utilities;",
    CompilerOptions {
      config: Some(TailwindConfig::new(config)),
      ..Default::default()
    },
  )
  .unwrap();
  let css = compiler.build(&["animate-shimmer".into()]);
  assert!(css.contains("animation:"), "{css}");
  assert!(css.contains("shimmer 8s infinite"), "{css}");
  assert!(css.contains("@keyframes shimmer"), "{css}");
  assert!(css.contains("background-position:"), "{css}");
}
