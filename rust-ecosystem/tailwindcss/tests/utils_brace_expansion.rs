//! Ported from upstream
//! `packages/tailwindcss/src/utils/brace-expansion.test.ts`.

use farmfe_ecosystem_tailwindcss::utils::brace_expansion::{expand, BraceExpansionError};

fn sorted(mut v: Vec<String>) -> Vec<String> {
  v.sort();
  v
}

fn s(items: &[&str]) -> Vec<String> {
  items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn no_braces_returns_input() {
  assert_eq!(sorted(expand("a/b/c").unwrap()), sorted(s(&["a/b/c"])));
}

#[test]
fn comma_groups() {
  assert_eq!(
    sorted(expand("a/{x,y,z}/b").unwrap()),
    sorted(s(&["a/x/b", "a/y/b", "a/z/b"]))
  );
  assert_eq!(
    sorted(expand("{a,b}/{x,y}").unwrap()),
    sorted(s(&["a/x", "a/y", "b/x", "b/y"]))
  );
  assert_eq!(
    sorted(expand("{{xs,sm,md,lg}:,}hidden").unwrap()),
    sorted(s(&[
      "xs:hidden",
      "sm:hidden",
      "md:hidden",
      "lg:hidden",
      "hidden"
    ]))
  );
}

#[test]
fn numeric_ranges() {
  assert_eq!(
    sorted(expand("a/{0..5}/b").unwrap()),
    sorted(s(&["a/0/b", "a/1/b", "a/2/b", "a/3/b", "a/4/b", "a/5/b"]))
  );
  assert_eq!(
    sorted(expand("a/{-5..0}/b").unwrap()),
    sorted(s(&[
      "a/-5/b", "a/-4/b", "a/-3/b", "a/-2/b", "a/-1/b", "a/0/b",
    ]))
  );
  assert_eq!(
    sorted(expand("a/{0..-5}/b").unwrap()),
    sorted(s(&[
      "a/0/b", "a/-1/b", "a/-2/b", "a/-3/b", "a/-4/b", "a/-5/b",
    ]))
  );
  assert_eq!(
    sorted(expand("a/{0..10..5}/b").unwrap()),
    sorted(s(&["a/0/b", "a/5/b", "a/10/b"]))
  );
  assert_eq!(
    sorted(expand("a/{0..10..-5}/b").unwrap()),
    sorted(s(&["a/10/b", "a/5/b", "a/0/b"]))
  );
  assert_eq!(
    sorted(expand("a/{10..0..5}/b").unwrap()),
    sorted(s(&["a/10/b", "a/5/b", "a/0/b"]))
  );
  assert_eq!(
    sorted(expand("a/{10..0..-5}/b").unwrap()),
    sorted(s(&["a/0/b", "a/5/b", "a/10/b"]))
  );
}

#[test]
fn numeric_range_padding_is_not_supported() {
  assert_eq!(
    sorted(expand("a/{00..05}/b").unwrap()),
    sorted(s(&["a/0/b", "a/1/b", "a/2/b", "a/3/b", "a/4/b", "a/5/b"]))
  );
  assert_eq!(
    sorted(expand("a{001..9}b").unwrap()),
    sorted(s(&[
      "a1b", "a2b", "a3b", "a4b", "a5b", "a6b", "a7b", "a8b", "a9b",
    ]))
  );
}

#[test]
fn numeric_range_with_step() {
  assert_eq!(
    sorted(expand("a/{0..5..2}/b").unwrap()),
    sorted(s(&["a/0/b", "a/2/b", "a/4/b"]))
  );
  assert_eq!(
    sorted(expand("bg-red-{100..900..100}").unwrap()),
    sorted(s(&[
      "bg-red-100",
      "bg-red-200",
      "bg-red-300",
      "bg-red-400",
      "bg-red-500",
      "bg-red-600",
      "bg-red-700",
      "bg-red-800",
      "bg-red-900",
    ]))
  );
}

#[test]
fn nested_braces() {
  assert_eq!(
    sorted(expand("a{b,c,/{x,y}}/e").unwrap()),
    sorted(s(&["ab/e", "ac/e", "a/x/e", "a/y/e"]))
  );
  assert_eq!(
    sorted(expand("a{b,c,/{x,y},{z,w}}/e").unwrap()),
    sorted(s(&["ab/e", "ac/e", "a/x/e", "a/y/e", "az/e", "aw/e",]))
  );
  assert_eq!(
    sorted(expand("a{b,c,/{x,y},{0..2}}/e").unwrap()),
    sorted(s(&[
      "ab/e", "ac/e", "a/x/e", "a/y/e", "a0/e", "a1/e", "a2/e",
    ]))
  );
  assert_eq!(
    sorted(expand("bg-red-{50,{100..900..100},950}").unwrap()),
    sorted(s(&[
      "bg-red-50",
      "bg-red-100",
      "bg-red-200",
      "bg-red-300",
      "bg-red-400",
      "bg-red-500",
      "bg-red-600",
      "bg-red-700",
      "bg-red-800",
      "bg-red-900",
      "bg-red-950",
    ]))
  );
}

#[test]
fn does_not_expand_decimal_ranges() {
  assert_eq!(
    sorted(expand("{1.1..2.2}").unwrap()),
    sorted(s(&["1.1..2.2"]))
  );
}

#[test]
fn throws_on_unbalanced_braces() {
  let err = expand("a{b,c{d,e},{f,g}h}x{y,z").unwrap_err();
  match err {
    BraceExpansionError::Unbalanced { pattern } => {
      assert_eq!(pattern, "x{y,z");
    }
    other => panic!("unexpected error: {:?}", other),
  }
}

#[test]
fn throws_when_step_is_zero() {
  assert_eq!(
    expand("a{0..5..0}/b").unwrap_err(),
    BraceExpansionError::ZeroStep
  );
}
