mod support;

#[allow(dead_code)]
mod normalize_path {
  include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/normalize_path.rs"));

  #[cfg(test)]
  mod moved_tests {
    use super::*;
    use farmfe_testing_helpers::assert_snapshot;

    #[test]
    fn normalize_path_matrix() {
      let cases = [
        ("forward", "foo/bar/baz"),
        ("backslashes", "foo\\bar\\baz"),
        ("mixed", "foo/bar\\baz"),
        ("trailing", "foo/bar/"),
        ("root-slash", "/"),
        ("root-backslash", "\\"),
        ("single", "a"),
        ("empty", ""),
        ("unc", "\\\\server\\share"),
        ("namespace-question", "\\\\?\\C:\\foo\\bar"),
        ("namespace-dot", "\\\\.\\C:\\foo\\bar"),
        ("consecutive", "foo//bar///baz"),
      ];

      let output = cases
        .into_iter()
        .map(|(name, input)| format!("{name}: {input:?} -> {:?}", normalize_path(input)))
        .collect::<Vec<_>>()
        .join("\n");

      assert_snapshot!(output);
    }
  }
}
