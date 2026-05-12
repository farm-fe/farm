use farmfe_ecosystem_tailwindcss::ast::{AstNode, Declaration, StyleRule};
use farmfe_ecosystem_tailwindcss::walk::{walk, WalkAction};

#[test]
fn test_walk_visits_all_nodes() {
  let decl = AstNode::Declaration(Declaration {
    property: "color".to_string(),
    value: Some("red".to_string()),
    important: false,
  });
  let mut visited = 0;
  walk(
    vec![decl.clone(), decl.clone()],
    &mut |_node: &AstNode, _path, _depth| {
      visited += 1;
      WalkAction::Continue
    },
  );
  assert_eq!(visited, 2);
}

#[test]
fn test_walk_skip_children() {
  let inner = AstNode::Declaration(Declaration {
    property: "color".to_string(),
    value: Some("red".to_string()),
    important: false,
  });
  let rule = AstNode::Rule(StyleRule {
    selector: ".foo".to_string(),
    nodes: vec![inner],
  });
  let mut visited_selectors = Vec::new();
  walk(vec![rule], &mut |node: &AstNode, _path, _depth| {
    if let AstNode::Rule(r) = node {
      visited_selectors.push(r.selector.clone());
      return WalkAction::Skip;
    }
    visited_selectors.push("other".to_string());
    WalkAction::Continue
  });
  // Only the rule was visited, not its children
  assert_eq!(visited_selectors, vec![".foo"]);
}

#[test]
fn test_walk_replace_node() {
  let decl = AstNode::Declaration(Declaration {
    property: "color".to_string(),
    value: Some("red".to_string()),
    important: false,
  });
  let replacement = AstNode::Declaration(Declaration {
    property: "color".to_string(),
    value: Some("blue".to_string()),
    important: false,
  });
  let rule_nodes = vec![decl];
  let result = walk(
    rule_nodes,
    &mut |node: &AstNode, _path, _depth| {
      if let AstNode::Declaration(d) = node {
        if d.value.as_deref() == Some("red") {
          return WalkAction::Replace(vec![replacement.clone()]);
        }
      }
      WalkAction::Continue
    },
  );
  assert_eq!(result.len(), 1);
  if let AstNode::Declaration(d) = &result[0] {
    assert_eq!(d.value.as_deref(), Some("blue"));
  } else {
    panic!("expected declaration");
  }
}

#[test]
fn test_walk_stop() {
  let d1 = AstNode::Declaration(Declaration {
    property: "a".to_string(),
    value: Some("1".to_string()),
    important: false,
  });
  let d2 = AstNode::Declaration(Declaration {
    property: "b".to_string(),
    value: Some("2".to_string()),
    important: false,
  });
  let mut visited = Vec::new();
  walk(vec![d1, d2], &mut |node: &AstNode, _path, _depth| {
    if let AstNode::Declaration(d) = node {
      visited.push(d.property.clone());
      if d.property == "a" {
        return WalkAction::Stop;
      }
    }
    WalkAction::Continue
  });
  assert_eq!(visited, vec!["a"]);
}
