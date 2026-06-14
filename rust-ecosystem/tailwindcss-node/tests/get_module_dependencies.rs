mod support;

#[allow(dead_code)]
mod get_module_dependencies {
  include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/generated/get_module_dependencies.rs"
  ));

  #[cfg(test)]
  mod moved_tests {
    use super::*;
    use crate::support::{fixture_path, sorted_manifest_paths};
    use farmfe_testing_helpers::assert_snapshot;

    #[test]
    fn dependency_traces_match_fixtures() {
      let cases = [
        (
          "single",
          fixture_path("get_module_dependencies/single/main.js"),
        ),
        (
          "relative import",
          fixture_path("get_module_dependencies/follows_relative_import/main.js"),
        ),
        (
          "non relative",
          fixture_path("get_module_dependencies/ignores_non_relative/main.js"),
        ),
        (
          "circular",
          fixture_path("get_module_dependencies/handles_circular/a.js"),
        ),
        (
          "no extension",
          fixture_path("get_module_dependencies/resolves_without_extension/main.ts"),
        ),
        (
          "require",
          fixture_path("get_module_dependencies/follows_require/main.js"),
        ),
      ];

      let output = cases
        .into_iter()
        .map(|(name, entry)| {
          let deps = get_module_dependencies(&entry).unwrap();
          format!("{name}:\n  {}", sorted_manifest_paths(deps).join("\n  "))
        })
        .collect::<Vec<_>>()
        .join("\n\n");

      assert_snapshot!(output);
    }
  }
}
