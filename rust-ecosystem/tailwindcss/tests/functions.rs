use farmfe_ecosystem_tailwindcss::ast::{AstNode, Declaration, StyleRule};
use farmfe_ecosystem_tailwindcss::functions::substitute_css_functions;
use farmfe_ecosystem_tailwindcss::theme::Theme;

#[test]
fn unicode_function_identifiers_are_not_theme_calls() {
  let mut theme = Theme::new();
  theme.insert("--color-red-500", "#ef4444");
  let value = "é--theme(--font-sans) étheme(colors.red.500)";
  let result = substitute_css_functions(
    vec![AstNode::Declaration(Declaration {
      property: "font-family".into(),
      value: Some(value.into()),
      important: false,
    })],
    &theme,
  );
  let AstNode::Declaration(decl) = &result[0] else {
    panic!("expected declaration");
  };
  assert_eq!(decl.value.as_deref(), Some(value));
}

#[test]
fn theme_roots_stay_at_the_definition_instead_of_an_unrelated_layer() {
  use farmfe_ecosystem_tailwindcss::{CompilerOptions, compile};
  let mut compiler = compile(
    r#"
@media (min-width: 1000px) { @layer theme { .unrelated { color: red; } @theme reference { --color-reference: blue; } } }
@layer theme { :root { --color-brand: pink; } }
@layer theme { @theme { --color-brand: #123456; } }
@layer utilities { @tailwind utilities; }
"#,
    CompilerOptions::default(),
  )
  .expect("valid theme input");
  let css = compiler.build(&["bg-brand".into()]);
  let conditional_end = css
    .find("@layer theme {\n  :root")
    .expect("unconditional theme layer");
  assert!(!css[..conditional_end].contains("#123456"), "{css}");
  assert!(
    css.find("--color-brand: pink").unwrap() < css.find("--color-brand: #123456").unwrap(),
    "{css}"
  );
}

#[test]
fn internal_theme_preserves_nested_fallbacks_and_quoted_text() {
  let nodes = vec![AstNode::Declaration(Declaration {
    property: "font-family".into(),
    value: Some(
      r#"--theme(--font-sans, var(--fallback, "字体 (sans)")), "--theme(literal)""#.into(),
    ),
    important: false,
  })];
  let result = substitute_css_functions(nodes, &Theme::default());
  let AstNode::Declaration(decl) = &result[0] else {
    panic!("expected declaration");
  };
  assert_eq!(
    decl.value.as_deref(),
    Some(r#"var(--font-sans, var(--fallback, "字体 (sans)")), "--theme(literal)""#)
  );
}

#[test]
fn unknown_theme_calls_are_preserved_without_restarting_the_scan() {
  let mut theme = Theme::new();
  theme.insert("--color-red-500", "#ef4444");
  let nodes = vec![AstNode::Declaration(Declaration {
    property: "color".into(),
    value: Some("theme(colors.missing) theme(colors.red.500)".into()),
    important: false,
  })];
  let result = substitute_css_functions(nodes, &theme);
  let AstNode::Declaration(decl) = &result[0] else {
    panic!("expected declaration");
  };
  assert_eq!(decl.value.as_deref(), Some("theme(colors.missing) #ef4444"));
}

#[test]
fn test_substitute_theme_function() {
  let mut theme = Theme::new();
  theme.insert("--color-red-500", "#ef4444");

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
