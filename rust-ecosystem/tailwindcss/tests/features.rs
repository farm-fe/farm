use farmfe_ecosystem_tailwindcss::{Features, Polyfills};

// ── Features ─────────────────────────────────────────────────────────────────

#[test]
fn features_none_has_no_bits_set() {
  assert!(!Features::NONE.contains(Features::AT_APPLY));
  assert!(!Features::NONE.contains(Features::JS_PLUGIN_COMPAT));
  assert!(!Features::NONE.contains(Features::THEME_FUNCTION));
  assert!(!Features::NONE.contains(Features::UTILITIES));
  assert!(!Features::NONE.has_any_output_feature());
}

#[test]
fn features_named_constants_are_distinct() {
  let all = [
    Features::AT_APPLY,
    Features::JS_PLUGIN_COMPAT,
    Features::THEME_FUNCTION,
    Features::UTILITIES,
  ];
  for (i, a) in all.iter().enumerate() {
    for (j, b) in all.iter().enumerate() {
      if i != j {
        // Two distinct flags must not overlap.
        assert!(
          !a.contains(*b),
          "flag at index {i} should not contain flag at index {j}"
        );
      }
    }
  }
}

#[test]
fn features_bitor_combines_flags() {
  let combined = Features::AT_APPLY | Features::UTILITIES;
  assert!(combined.contains(Features::AT_APPLY));
  assert!(combined.contains(Features::UTILITIES));
  assert!(!combined.contains(Features::THEME_FUNCTION));
  assert!(!combined.contains(Features::JS_PLUGIN_COMPAT));
}

#[test]
fn features_bitorassign_accumulates_flags() {
  let mut f = Features::NONE;
  f |= Features::THEME_FUNCTION;
  f |= Features::AT_APPLY;
  assert!(f.contains(Features::THEME_FUNCTION));
  assert!(f.contains(Features::AT_APPLY));
  assert!(!f.contains(Features::UTILITIES));
}

#[test]
fn features_bitand_intersects_flags() {
  let a = Features::AT_APPLY | Features::UTILITIES;
  let b = Features::UTILITIES | Features::THEME_FUNCTION;
  let intersection = a & b;
  assert!(intersection.contains(Features::UTILITIES));
  assert!(!intersection.contains(Features::AT_APPLY));
  assert!(!intersection.contains(Features::THEME_FUNCTION));
}

#[test]
fn features_has_any_output_feature_reflects_output_flags() {
  assert!(Features::AT_APPLY.has_any_output_feature());
  assert!(Features::JS_PLUGIN_COMPAT.has_any_output_feature());
  assert!(Features::THEME_FUNCTION.has_any_output_feature());
  assert!(Features::UTILITIES.has_any_output_feature());
  // Combination also returns true.
  assert!((Features::AT_APPLY | Features::UTILITIES).has_any_output_feature());
}

// ── Polyfills ─────────────────────────────────────────────────────────────────

#[test]
fn polyfills_none_has_no_bits_set() {
  assert!(!Polyfills::NONE.contains(Polyfills::AT_MEDIA_HOVER));
}

#[test]
fn polyfills_bitor_combines_flags() {
  let combined = Polyfills::NONE | Polyfills::AT_MEDIA_HOVER;
  assert!(combined.contains(Polyfills::AT_MEDIA_HOVER));
}

#[test]
fn polyfills_bitorassign_accumulates_flags() {
  let mut p = Polyfills::NONE;
  p |= Polyfills::AT_MEDIA_HOVER;
  assert!(p.contains(Polyfills::AT_MEDIA_HOVER));
}
