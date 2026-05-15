use farmfe_ecosystem_tailwindcss::apply::substitute_at_apply;
use farmfe_ecosystem_tailwindcss::ast::{AstNode, AtRule, StyleRule, to_css};
use farmfe_ecosystem_tailwindcss::design_system::DesignSystem;
use farmfe_ecosystem_tailwindcss::theme::Theme;
use farmfe_ecosystem_tailwindcss::utilities::UtilityRegistry;
use farmfe_ecosystem_tailwindcss::variants::VariantRegistry;

fn make_design_system() -> DesignSystem {
  DesignSystem {
    theme: Theme::default(),
    utilities: UtilityRegistry::builtin(),
    variants: VariantRegistry::builtin(),
  }
}

#[test]
fn test_substitute_simple_apply() {
  let ds = make_design_system();

  let rule = StyleRule {
    selector: ".foo".to_string(),
    nodes: vec![AstNode::AtRule(AtRule {
      name: "@apply".to_string(),
      params: "flex".to_string(),
      nodes: vec![],
    })],
  };

  let result = substitute_at_apply(vec![AstNode::Rule(rule)], &ds).unwrap();
  let output = to_css(&result);

  assert!(!output.contains("@apply"));
  assert!(output.contains("display: flex"));
}

#[test]
fn test_substitute_apply_with_multiple_utilities() {
  let ds = make_design_system();

  let rule = StyleRule {
    selector: ".foo".to_string(),
    nodes: vec![AstNode::AtRule(AtRule {
      name: "@apply".to_string(),
      params: "flex items-center".to_string(),
      nodes: vec![],
    })],
  };

  let result = substitute_at_apply(vec![AstNode::Rule(rule)], &ds).unwrap();
  let output = to_css(&result);

  assert!(!output.contains("@apply"));
  assert!(output.contains("display: flex"));
  assert!(output.contains("align-items: center"));
}

#[test]
fn test_apply_with_unknown_utility_errors() {
  let ds = make_design_system();

  let rule = StyleRule {
    selector: ".foo".to_string(),
    nodes: vec![AstNode::AtRule(AtRule {
      name: "@apply".to_string(),
      params: "nonexistent-utility-xyz".to_string(),
      nodes: vec![],
    })],
  };

  let result = substitute_at_apply(vec![AstNode::Rule(rule)], &ds);
  assert!(result.is_err());
}

#[test]
fn test_apply_inside_keyframes_is_rejected() {
  let ds = make_design_system();

  let at_rule = AtRule {
    name: "@keyframes".to_string(),
    params: "spin".to_string(),
    nodes: vec![AstNode::AtRule(AtRule {
      name: "@apply".to_string(),
      params: "flex".to_string(),
      nodes: vec![],
    })],
  };

  let result = substitute_at_apply(vec![AstNode::AtRule(at_rule)], &ds);
  assert!(result.is_err());
}

// ── Phase 17: variant + at-rule + important parity ─────────────────────────

fn apply_in_rule(ds: &DesignSystem, apply_params: &str) -> String {
  let rule = StyleRule {
    selector: ".foo".to_string(),
    nodes: vec![AstNode::AtRule(AtRule {
      name: "@apply".to_string(),
      params: apply_params.to_string(),
      nodes: vec![],
    })],
  };
  let result = substitute_at_apply(vec![AstNode::Rule(rule)], ds).unwrap();
  to_css(&result)
}

#[test]
fn test_apply_with_pseudo_variant_produces_nested_ampersand_rule() {
  let ds = make_design_system();
  let css = apply_in_rule(&ds, "hover:flex");
  // Expect a nested `&:hover { display: flex }` rule inside `.foo`.
  assert!(!css.contains("@apply"), "css={}", css);
  assert!(css.contains("&:hover"), "css={}", css);
  assert!(css.contains("display: flex"), "css={}", css);
}

#[test]
fn test_apply_with_media_variant_produces_nested_at_rule() {
  let ds = make_design_system();
  let css = apply_in_rule(&ds, "md:flex");
  // Expect a nested @media wrapping a `&` rule.
  assert!(!css.contains("@apply"), "css={}", css);
  assert!(css.contains("@media"), "css={}", css);
  assert!(css.contains("display: flex"), "css={}", css);
}

#[test]
fn test_apply_with_important_modifier_marks_declarations() {
  let ds = make_design_system();
  let css = apply_in_rule(&ds, "flex!");
  assert!(!css.contains("@apply"), "css={}", css);
  assert!(css.contains("display: flex"), "css={}", css);
  assert!(css.contains("!important"), "css={}", css);
}

#[test]
fn test_apply_with_dark_variant_produces_nested_rule() {
  let ds = make_design_system();
  let css = apply_in_rule(&ds, "dark:flex");
  assert!(!css.contains("@apply"), "css={}", css);
  // dark may be either a media query or a nested selector — at minimum the
  // class prefix must have been rewritten to `&`.
  assert!(css.contains('&') || css.contains("@media"), "css={}", css);
  assert!(css.contains("display: flex"), "css={}", css);
}

