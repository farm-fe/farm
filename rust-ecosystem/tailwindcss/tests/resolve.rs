mod support;

#[allow(dead_code)]
mod resolve {
  include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/resolve.rs"));

  #[cfg(test)]
  mod moved_tests {
    use super::*;
    use crate::support::{fixture_path, manifest_path};
    use farmfe_testing_helpers::assert_snapshot;

    #[test]
    fn resolves_fixture_matrix() {
      let fixture_root = fixture_path("resolve");
      let custom_target = fixture_root.join("custom/custom.css");
      let custom_target_string = custom_target.to_string_lossy().to_string();
      let custom: CustomResolver = Box::new(move |_id, _base| Some(custom_target_string.clone()));

      let cases = vec![
        (
          "relative css",
          resolve_css_id(
            "./styles.css",
            fixture_root.join("rel_css").to_str().unwrap(),
            None,
          )
          .map(|path| manifest_path(&path)),
        ),
        (
          "relative css no ext",
          resolve_css_id(
            "./styles",
            fixture_root.join("no_ext_css").to_str().unwrap(),
            None,
          )
          .map(|path| manifest_path(&path)),
        ),
        (
          "relative js",
          resolve_js_id(
            "./mod.js",
            fixture_root.join("rel_js").to_str().unwrap(),
            None,
          )
          .map(|path| manifest_path(&path)),
        ),
        (
          "relative js no ext",
          resolve_js_id(
            "./mod",
            fixture_root.join("no_ext_js").to_str().unwrap(),
            None,
          )
          .map(|path| manifest_path(&path)),
        ),
        (
          "index js",
          resolve_js_id(
            "./utils",
            fixture_root.join("index_js").to_str().unwrap(),
            None,
          )
          .map(|path| manifest_path(&path)),
        ),
        (
          "custom css",
          resolve_css_id("ignored", fixture_root.join("custom").to_str().unwrap(), Some(&custom))
            .map(|path| manifest_path(&path)),
        ),
      ];

      let output = cases
        .into_iter()
        .map(|(name, result)| match result {
          Ok(path) => format!("{name}: {path}"),
          Err(error) => format!("{name}: ERROR {error}"),
        })
        .collect::<Vec<_>>()
        .join("\n");

      assert_snapshot!(output);
    }

    #[test]
    fn missing_path_returns_not_found() {
      let fixture_root = fixture_path("resolve/not_found");
      let error = resolve_css_id("./nope.css", fixture_root.to_str().unwrap(), None).unwrap_err();
      assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }
  }
}
