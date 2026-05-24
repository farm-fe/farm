//! Ported from upstream `packages/tailwindcss/src/utils/compare.test.ts`.

use farmfe_ecosystem_tailwindcss::utils::compare::compare;
use std::cmp::Ordering;

fn sign(o: Ordering) -> i32 {
  match o {
    Ordering::Less => -1,
    Ordering::Equal => 0,
    Ordering::Greater => 1,
  }
}

#[test]
fn equal_strings() {
  assert_eq!(sign(compare("abc", "abc")), 0);
}

#[test]
fn shorter_string_first() {
  assert_eq!(sign(compare("abc", "abcd")), -1);
}

#[test]
fn longer_string_last() {
  assert_eq!(sign(compare("abcd", "abc")), 1);
}

#[test]
fn numeric_compare() {
  assert_eq!(sign(compare("1", "1")), 0);
  assert_eq!(sign(compare("1", "2")), -1);
  assert_eq!(sign(compare("2", "1")), 1);
  assert_eq!(sign(compare("1", "10")), -1);
  assert_eq!(sign(compare("10", "1")), 1);
}

#[test]
fn numeric_compare_different_lengths() {
  assert_eq!(sign(compare("75", "700")), -1);
  assert_eq!(sign(compare("700", "75")), 1);
  assert_eq!(sign(compare("75", "770")), -1);
  assert_eq!(sign(compare("770", "75")), 1);
}

#[test]
fn sort_strings_with_numbers() {
  let mut input: Vec<&str> = vec![
    "p-0", "p-0.5", "p-1", "p-1.5", "p-10", "p-12", "p-2", "p-20", "p-21",
  ];
  input.sort_by(|a, b| compare(a, b));
  assert_eq!(
    input,
    vec!["p-0", "p-0.5", "p-1", "p-1.5", "p-2", "p-10", "p-12", "p-20", "p-21"]
  );
}

#[test]
fn sort_strings_with_modifiers() {
  let mut input: Vec<&str> = vec![
    "text-5xl",
    "text-6xl",
    "text-6xl/loose",
    "text-6xl/wide",
    "bg-red-500",
    "bg-red-500/50",
    "bg-red-500/70",
    "bg-red-500/60",
    "bg-red-50",
    "bg-red-50/50",
    "bg-red-50/70",
    "bg-red-50/60",
  ];
  input.sort_by(|a, b| compare(a, b));
  assert_eq!(
    input,
    vec![
      "bg-red-50",
      "bg-red-50/50",
      "bg-red-50/60",
      "bg-red-50/70",
      "bg-red-500",
      "bg-red-500/50",
      "bg-red-500/60",
      "bg-red-500/70",
      "text-5xl",
      "text-6xl",
      "text-6xl/loose",
      "text-6xl/wide",
    ]
  );
}

#[test]
fn sort_with_multiple_numbers_per_string() {
  let mut input: Vec<&str> = vec![
    "foo-123-bar-456-baz-789",
    "foo-123-bar-456-baz-788",
    "foo-123-bar-456-baz-790",
    "foo-123-bar-455-baz-789",
    "foo-123-bar-456-baz-789",
    "foo-123-bar-457-baz-789",
    "foo-123-bar-456-baz-789",
    "foo-124-bar-456-baz-788",
    "foo-125-bar-456-baz-790",
    "foo-126-bar-455-baz-789",
    "foo-127-bar-456-baz-789",
    "foo-128-bar-457-baz-789",
    "foo-1-bar-2-baz-3",
    "foo-12-bar-34-baz-45",
    "foo-12-bar-34-baz-4",
    "foo-12-bar-34-baz-456",
  ];
  input.sort_by(|a, b| compare(a, b));
  assert_eq!(
    input,
    vec![
      "foo-1-bar-2-baz-3",
      "foo-12-bar-34-baz-4",
      "foo-12-bar-34-baz-45",
      "foo-12-bar-34-baz-456",
      "foo-123-bar-455-baz-789",
      "foo-123-bar-456-baz-788",
      "foo-123-bar-456-baz-789",
      "foo-123-bar-456-baz-789",
      "foo-123-bar-456-baz-789",
      "foo-123-bar-456-baz-790",
      "foo-123-bar-457-baz-789",
      "foo-124-bar-456-baz-788",
      "foo-125-bar-456-baz-790",
      "foo-126-bar-455-baz-789",
      "foo-127-bar-456-baz-789",
      "foo-128-bar-457-baz-789",
    ]
  );
}

#[test]
fn sort_is_stable_for_mixed_numeric_and_keyword() {
  // Heap's algorithm permutations.
  fn permutations<T: Clone>(items: Vec<T>) -> Vec<Vec<T>> {
    let mut input = items;
    let n = input.len();
    let mut out = vec![input.clone()];
    let mut stack = vec![0usize; n];
    let mut pos = 1usize;
    while pos < n {
      if stack[pos] < pos {
        let k = if pos % 2 == 0 { 0 } else { stack[pos] };
        input.swap(k, pos);
        out.push(input.clone());
        stack[pos] += 1;
        pos = 1;
      } else {
        stack[pos] = 0;
        pos += 1;
      }
    }
    out
  }

  let classes = vec![
    "duration-initial",
    "duration-75",
    "duration-150",
    "duration-700",
    "duration-1000",
  ];
  for p in permutations(classes) {
    let mut sorted: Vec<&str> = p.clone();
    sorted.sort_by(|a, b| compare(a, b));
    assert_eq!(
      sorted,
      vec![
        "duration-75",
        "duration-150",
        "duration-700",
        "duration-1000",
        "duration-initial",
      ]
    );
  }
}
