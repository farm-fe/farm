use farmfe_docs_config::{
  BuildConfig, HighlightRepresentation, parse_config, public_schema, schema,
};
use serde_json::{Value, json};
use std::path::Path;

#[test]
fn build_defaults_and_explicit_values_are_private() {
  let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site");
  let mut raw = json!({"title":"Build","url":"https://example.com","pages":{"dir":"docs"}});
  let config = parse_config(&raw.to_string(), &root).unwrap();
  assert_eq!(
    config.build.highlight_representation,
    HighlightRepresentation::Jsx
  );
  assert_eq!(config.build.target_concurrency, 1);
  assert!(
    serde_json::to_value(config.public)
      .unwrap()
      .get("build")
      .is_none()
  );
  for concurrency in [1, 2] {
    raw["build"] = json!({"highlightRepresentation":"static-html","targetConcurrency":concurrency});
    let config = parse_config(&raw.to_string(), &root).unwrap();
    assert_eq!(
      config.build.highlight_representation,
      HighlightRepresentation::StaticHtml
    );
    assert_eq!(config.build.target_concurrency, concurrency);
  }
  assert!(
    serde_json::to_value(schema()).unwrap()["properties"]
      .get("build")
      .is_some()
  );
  assert!(
    serde_json::to_value(public_schema()).unwrap()["properties"]
      .get("build")
      .is_none()
  );
}

#[test]
fn build_rejects_invalid_values_and_shapes() {
  for raw in [
    json!({"highlightRepresentation":"html"}),
    json!({"highlightRepresentation":false}),
    json!({"targetConcurrency":0}),
    json!({"targetConcurrency":3}),
    json!({"targetConcurrency":1.5}),
    json!({"targetConcurrency":"2"}),
    json!({"targetConcurrency":null}),
    json!({"unknown":true}),
    Value::Null,
    json!([]),
    json!(["jsx", 1]),
  ] {
    assert!(
      serde_json::from_value::<BuildConfig>(raw.clone()).is_err(),
      "accepted {raw}"
    );
  }
}
