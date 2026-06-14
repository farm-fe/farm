//! Ported from upstream `packages/tailwindcss/src/utils/segment.test.ts`.

use farmfe_ecosystem_tailwindcss::utils::segment::segment;

#[test]
fn single_segment_when_separator_absent() {
  assert_eq!(segment("foo", ':'), vec!["foo".to_string()]);
}

#[test]
fn split_by_separator() {
  assert_eq!(
    segment("foo:bar:baz", ':'),
    vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
  );
}

#[test]
fn no_split_inside_parens() {
  assert_eq!(
    segment("a:(b:c):d", ':'),
    vec!["a".to_string(), "(b:c)".to_string(), "d".to_string()]
  );
}

#[test]
fn no_split_inside_brackets() {
  assert_eq!(
    segment("a:[b:c]:d", ':'),
    vec!["a".to_string(), "[b:c]".to_string(), "d".to_string()]
  );
}

#[test]
fn no_split_inside_curlies() {
  assert_eq!(
    segment("a:{b:c}:d", ':'),
    vec!["a".to_string(), "{b:c}".to_string(), "d".to_string()]
  );
}

#[test]
fn no_split_inside_double_quotes() {
  assert_eq!(
    segment("a:\"b:c\":d", ':'),
    vec!["a".to_string(), "\"b:c\"".to_string(), "d".to_string()]
  );
}

#[test]
fn no_split_inside_single_quotes() {
  assert_eq!(
    segment("a:'b:c':d", ':'),
    vec!["a".to_string(), "'b:c'".to_string(), "d".to_string()]
  );
}

#[test]
fn unbalanced_double_quotes_do_not_crash() {
  assert_eq!(
    segment("a:\"b:c:d", ':'),
    vec!["a".to_string(), "\"b:c:d".to_string()]
  );
}

#[test]
fn unbalanced_single_quotes_do_not_crash() {
  assert_eq!(
    segment("a:'b:c:d", ':'),
    vec!["a".to_string(), "'b:c:d".to_string()]
  );
}

#[test]
fn skip_escaped_double_quotes() {
  assert_eq!(
    segment(r#"a:"b:c\":d":e"#, ':'),
    vec!["a".to_string(), r#""b:c\":d""#.to_string(), "e".to_string()]
  );
}

#[test]
fn skip_escaped_single_quotes() {
  assert_eq!(
    segment(r"a:'b:c\':d':e", ':'),
    vec!["a".to_string(), r"'b:c\':d'".to_string(), "e".to_string()]
  );
}

#[test]
fn split_by_escape_sequence_when_separator_is_backslash() {
  assert_eq!(
    segment(r"a\b\c\d", '\\'),
    vec![
      "a".to_string(),
      "b".to_string(),
      "c".to_string(),
      "d".to_string()
    ]
  );
  assert_eq!(
    segment(r"a\(b\c)\d", '\\'),
    vec!["a".to_string(), r"(b\c)".to_string(), "d".to_string()]
  );
  assert_eq!(
    segment(r"a\[b\c]\d", '\\'),
    vec!["a".to_string(), r"[b\c]".to_string(), "d".to_string()]
  );
  assert_eq!(
    segment(r"a\{b\c}\d", '\\'),
    vec!["a".to_string(), r"{b\c}".to_string(), "d".to_string()]
  );
}
