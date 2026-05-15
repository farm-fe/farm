use farmfe_ecosystem_tailwindcss::ast::{
  optimize_ast, to_css, AstNode, AtRule, Declaration, StyleRule,
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

// ── Phase 12: merge adjacent identical at-rules ───────────────────────────────

fn make_rule(selector: &str, prop: &str, value: &str) -> AstNode {
  AstNode::Rule(StyleRule {
    selector: selector.to_string(),
    nodes: vec![AstNode::Declaration(Declaration {
      property: prop.to_string(),
      value: Some(value.to_string()),
      important: false,
    })],
  })
}

#[test]
fn test_optimize_ast_merges_adjacent_identical_at_rules() {
  let media_a = AtRule {
    name: "@media".to_string(),
    params: "(min-width: 640px)".to_string(),
    nodes: vec![make_rule(".sm\\:flex", "display", "flex")],
  };
  let media_b = AtRule {
    name: "@media".to_string(),
    params: "(min-width: 640px)".to_string(),
    nodes: vec![make_rule(".sm\\:hidden", "display", "none")],
  };
  let result = optimize_ast(vec![AstNode::AtRule(media_a), AstNode::AtRule(media_b)]);
  assert_eq!(
    result.len(),
    1,
    "two same-params @media blocks should merge"
  );
  if let AstNode::AtRule(merged) = &result[0] {
    assert_eq!(merged.nodes.len(), 2);
  } else {
    panic!("expected at-rule");
  }
}

#[test]
fn test_optimize_ast_does_not_merge_different_params() {
  let media_a = AtRule {
    name: "@media".to_string(),
    params: "(min-width: 640px)".to_string(),
    nodes: vec![make_rule(".a", "color", "red")],
  };
  let media_b = AtRule {
    name: "@media".to_string(),
    params: "(min-width: 768px)".to_string(),
    nodes: vec![make_rule(".b", "color", "blue")],
  };
  let result = optimize_ast(vec![AstNode::AtRule(media_a), AstNode::AtRule(media_b)]);
  assert_eq!(result.len(), 2);
}

#[test]
fn test_optimize_ast_does_not_merge_non_adjacent_at_rules() {
  let media_a = AtRule {
    name: "@media".to_string(),
    params: "(min-width: 640px)".to_string(),
    nodes: vec![make_rule(".a", "color", "red")],
  };
  let between = make_rule(".between", "color", "green");
  let media_b = AtRule {
    name: "@media".to_string(),
    params: "(min-width: 640px)".to_string(),
    nodes: vec![make_rule(".b", "color", "blue")],
  };
  let result = optimize_ast(vec![
    AstNode::AtRule(media_a),
    between,
    AstNode::AtRule(media_b),
  ]);
  assert_eq!(result.len(), 3);
}

#[test]
fn test_optimize_ast_merges_nested_at_rules() {
  // @supports { @media (a) { .x {} } } followed by @supports { @media (a) { .y {} } }
  // should merge at both levels.
  let outer_a = AtRule {
    name: "@supports".to_string(),
    params: "(display: grid)".to_string(),
    nodes: vec![AstNode::AtRule(AtRule {
      name: "@media".to_string(),
      params: "(min-width: 640px)".to_string(),
      nodes: vec![make_rule(".x", "color", "red")],
    })],
  };
  let outer_b = AtRule {
    name: "@supports".to_string(),
    params: "(display: grid)".to_string(),
    nodes: vec![AstNode::AtRule(AtRule {
      name: "@media".to_string(),
      params: "(min-width: 640px)".to_string(),
      nodes: vec![make_rule(".y", "color", "blue")],
    })],
  };
  let result = optimize_ast(vec![AstNode::AtRule(outer_a), AstNode::AtRule(outer_b)]);
  assert_eq!(result.len(), 1);
  if let AstNode::AtRule(outer) = &result[0] {
    assert_eq!(
      outer.nodes.len(),
      1,
      "inner @media blocks should also merge"
    );
    if let AstNode::AtRule(inner) = &outer.nodes[0] {
      assert_eq!(inner.nodes.len(), 2);
    } else {
      panic!("expected nested at-rule");
    }
  } else {
    panic!("expected outer at-rule");
  }
}
