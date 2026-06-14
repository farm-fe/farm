use farmfe_ecosystem_tailwindcss::parser::parse;

#[test]
fn test_parse_simple_rule() {
  let css = ".foo { color: red; }";
  let ast = parse(css);
  assert!(!ast.is_empty());
}

#[test]
fn test_parse_at_rule_semicolon() {
  let css = r#"@import "tailwindcss";"#;
  let ast = parse(css);
  assert!(!ast.is_empty());
}

#[test]
fn test_parse_at_rule_block() {
  let css = "@media screen { .foo { color: red; } }";
  let ast = parse(css);
  assert!(!ast.is_empty());
}

#[test]
fn test_parse_declaration_with_important() {
  let css = ".foo { color: red !important; }";
  let ast = parse(css);
  assert!(!ast.is_empty());
}

#[test]
fn test_parse_comment() {
  let css = "/* license */\n.foo { color: red; }";
  let ast = parse(css);
  assert!(!ast.is_empty());
}

#[test]
fn test_parse_empty_input() {
  let css = "";
  let ast = parse(css);
  assert!(ast.is_empty());
}

#[test]
fn test_parse_multiple_rules() {
  let css = ".foo { color: red; }\n.bar { color: blue; }";
  let ast = parse(css);
  assert_eq!(ast.len(), 2);
}

#[test]
fn test_parse_nested_at_rule() {
  let css = "@layer base {\n  .foo { color: red; }\n}";
  let ast = parse(css);
  assert!(!ast.is_empty());
}
