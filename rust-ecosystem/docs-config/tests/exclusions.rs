use farmfe_docs_config::parse_config;
use serde_json::json;
use std::path::Path;

#[test]
fn exact_exclusions_validate_files_without_leaking_private_paths() {
  let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site");
  let raw = json!({"title":"Docs","url":"https://example.com","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/","exclude":["index.md"]}]}});
  let config = parse_config(&raw.to_string(), &root).unwrap();
  assert!(
    !serde_json::to_string(&config.public)
      .unwrap()
      .contains("index.md")
  );
  for exclusions in [
    json!(["index.md", "index.md"]),
    json!(["../retired.md"]),
    json!(["/retired.md"]),
    json!(["C:/retired.md"]),
    json!(["a\\retired.md"]),
    json!(["./retired.md"]),
    json!(["a//retired.md"]),
    json!(["*.md"]),
    json!([""]),
    json!(["missing.md"]),
    json!(["."]),
  ] {
    let mut invalid = raw.clone();
    invalid["docs"]["collections"][0]["exclude"] = exclusions;
    let error = parse_config(&invalid.to_string(), &root)
      .err()
      .expect("invalid exclusion");
    assert_eq!(error.code, "DOCS_CONFIG_COLLECTION_EXCLUDE", "{error}");
    assert!(error.field.contains("exclude["), "{error}");
  }
}
