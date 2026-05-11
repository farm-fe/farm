mod support;

#[allow(dead_code)]
mod env {
  include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/generated/env.rs"
  ));

  #[cfg(test)]
  mod moved_tests {
    use super::*;
    use farmfe_testing_helpers::assert_snapshot;

    #[test]
    fn resolve_debug_matrix() {
      let cases = [
        ("none", None, false),
        ("true", Some("true"), true),
        ("one", Some("1"), true),
        ("false", Some("false"), false),
        ("zero", Some("0"), false),
        ("wildcard", Some("*"), true),
        (
          "tailwindcss list",
          Some("projectA,tailwindcss,projectB"),
          true,
        ),
        (
          "tailwindcss namespace",
          Some("other,tailwindcss:submodule"),
          true,
        ),
        ("exclude", Some("projectA,-tailwindcss"), false),
        (
          "exclusion precedence",
          Some("tailwindcss,-tailwindcss"),
          false,
        ),
        ("unrelated", Some("projectA,projectB"), false),
        ("empty", Some(""), false),
      ];

      let output = cases
        .into_iter()
        .map(|(name, value, expected)| {
          format!(
            "{name}: input={value:?}, expected={expected}, actual={}",
            resolve_debug(value)
          )
        })
        .collect::<Vec<_>>()
        .join("\n");

      assert_snapshot!(output);
    }
  }
}
