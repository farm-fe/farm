use farmfe_ecosystem_tailwindcss::apply::substitute_at_apply;
use farmfe_ecosystem_tailwindcss::ast::{AstNode, AtRule, Declaration, StyleRule, to_css};
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
