use farmfe_docs_config::parse_config;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site")
}
fn input() -> Value {
  json!({"title":"Localized","url":"https://example.com",
    "locales":{"default":"en","items":["en","zh"]},"pages":{"dir":"docs"}})
}

#[test]
fn not_found_labels_resolve_defaults_for_every_declared_locale() {
  let config = parse_config(&input().to_string(), &root()).unwrap();
  let public = serde_json::to_value(config.public).unwrap();
  for locale in ["en", "zh"] {
    assert_eq!(
      public["notFound"]["labels"][locale]["title"],
      "Page not found"
    );
    assert_eq!(
      public["notFound"]["labels"][locale]["homeLabel"],
      "Return home"
    );
  }
}

#[test]
fn not_found_labels_are_explicit_public_text_not_language_specific_code() {
  let mut raw = input();
  raw["notFound"] = json!({"labels":{"zh":{"title":"找不到页面","description":"页面不存在。","homeLabel":"返回首页"}}});
  let config = parse_config(&raw.to_string(), &root()).unwrap();
  let public = serde_json::to_value(config.public).unwrap();
  assert_eq!(public["notFound"]["labels"]["zh"]["title"], "找不到页面");
  assert_eq!(
    public["notFound"]["labels"]["zh"]["description"],
    "页面不存在。"
  );
  assert_eq!(public["notFound"]["labels"]["zh"]["homeLabel"], "返回首页");
  assert_eq!(
    public["notFound"]["labels"]["en"]["title"],
    "Page not found"
  );
}

#[test]
fn not_found_labels_reject_undeclared_locale_and_blank_text_with_field_diagnostics() {
  let mut unknown = input();
  unknown["notFound"] = json!({"labels":{"fr":{"title":"Absent"}}});
  let error = parse_config(&unknown.to_string(), &root()).err().unwrap();
  assert_eq!(error.code, "DOCS_CONFIG_NOT_FOUND");
  assert_eq!(error.field, "notFound.labels.fr");
  for field in ["title", "description", "homeLabel"] {
    let mut raw = input();
    raw["notFound"] = json!({"labels":{"zh":{}}});
    raw["notFound"]["labels"]["zh"][field] = json!("  ");
    let error = parse_config(&raw.to_string(), &root()).err().unwrap();
    assert_eq!(error.code, "DOCS_CONFIG_NOT_FOUND");
    assert_eq!(error.field, format!("notFound.labels.zh.{field}"));
  }
}

#[test]
fn not_found_configuration_rejects_unknown_fields_and_wrong_shapes() {
  for value in [
    Value::Null,
    json!([]),
    json!({"unknown":true}),
    json!({"labels":{"zh":{"unknown":"x"}}}),
    json!({"labels":{"zh":[]}}),
    json!({"labels":{"zh":null}}),
    json!({"labels":[]}),
  ] {
    let mut raw = input();
    raw["notFound"] = value;
    assert_eq!(
      parse_config(&raw.to_string(), &root()).err().unwrap().code,
      "DOCS_CONFIG_PARSE"
    );
  }
}
