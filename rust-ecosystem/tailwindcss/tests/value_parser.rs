//! Verbatim port of upstream `packages/tailwindcss/src/value-parser.test.ts`
//! (parse + toCss describes only; the `walk` describe relies on a separate
//! value-AST walker not yet ported).

use farmfe_ecosystem_tailwindcss::value_parser::{parse, to_css, ValueAstNode as N};

fn word(v: &str) -> N {
  N::Word(v.to_string())
}
fn sep(v: &str) -> N {
  N::Separator(v.to_string())
}
fn func(name: &str, nodes: Vec<N>) -> N {
  N::Function { value: name.to_string(), nodes }
}

// ----- parse ------------------------------------------------------------------

#[test]
fn parse_a_value() {
  assert_eq!(parse("123px"), vec![word("123px")]);
}

#[test]
fn parse_a_string_value() {
  assert_eq!(parse("'hello world'"), vec![word("'hello world'")]);
}

#[test]
fn parse_a_list() {
  assert_eq!(parse("hello world"), vec![word("hello"), sep(" "), word("world")]);
}

#[test]
fn parse_string_containing_parens() {
  assert_eq!(parse("'hello ( world )'"), vec![word("'hello ( world )'")]);
}

#[test]
fn parse_function_no_args() {
  assert_eq!(parse("theme()"), vec![func("theme", vec![])]);
}

#[test]
fn parse_function_single_arg() {
  assert_eq!(parse("theme(foo)"), vec![func("theme", vec![word("foo")])]);
}

#[test]
fn parse_function_single_string_arg() {
  assert_eq!(parse("theme('foo')"), vec![func("theme", vec![word("'foo'")])]);
}

#[test]
fn parse_function_multiple_args() {
  assert_eq!(
    parse("theme(foo, bar)"),
    vec![func("theme", vec![word("foo"), sep(", "), word("bar")])]
  );
}

#[test]
fn parse_function_multiple_args_across_lines() {
  assert_eq!(
    parse("theme(\n\tfoo,\n\tbar\n)"),
    vec![func(
      "theme",
      vec![
        sep("\n\t"),
        word("foo"),
        sep(",\n\t"),
        word("bar"),
        sep("\n"),
      ]
    )]
  );
}

#[test]
fn parse_function_nested_args() {
  assert_eq!(
    parse("theme(foo, theme(bar))"),
    vec![func(
      "theme",
      vec![word("foo"), sep(", "), func("theme", vec![word("bar")])]
    )]
  );
}

#[test]
fn parse_function_nested_args_with_slash() {
  assert_eq!(
    parse("theme(colors.red.500/var(--opacity))"),
    vec![func(
      "theme",
      vec![
        word("colors.red.500"),
        word("/"),
        func("var", vec![word("--opacity")]),
      ]
    )]
  );
}

#[test]
fn parse_calculations() {
  assert_eq!(
    parse("calc((1 + 2) * 3)"),
    vec![func(
      "calc",
      vec![
        func(
          "",
          vec![
            word("1"),
            sep(" "),
            word("+"),
            sep(" "),
            word("2"),
          ]
        ),
        sep(" "),
        word("*"),
        sep(" "),
        word("3"),
      ]
    )]
  );
}

#[test]
fn parse_media_query_params_with_functions() {
  assert_eq!(
    parse("(min-width: 600px) and (max-width:theme(colors.red.500)) and (theme(--breakpoint-sm)<width<=theme(--breakpoint-md))"),
    vec![
      func("", vec![word("min-width"), sep(": "), word("600px")]),
      sep(" "),
      word("and"),
      sep(" "),
      func("", vec![
        word("max-width"),
        sep(":"),
        func("theme", vec![word("colors.red.500")]),
      ]),
      sep(" "),
      word("and"),
      sep(" "),
      func("", vec![
        func("theme", vec![word("--breakpoint-sm")]),
        sep("<"),
        word("width"),
        sep("<="),
        func("theme", vec![word("--breakpoint-md")]),
      ]),
    ]
  );
}

#[test]
fn parse_does_not_error_on_extra_close_paren() {
  assert_eq!(
    parse("calc(1 + 2))"),
    vec![func(
      "calc",
      vec![word("1"), sep(" "), word("+"), sep(" "), word("2")]
    )]
  );
}

// ----- to_css -----------------------------------------------------------------

#[test]
fn to_css_pretty_prints_calculations() {
  assert_eq!(to_css(&parse("calc((1 + 2) * 3)")), "calc((1 + 2) * 3)");
}

#[test]
fn to_css_pretty_prints_nested_function_calls() {
  assert_eq!(to_css(&parse("theme(foo, theme(bar))")), "theme(foo, theme(bar))");
}

#[test]
fn to_css_pretty_prints_media_query_params() {
  assert_eq!(
    to_css(&parse("(min-width: 600px) and (max-width:theme(colors.red.500))")),
    "(min-width: 600px) and (max-width:theme(colors.red.500))"
  );
}

#[test]
fn to_css_preserves_multiple_spaces() {
  assert_eq!(to_css(&parse("foo(   bar  )")), "foo(   bar  )");
}
