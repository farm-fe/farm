mod support;

#[allow(dead_code)]
mod optimize {
  include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/optimize.rs"));

  #[cfg(test)]
  mod moved_tests {
    use super::*;
    use crate::support::fixture_path;
    use farmfe_testing_helpers::assert_snapshot;
    use std::fs;

    #[test]
    fn optimize_fixture_outputs_are_snapshotted() {
      let cases = [
        (
          "plain",
          fixture_path("optimize/plain.css"),
          OptimizeOptions::default(),
        ),
        (
          "nesting",
          fixture_path("optimize/nesting.css"),
          OptimizeOptions::default(),
        ),
        (
          "media-not",
          fixture_path("optimize/media-not.css"),
          OptimizeOptions::default(),
        ),
        (
          "minify",
          fixture_path("optimize/minify.css"),
          OptimizeOptions {
            minify: true,
            ..Default::default()
          },
        ),
      ];

      let output = cases
        .into_iter()
        .map(|(name, path, options)| {
          let input = fs::read_to_string(&path).unwrap();
          let result = optimize(&input, options).unwrap();
          format!(
            "{name}:\n-- input --\n{input}\n-- output --\n{}",
            result.code
          )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

      assert_snapshot!(output);
    }

    #[test]
    fn optimize_metadata_and_identity_cases() {
      assert!(optimize("", OptimizeOptions::default()).unwrap().code.trim().is_empty());
      assert_eq!(OptimizeOptions::default().file, "input.css");
      assert!(!OptimizeOptions::default().minify);
      assert_eq!(
        OptimizeOptions {
          file: "styles.css".to_string(),
          ..Default::default()
        }
        .file,
        "styles.css"
      );

      let input = ":root { --color: red; }\n.foo { color: var(--color); }";
      let first = optimize(input, OptimizeOptions::default()).unwrap();
      let second = optimize(input, OptimizeOptions::default()).unwrap();
      assert_eq!(first, second);
      assert!(first.code.contains("--color"));
      assert!(first.code.contains("var(--color)"));

      let keyframes = optimize(
        "@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }\n.foo { animation: spin 1s linear; }",
        OptimizeOptions::default(),
      )
      .unwrap();
      assert!(keyframes.code.contains("@keyframes"));
      assert!(keyframes.code.contains("spin"));
    }
  }
}
