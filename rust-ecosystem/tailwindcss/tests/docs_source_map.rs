use farmfe_ecosystem_tailwindcss::compiler::{CompilerOptions, compile};

#[test]
fn source_map_contains_original_css_after_build() {
  let input = "/* 中文 😀 */\n@tailwind utilities;\n.original { color: red; }\n";
  let mut compiler = compile(
    input,
    CompilerOptions {
      source_maps_enabled: true,
      ..Default::default()
    },
  )
  .unwrap();
  let css = compiler.build(&["flex".into()]);
  assert!(css.contains("display: flex"));
  let map: serde_json::Value = serde_json::from_str(&compiler.build_source_map().unwrap()).unwrap();
  assert_eq!(
    map["sourcesContent"][0], input,
    "original input must survive transformation"
  );
  assert!(
    !map["mappings"].as_str().unwrap().is_empty(),
    "a placeholder is not a source map"
  );
}

fn position(text: &str, needle: &str) -> (u32, u32) {
  let before = &text[..text.find(needle).unwrap()];
  (
    before.bytes().filter(|byte| *byte == b'\n').count() as u32,
    before.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
  )
}

#[test]
fn maps_original_declarations_and_generated_utilities_to_real_origins() {
  let input = "/* 中文 😀 */\n@tailwind utilities;\n.中文😀 { content: '😀'; color: red; }\n.other { color: red; }\n";
  let mut compiler = compile(
    input,
    CompilerOptions {
      source_maps_enabled: true,
      ..Default::default()
    },
  )
  .unwrap();
  let css = compiler.build(&["flex".into()]);
  let map =
    sourcemap::SourceMap::from_slice(compiler.build_source_map().unwrap().as_bytes()).unwrap();
  for (generated, original) in [
    ("display: flex", "@tailwind"),
    ("content:", "content:"),
    ("color: red", "color: red"),
  ] {
    let (line, column) = position(&css, generated);
    let token = map.lookup_token(line, column).unwrap();
    assert_eq!(
      (token.get_src_line(), token.get_src_col()),
      position(input, original),
      "{generated}"
    );
  }
  let (line, column) = position(&css, ".other");
  let token = map.lookup_token(line + 1, column + 2).unwrap();
  assert_eq!((token.get_src_line(), token.get_src_col()), (3, 9));
}

#[test]
fn map_switch_preserves_optimization_theme_and_apply_output() {
  let input = "@theme { --color-brand: red; }\n@tailwind utilities;\n.a { color: red; color: red; @apply flex; }\n@media screen { .a { color: red; } }\n@media screen { .b { color: blue; } }";
  let candidates = vec!["bg-brand".into(), "md:flex".into()];
  let mut plain = compile(input, CompilerOptions::default()).unwrap();
  let mut mapped = compile(
    input,
    CompilerOptions {
      source_maps_enabled: true,
      ..Default::default()
    },
  )
  .unwrap();
  assert_eq!(mapped.build(&candidates), plain.build(&candidates));
  assert!(plain.build_source_map().is_none());
  let first = mapped.build_source_map().unwrap();
  mapped.build(&["hidden".into()]);
  assert_ne!(
    mapped.build_source_map().unwrap(),
    first,
    "map must track the most recent build"
  );
}
