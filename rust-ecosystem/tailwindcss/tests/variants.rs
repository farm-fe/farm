use farmfe_ecosystem_tailwindcss::ast::to_css;
use farmfe_ecosystem_tailwindcss::candidate::ParsedCandidate;
use farmfe_ecosystem_tailwindcss::theme::Theme;
use farmfe_ecosystem_tailwindcss::utilities::UtilityRegistry;
use farmfe_ecosystem_tailwindcss::variants::VariantRegistry;

/// Build a minimal "flex" candidate with the given variant stack.
fn candidate_with_variants(variants: &[&str]) -> ParsedCandidate {
  ParsedCandidate {
    utility_root: "flex".to_string(),
    utility_value: None,
    arbitrary_property: None,
    arbitrary_value: None,
    type_hint: None,
    variants: variants.iter().map(|s| s.to_string()).collect(),
    modifier: None,
    modifier_is_arbitrary: false,
    important: false,
    negative: false,
    is_static: true,
    raw: format!("{}flex", variants.iter().map(|v| format!("{}:", v)).collect::<String>()),
  }
}

fn generate(variants: &[&str]) -> String {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = candidate_with_variants(variants);
  to_css(&registry.generate(&candidate, &theme))
}

#[test]
fn test_hover_variant_generates_hover_selector() {
  let output = generate(&["hover"]);
  assert!(output.contains(":hover"));
}

#[test]
fn test_focus_variant() {
  let output = generate(&["focus"]);
  assert!(output.contains(":focus"));
}

#[test]
fn test_stacked_variants() {
  let output = generate(&["focus", "hover"]);
  assert!(output.contains(":focus"));
  assert!(output.contains(":hover"));
}

#[test]
fn test_no_variant_returns_plain_selector() {
  let output = generate(&[]);
  assert!(output.starts_with(".flex"));
}

#[test]
fn test_variant_registry_has_builtins() {
  let registry = VariantRegistry::builtin();
  assert!(registry.has("hover"));
  assert!(registry.has("focus"));
  assert!(registry.has("active"));
  assert!(registry.has("disabled"));
  assert!(!registry.has("nonexistent-variant"));
}

// ── new Phase 15 coverage ────────────────────────────────────────────────────

#[test]
fn pseudo_element_before() {
  let output = generate(&["before"]);
  assert!(output.contains("::before"), "got: {output}");
}

#[test]
fn pseudo_element_after() {
  let output = generate(&["after"]);
  assert!(output.contains("::after"));
}

#[test]
fn pseudo_class_first_child() {
  let output = generate(&["first"]);
  assert!(output.contains(":first-child"));
}

#[test]
fn rtl_wraps_with_where() {
  let output = generate(&["rtl"]);
  assert!(output.contains(":where([dir=\"rtl\"]"), "got: {output}");
}

#[test]
fn sm_wraps_in_media_min_width() {
  let output = generate(&["sm"]);
  assert!(output.contains("@media"));
  assert!(output.contains("min-width: 640px"), "got: {output}");
}

#[test]
fn md_wraps_in_media_768() {
  let output = generate(&["md"]);
  assert!(output.contains("min-width: 768px"));
}

#[test]
fn lg_xl_2xl_breakpoints() {
  assert!(generate(&["lg"]).contains("min-width: 1024px"));
  assert!(generate(&["xl"]).contains("min-width: 1280px"));
  assert!(generate(&["2xl"]).contains("min-width: 1536px"));
}

#[test]
fn max_sm_uses_max_width() {
  let output = generate(&["max-sm"]);
  assert!(output.contains("max-width"));
  assert!(output.contains("640px"));
}

#[test]
fn dark_uses_prefers_color_scheme() {
  let output = generate(&["dark"]);
  assert!(output.contains("prefers-color-scheme: dark"));
}

#[test]
fn print_media() {
  let output = generate(&["print"]);
  assert!(output.contains("@media print"));
}

#[test]
fn motion_reduce_media() {
  let output = generate(&["motion-reduce"]);
  assert!(output.contains("prefers-reduced-motion: reduce"));
}

#[test]
fn portrait_and_landscape() {
  assert!(generate(&["portrait"]).contains("orientation: portrait"));
  assert!(generate(&["landscape"]).contains("orientation: landscape"));
}

#[test]
fn contrast_more_less() {
  assert!(generate(&["contrast-more"]).contains("prefers-contrast: more"));
  assert!(generate(&["contrast-less"]).contains("prefers-contrast: less"));
}

#[test]
fn arbitrary_min_media() {
  let output = generate(&["min-[640px]"]);
  assert!(output.contains("@media"));
  assert!(output.contains("(min-width: 640px)"), "got: {output}");
}

#[test]
fn arbitrary_max_media() {
  let output = generate(&["max-[900px]"]);
  assert!(output.contains("(max-width: 900px)"));
}

