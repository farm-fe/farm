use farmfe_ecosystem_tailwindcss_node::compile::{CompileOptions, compile};

#[test]
fn imported_stylesheet_keeps_its_own_source_and_original_content() {
  let entry = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures/docs-source-map/entry.css");
  let input = std::fs::read_to_string(&entry).unwrap();
  let mut compiler = compile(
    &input,
    CompileOptions {
      base: entry.parent().unwrap().to_string_lossy().into_owned(),
      from: Some(entry.to_string_lossy().into_owned()),
      should_rewrite_urls: true,
      ..Default::default()
    },
  )
  .unwrap();
  let css = compiler.build(&["flex".into()]);
  assert!(
    css.contains(".imported"),
    "fixture import must actually be resolved: {css}"
  );
  assert!(
    css.contains("./nested/pixel.svg"),
    "URL must actually be rebased: {css}"
  );
  let map = compiler.build_source_map().unwrap();
  assert!(
    map.contains("piece.css"),
    "imported file lost its identity: {map}"
  );
  assert!(
    map.contains("entry.css"),
    "root file lost its identity: {map}"
  );
  assert!(
    map.contains("./pixel.svg"),
    "map must contain original URL before rewriting: {map}"
  );
  let map = sourcemap::SourceMap::from_slice(map.as_bytes()).unwrap();
  let imported_path = entry.parent().unwrap().join("nested/piece.css");
  let imported = std::fs::read_to_string(&imported_path).unwrap();
  for (generated, source, filename, needle) in [
    ("background:", imported.as_str(), "piece.css", "background:"),
    ("color: red", imported.as_str(), "piece.css", "color: red"),
    ("color: blue", input.as_str(), "entry.css", "color: blue"),
    ("display: flex", input.as_str(), "entry.css", "@tailwind"),
  ] {
    let (line, column) = position(&css, generated);
    let token = map.lookup_token(line, column).unwrap();
    assert!(token.get_source().unwrap().ends_with(filename));
    assert_eq!(
      (token.get_src_line(), token.get_src_col()),
      position(source, needle),
      "{generated}"
    );
    assert_eq!(map.get_source_contents(token.get_src_id()), Some(source));
  }
}

fn position(text: &str, needle: &str) -> (u32, u32) {
  let prefix = &text[..text.find(needle).unwrap()];
  (
    prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
    prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
  )
}
