use std::collections::HashMap;
use farmfe_ecosystem_tailwindcss::ast::{AstNode, Declaration, StyleRule};
use farmfe_ecosystem_tailwindcss::functions::substitute_css_functions;
use farmfe_ecosystem_tailwindcss::theme::Theme;

#[test]
fn test_substitute_theme_function() {
  let mut variables = HashMap::new();
  variables.insert("--color-red-500".to_string(), "#ef4444".to_string());
  let theme = Theme {
    variables,
    keyframes: HashMap::new(),
  };

  let decl = Declaration {
    property: "color".to_string(),
    value: Some("theme(colors.red.500)".to_string()),
    important: false,
  };
  let rule = StyleRule {
    selector: ".foo".to_string(),
    nodes: vec![AstNode::Declaration(decl)],
  };

  let result = substitute_css_functions(vec![AstNode::Rule(rule)], &theme);

  if let AstNode::Rule(r) = &result[0] {
    if let AstNode::Declaration(d) = &r.nodes[0] {
      assert_eq!(d.value.as_deref(), Some("#ef4444"));
    } else {
      panic!("expected declaration");
    }
  } else {
    panic!("expected rule");
  }
}

#[test]
fn test_no_substitution_without_theme_function() {
  let theme = Theme::default();
  let decl = Declaration {
    property: "color".to_string(),
    value: Some("red".to_string()),
    important: false,
  };
  let rule = StyleRule {
    selector: ".foo".to_string(),
    nodes: vec![AstNode::Declaration(decl)],
  };
  let result = substitute_css_functions(vec![AstNode::Rule(rule)], &theme);

  if let AstNode::Rule(r) = &result[0] {
    if let AstNode::Declaration(d) = &r.nodes[0] {
      assert_eq!(d.value.as_deref(), Some("red"));
    } else {
      panic!("expected declaration");
    }
  } else {
    panic!("expected rule");
  }
}
