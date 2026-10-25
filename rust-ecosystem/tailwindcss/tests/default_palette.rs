use farmfe_ecosystem_tailwindcss::theme::Theme;

#[test]
fn default_palette_covers_every_upstream_family_and_shade() {
  let theme = Theme::with_defaults();
  for family in [
    "red", "orange", "amber", "yellow", "lime", "green", "emerald", "teal", "cyan", "sky", "blue",
    "indigo", "violet", "purple", "fuchsia", "pink", "rose", "slate", "gray", "zinc", "neutral",
    "stone",
  ] {
    for shade in [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950] {
      let key = format!("--color-{family}-{shade}");
      assert!(theme.get(&[&key]).is_some(), "Missing default token {key}");
    }
  }
  assert_eq!(
    theme.get(&["--color-emerald-100"]),
    Some("oklch(95% 0.052 163.051)")
  );
  assert_eq!(
    theme.get(&["--color-emerald-900"]),
    Some("oklch(37.8% 0.077 168.94)")
  );
}
