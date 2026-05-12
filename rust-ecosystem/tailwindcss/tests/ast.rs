use farmfe_ecosystem_tailwindcss::ast::{
  optimize_ast, to_css, AtRule, AstNode, Declaration, StyleRule,
};

#[test]
fn test_style_rule_to_css() {
  let rule = StyleRule {
    selector: ".foo".to_string(),
    nodes: vec![AstNode::Declaration(Declaration {
      property: "color".to_string(),
      value: Some("red".to_string()),
      important: false,
    })],
  };
  let css = to_css(&[AstNode::Rule(rule)]);
  assert_eq!(css, ".foo {\n  color: red;\n}\n");
}

#[test]
fn test_at_rule_without_block_to_css() {
  let at_rule = AtRule {
    name: "@import".to_string(),
    params: "\"tailwindcss\"".to_string(),
    nodes: vec![],
  };
  let css = to_css(&[AstNode::AtRule(at_rule)]);
  assert_eq!(css, "@import \"tailwindcss\";\n");
}

#[test]
fn test_at_rule_with_block_to_css() {
  let at_rule = AtRule {
    name: "@media".to_string(),
    params: "screen".to_string(),
    nodes: vec![AstNode::Declaration(Declaration {
      property: "color".to_string(),
      value: Some("red".to_string()),
      important: false,
    })],
  };
  let css = to_css(&[AstNode::AtRule(at_rule)]);
  assert_eq!(css, "@media screen {\n  color: red;\n}\n");
}

#[test]
fn test_declaration_important_to_css() {
  let decl = Declaration {
    property: "color".to_string(),
    value: Some("red".to_string()),
    important: true,
  };
  let css = to_css(&[AstNode::Declaration(decl)]);
  assert_eq!(css, "color: red !important;\n");
}

#[test]
fn test_comment_to_css() {
  let css = to_css(&[AstNode::Comment(" license ".to_string())]);
  assert_eq!(css, "/* license */\n");
}

#[test]
fn test_nested_rules_to_css() {
  let inner = StyleRule {
    selector: "& .bar".to_string(),
    nodes: vec![AstNode::Declaration(Declaration {
      property: "color".to_string(),
      value: Some("blue".to_string()),
      important: false,
    })],
  };
  let outer = StyleRule {
    selector: ".foo".to_string(),
    nodes: vec![AstNode::Rule(inner)],
  };
  let css = to_css(&[AstNode::Rule(outer)]);
  assert_eq!(css, ".foo {\n  & .bar {\n    color: blue;\n  }\n}\n");
}

#[test]
fn test_optimize_ast_removes_empty_rules() {
  let rule = StyleRule {
    selector: ".empty".to_string(),
    nodes: vec![],
  };
  let result = optimize_ast(vec![AstNode::Rule(rule)]);
  assert!(result.is_empty());
}

#[test]
fn test_optimize_ast_preserves_empty_at_rules() {
  let at_rule = AtRule {
    name: "@layer".to_string(),
    params: "base".to_string(),
    nodes: vec![],
  };
  let result = optimize_ast(vec![AstNode::AtRule(at_rule)]);
  assert_eq!(result.len(), 1);
}

#[test]
fn test_optimize_ast_deduplicates_declarations() {
  let rule = StyleRule {
    selector: ".foo".to_string(),
    nodes: vec![
      AstNode::Declaration(Declaration {
        property: "color".to_string(),
        value: Some("red".to_string()),
        important: false,
      }),
      AstNode::Declaration(Declaration {
        property: "color".to_string(),
        value: Some("red".to_string()),
        important: false,
      }),
      AstNode::Declaration(Declaration {
        property: "margin".to_string(),
        value: Some("0".to_string()),
        important: false,
      }),
    ],
  };
  let result = optimize_ast(vec![AstNode::Rule(rule)]);
  if let AstNode::Rule(r) = &result[0] {
    assert_eq!(r.nodes.len(), 2);
  } else {
    panic!("expected rule");
  }
}
