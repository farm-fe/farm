mod support;

#[allow(dead_code)]
mod normalize_path {
  include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/generated/normalize_path.rs"
  ));
}

#[allow(dead_code)]
mod urls {
  include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/generated/urls.rs"
  ));

  #[cfg(test)]
  mod moved_tests {
    use super::*;
    use crate::support::fixture_path;
    use farmfe_testing_helpers::assert_snapshot;
    use std::fs;

    #[test]
    fn helper_outputs_are_snapshotted() {
      let cases = [
        ("join basic", join_posix("a/b", "c/d")),
        ("relative basic", make_relative("a/b", "a/c/d")),
        ("rebase basic", rebase_url("./img.png", "src/styles", "src")),
        (
          "rebase parent",
          rebase_url("../img.png", "src/styles/components", "src"),
        ),
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
        (
          "relative",
          ".foo { background: url(./image.png); }",
          "src/styles",
          "src",
        ),
        (
          "parent relative",
          ".foo { background: url(../image.png); }",
          "src/styles/components",
          "src",
        ),
        (
          "absolute",
          ".foo { background: url(/image.png); }",
          "src/styles",
          "src",
        ),
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
        (
          "fragment",
          ".foo { background: url(#pattern); }",
          "src/styles",
          "src",
        ),
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
        .map(|(name, css, base, root)| format!("{name}:\n{}", rewrite_urls(css, base, root)))
        .collect::<Vec<_>>()
        .join("\n\n");

      assert_snapshot!(output);
    }

    #[test]
    fn upstream_tailwind_urls_case() {
      let input = fs::read_to_string(fixture_path("upstream/urls/input.css")).unwrap();
      let output = rewrite_urls(&input, "/root/foo/bar", "/root");

      assert!(output.contains("url(./foo/bar/image.jpg)"));
      assert!(output.contains("url(./foo/image.jpg)"));
      assert!(output.contains("url(/image.jpg)"));
      assert!(output.contains("url(~/image.jpg)"));
      assert!(output.contains("url(#/image.jpg)"));
      assert!(output.contains("url(@/image.jpg)"));
      assert!(output.contains("url(http://example.com/image.jpg)"));
      assert!(output.contains("url('data:image/png;base64,abc==')"));
      assert!(output.contains("url(var(--foo))"));
      assert!(output.contains("url(#dont-touch-this)"));
      assert!(output.contains("url('./foo/bar/image1.jpg')"));
      assert!(output.contains("url(\"./foo/bar/image2.jpg\")"));
      assert!(output.contains("url(\"./foo/bar/newman-outline.woff\")"));

      assert_snapshot!(output);
    }
  }
}
