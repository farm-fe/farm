//! Smoke tests for the breakpoint comparator. Upstream does not ship a
//! dedicated `compare-breakpoints.test.ts`; these cases mirror the
//! examples in the source-level documentation comments.

use farmfe_ecosystem_tailwindcss::utils::compare_breakpoints::{
  compare_breakpoints, Direction,
};
use std::cmp::Ordering;

fn sign(o: Ordering) -> i32 {
  match o {
    Ordering::Less => -1,
    Ordering::Equal => 0,
    Ordering::Greater => 1,
  }
}

#[test]
fn equal_inputs_compare_equal() {
  assert_eq!(
    sign(compare_breakpoints("40rem", "40rem", Direction::Asc)),
    0
  );
}

#[test]
fn ascending_compares_numeric_within_unit_bucket() {
  assert_eq!(
    sign(compare_breakpoints("40rem", "60rem", Direction::Asc)),
    -1
  );
  assert_eq!(
    sign(compare_breakpoints("60rem", "40rem", Direction::Asc)),
    1
  );
}

#[test]
fn descending_reverses_numeric_order() {
  assert_eq!(
    sign(compare_breakpoints("40rem", "60rem", Direction::Desc)),
    1
  );
  assert_eq!(
    sign(compare_breakpoints("60rem", "40rem", Direction::Desc)),
    -1
  );
}

#[test]
fn different_units_bucket_separately() {
  // Buckets `rem` vs `px` compare lexicographically: "px" < "rem".
  assert_eq!(
    sign(compare_breakpoints("100px", "1rem", Direction::Asc)),
    -1
  );
}

#[test]
fn css_function_buckets_use_function_name() {
  // Both `calc(...)`, same bucket, no parseable leading int → string fallback.
  assert_eq!(
    sign(compare_breakpoints(
      "calc(100% - 1rem)",
      "calc(100% - 2rem)",
      Direction::Asc,
    )),
    -1
  );
  assert_eq!(
    sign(compare_breakpoints(
      "calc(100% - 2rem)",
      "calc(100% - 1rem)",
      Direction::Asc,
    )),
    1
  );
}
