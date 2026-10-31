use farmfe_syntax_highlight::{CodeThemeConfig, HighlightOptions, highlight};
#[test]
fn strict_palette_rejects_unsafe_colors_and_unknown_categories() {
  for json in [
    r##"[]"##,
    r##"{"tokens":[]}"##,
    r##"{"tokens":{"light":[]}}"##,
    r##"{"light":"unknown"}"##,
    r##"{"tokens":{"dark":{"typo":"#123456"}}}"##,
    r##"{"tokens":{"dark":{"keyword":"url(evil)"}}}"##,
    r##"{"tokens":{"dark":{"keyword":"#fff"}}}"##,
  ] {
    assert!(
      serde_json::from_str::<CodeThemeConfig>(json).is_err(),
      "{json}"
    );
  }
}
#[test]
fn farm_palette_keeps_original_text_and_custom_scopes_are_isolated() {
  let source = r#"interface User { name: string }
class Service<T> { async load(id: number): Promise<void> { console.log("hello", id); } }
const view = <Button disabled={false}>Hello</Button>;
"#;
  let options: HighlightOptions =
    serde_json::from_value(serde_json::json!({"light":"farm-light","dark":"farm-dark"})).unwrap();
  let out = highlight(source, "tsx", &options).unwrap();
  assert_eq!(
    out
      .lines
      .iter()
      .flatten()
      .map(|v| v.text.as_str())
      .collect::<String>(),
    source
  );
  let custom: HighlightOptions = serde_json::from_value(serde_json::json!({"light":"farm-light","dark":"farm-dark","tokens":{"light":{"type":"#AABBCC"},"dark":{"type":"#DDEEFF"}}})).unwrap();
  let altered = highlight(source, "tsx", &custom).unwrap();
  fn token<'a>(
    rows: &'a farmfe_syntax_highlight::Highlighted,
    name: &str,
  ) -> &'a farmfe_syntax_highlight::Token {
    rows
      .lines
      .iter()
      .flatten()
      .find(|t| t.text == name)
      .unwrap()
  }
  assert_eq!(token(&altered, "Promise").light, "#aabbcc");
  assert_eq!(token(&altered, "Promise").dark, "#ddeeff");
  assert_ne!(
    token(&out, "Promise").light,
    token(&altered, "Promise").light
  );
  assert_eq!(
    highlight(source, "tsx", &options).unwrap().lines,
    out.lines,
    "custom colors must not mutate shared preset state"
  );
}
