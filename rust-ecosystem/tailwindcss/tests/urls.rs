mod support;

#[allow(dead_code)]
mod normalize_path {
  include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/normalize_path.rs"));
}

#[allow(dead_code)]
mod urls {
  include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/urls.rs"));

  #[cfg(test)]
  mod moved_tests {
    use super::*;
    use farmfe_testing_helpers::assert_snapshot;

    #[test]
    fn helper_outputs_are_snapshotted() {
      let cases = [
        ("join basic", join_posix("a/b", "c/d")),
        ("relative basic", make_relative("a/b", "a/c/d")),
        ("rebase basic", rebase_url("./img.png", "src/styles", "src")),
        ("rebase parent", rebase_url("../img.png", "src/styles/components", "src")),
        ("rebase plain", rebase_url("img.png", "src/styles", "src")),
      ];

      let output = cases
        .into_iter()
        .map(|(name, value)| format!("{name}: {value}"))
        .collect::<Vec<_>>()
        .join("\n");

      assert_snapshot!(output);
    }

    #[test]
    fn rewrite_urls_matrix() {
      let cases = [
        ("relative", ".foo { background: url(./image.png); }", "src/styles", "src"),
        (
          "parent relative",
          ".foo { background: url(../image.png); }",
          "src/styles/components",
          "src",
        ),
        ("absolute", ".foo { background: url(/image.png); }", "src/styles", "src"),
        (
          "external",
          ".foo { background: url(https://example.com/image.png); }",
          "src/styles",
          "src",
        ),
        (
          "data uri",
          ".foo { background: url(data:image/png;base64,abc); }",
          "src/styles",
          "src",
        ),
        ("fragment", ".foo { background: url(#pattern); }", "src/styles", "src"),
        (
          "function call",
          ".foo { background: url(var(--image)); }",
          "src/styles",
          "src",
        ),
        (
          "image set",
          ".foo { background-image: image-set('./image.png' 1x, './image@2x.png' 2x); }",
          "src/styles",
          "src",
        ),
        ("no urls", ".foo { color: red; }", "src/styles", "src"),
      ];

      let output = cases
        .into_iter()
        .map(|(name, css, base, root)| {
          format!("{name}:\n{}", rewrite_urls(css, base, root))
        })
        .collect::<Vec<_>>()
        .join("\n\n");

      assert_snapshot!(output);
    }
  }
}
