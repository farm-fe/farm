mod support;

#[allow(dead_code)]
mod source_maps {
  include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/generated/source_maps.rs"
  ));

  #[cfg(test)]
  mod moved_tests {
    use super::*;
    use farmfe_testing_helpers::assert_snapshot;

    #[test]
    fn source_map_outputs_are_stable() {
      let raw = r#"{"version":3,"sources":[],"names":[],"mappings":""}"#.to_string();
      let source_map = SourceMap::new(raw.clone());

      let output = format!(
        "raw: {}\ncomment: {}inline: {}",
        source_map.raw(),
        source_map.comment("app.css.map"),
        source_map.inline()
      );

      assert_snapshot!(output);
      assert_eq!(source_map.raw(), raw);
    }
  }
}
