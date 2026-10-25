use farmfe_ecosystem_tailwindcss::{
  ast::to_css, candidate::parse_candidate, theme::Theme, utilities::UtilityRegistry,
};

fn render(input: &str) -> String {
  let candidate = parse_candidate(input).expect("valid class candidate");
  to_css(&UtilityRegistry::builtin().generate(&candidate, &Theme::with_defaults()))
}

#[test]
fn responsive_selector_matches_the_actual_prefixed_html_class() {
  let css = render("md:grid-cols-2");
  assert!(css.contains(".md\\:grid-cols-2"), "{css}");
  assert!(!css.contains(".grid-cols-2 {"), "{css}");
}

#[test]
fn stacked_state_variants_keep_the_full_class_identity() {
  let css = render("sm:hover:flex");
  assert!(css.contains(".sm\\:hover\\:flex:hover"), "{css}");
}

#[test]
fn arbitrary_property_variant_keeps_its_original_class_identity() {
  let css = render("hover:[color:red]");
  assert!(css.contains(".hover\\:\\[color\\:red\\]:hover"), "{css}");
}

#[test]
fn numeric_breakpoint_prefix_is_a_valid_css_identifier() {
  let css = render("2xl:flex");
  assert!(css.contains(".\\32 xl\\:flex"), "{css}");
}

#[test]
fn arbitrary_variant_punctuation_is_escaped_in_the_class_only() {
  let css = render("[&>p]:flex");
  assert!(css.contains(".\\[\\&\\>p\\]\\:flex"), "{css}");
  assert!(css.contains(">p"), "{css}");
}

#[test]
fn negative_important_variant_keeps_prefix_and_suffix() {
  let css = render("md:-mt-4!");
  assert!(css.contains(".md\\:-mt-4\\!"), "{css}");
  assert!(css.contains("!important"), "{css}");
}
