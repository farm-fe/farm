use farmfe_docs_config::{parse_config, public_schema, schema};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site")
}

fn input() -> Value {
  json!({"title":"SEO","url":"https://example.com","pages":{"dir":"docs"}})
}

#[test]
fn seo_fallback_policy_defaults_to_noindex_and_is_explicitly_public() {
  let config = parse_config(&input().to_string(), &root()).expect("minimal configuration");
  let public = serde_json::to_value(config.public).unwrap();
  assert_eq!(public["seo"]["fallbackIndexing"], "noindex");
  for policy in ["noindex", "index"] {
    let mut raw = input();
    raw["seo"] = json!({"fallbackIndexing":policy});
    let config = parse_config(&raw.to_string(), &root()).expect("explicit SEO policy");
    assert_eq!(
      serde_json::to_value(config.public).unwrap()["seo"]["fallbackIndexing"],
      policy
    );
  }
}

#[test]
fn seo_policy_rejects_unknown_values_fields_and_wrong_shapes() {
  for seo in [
    json!({"fallbackIndexing":"all"}),
    json!({"fallbackIndexing":true}),
    json!({"unknown":1}),
    Value::Null,
    json!([]),
  ] {
    let mut raw = input();
    raw["seo"] = seo;
    let error = parse_config(&raw.to_string(), &root())
      .err()
      .expect("invalid SEO configuration");
    assert_eq!(error.code, "DOCS_CONFIG_PARSE");
  }
}

#[test]
fn seo_input_is_optional_but_normalized_public_contract_requires_it() {
  let input = serde_json::to_value(schema()).unwrap();
  let public = serde_json::to_value(public_schema()).unwrap();
  assert!(input["properties"].get("seo").is_some());
  assert!(
    !input["required"]
      .as_array()
      .unwrap()
      .contains(&json!("seo"))
  );
  assert!(
    public["required"]
      .as_array()
      .unwrap()
      .contains(&json!("seo"))
  );
}

#[test]
fn seo_rejects_duplicate_policy_fields() {
  let raw = r#"{"title":"SEO","url":"https://example.com","pages":{"dir":"docs"},"seo":{"fallbackIndexing":"index","fallbackIndexing":"noindex"}}"#;
  assert!(
    parse_config(raw, &root()).is_err(),
    "duplicate fields must not silently replace policy"
  );
}