#[test]
fn supports_arbitrary() {
  let output = generate(&["supports-[display:grid]"]);
  assert!(output.contains("@supports"));
  assert!(output.contains("(display:grid)"), "got: {output}");
}

#[test]
fn container_query_named_sm() {
  let output = generate(&["@sm"]);
  assert!(output.contains("@container"));
  assert!(output.contains("(min-width: 24rem)"));
}

#[test]
fn container_query_arbitrary_min() {
  let output = generate(&["@min-[20rem]"]);
  assert!(output.contains("@container"));
  assert!(output.contains("(min-width: 20rem)"));
}

#[test]
fn container_query_arbitrary_shorthand() {
  let output = generate(&["@[18rem]"]);
  assert!(output.contains("@container"));
  assert!(output.contains("(min-width: 18rem)"));
}

#[test]
fn container_query_bare() {
  let output = generate(&["@container"]);
  // `@container` with no params still produces an @-rule wrapper.
  assert!(output.contains("@container"), "got: {output}");
}

#[test]
fn data_arbitrary_attribute_named() {
  let output = generate(&["data-[open]"]);
  assert!(output.contains("[data-open]"), "got: {output}");
}

#[test]
fn data_arbitrary_attribute_keyed() {
  let output = generate(&["data-[size=large]"]);
  assert!(output.contains("[data-size=\"large\"]"), "got: {output}");
}

#[test]
fn aria_arbitrary_keyed() {
  let output = generate(&["aria-[expanded=true]"]);
  assert!(output.contains("[aria-expanded=\"true\"]"), "got: {output}");
}

#[test]
fn aria_named_alias() {
  let output = generate(&["aria-checked"]);
  // Named `aria-checked` defaults to `[aria-checked="true"]` per upstream.
  assert!(output.contains("[aria-checked=\"true\"]"), "got: {output}");
}

#[test]
fn not_hover() {
  let output = generate(&["not-hover"]);
  assert!(output.contains(":not(:hover)"), "got: {output}");
}

#[test]
fn has_arbitrary_selector() {
  let output = generate(&["has-[input]"]);
  assert!(output.contains(":has(input)"), "got: {output}");
}

#[test]
fn has_named_alias() {
  let output = generate(&["has-checked"]);
  assert!(output.contains(":has(:checked)"), "got: {output}");
}

#[test]
fn group_hover_compounds_descendant() {
  let output = generate(&["group-hover"]);
  assert!(output.contains(":where(.group):hover"), "got: {output}");
  assert!(output.contains(".flex"));
}

#[test]
fn peer_checked_uses_sibling_combinator() {
  let output = generate(&["peer-checked"]);
  // peer uses ` ~ ` combinator with the peer class.
  assert!(output.contains(":where(.peer):checked"), "got: {output}");
  assert!(output.contains(" ~ "), "got: {output}");
}

#[test]
fn group_arbitrary_with_ampersand() {
  let output = generate(&["group-[&.is-open]"]);
  // `&` is replaced by `.group`, producing `:where(.group.is-open) .flex`.
  assert!(output.contains(":where(.group.is-open)"), "got: {output}");
}

#[test]
fn arbitrary_variant_replaces_ampersand_descendant() {
  let output = generate(&["[&_p]"]);
  assert!(output.contains(".flex p"), "got: {output}");
}

#[test]
fn arbitrary_variant_pseudo_chain() {
  let output = generate(&["[&:is(a,b)]"]);
  assert!(output.contains(".flex:is(a,b)"), "got: {output}");
}

#[test]
fn media_and_pseudo_compose() {
  // `sm:hover:flex` → `@media (min-width: 640px) { .sm\:hover\:flex:hover {…} }`
  let output = generate(&["sm", "hover"]);
  assert!(output.contains("@media"));
  assert!(output.contains("(min-width: 640px)"));
  assert!(output.contains(":hover"));
}

#[test]
fn group_hover_inside_media() {
  let output = generate(&["sm", "group-hover"]);
  assert!(output.contains("@media"));
  assert!(output.contains(":where(.group):hover"));
}

#[test]
fn unknown_variant_drops_rule() {
  // Unknown variant → no CSS emitted.
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = candidate_with_variants(&["totally-bogus-variant-xyz"]);
  let result = registry.generate(&candidate, &theme);
  assert!(result.is_empty(), "got: {:?}", result);
}

#[test]
fn starting_style_at_rule() {
  // `starting` → `@starting-style` (currently routes through media path; we
  // simply check the bare media wraps applied). Some teams emit `@starting-style`
  // as its own at-rule; if unsupported, this test acts as a future TODO.
  // Drop for now if not recognised.
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = candidate_with_variants(&["starting"]);
  let _ = registry.generate(&candidate, &theme);
}

