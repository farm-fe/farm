use farmfe_docs_config::{parse_config, schema};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site")
}
fn input() -> Value {
  json!({"title":"Syndication","url":"https://example.com","pages":{"dir":"docs"},
    "seo":{"atom":{"updated":"2026-10-18T00:00:00Z","author":"Docs contributors"}},
    "search":{"provider":"algolia","options":{"appId":"APP","apiKey":"public","indexName":"docs"},
      "openSearch":{"pageId":"search","shortName":"Docs 中文","description":"Search & discover"}}})
}

#[test]
fn explicit_syndication_configuration_is_public_and_optional() {
  let config =
    parse_config(&input().to_string(), &root()).expect("explicit Rust syndication config");
  let public = serde_json::to_value(config.public).unwrap();
  assert_eq!(public["seo"]["atom"], input()["seo"]["atom"]);
  assert_eq!(
    public["search"]["openSearch"],
    input()["search"]["openSearch"]
  );
  let mut omitted = input();
  omitted.as_object_mut().unwrap().remove("seo");
  omitted["search"]
    .as_object_mut()
    .unwrap()
    .remove("openSearch");
  let config = parse_config(&omitted.to_string(), &root()).unwrap();
  let public = serde_json::to_value(config.public).unwrap();
  assert!(public["seo"].get("atom").is_none());
  assert!(public["search"].get("openSearch").is_none());
  let schema = serde_json::to_value(schema()).unwrap();
  assert_eq!(
    schema["definitions"]["SeoConfig"]["properties"]["atom"]["$ref"],
    "#/definitions/AtomConfig"
  );
  assert_eq!(
    schema["definitions"]["SearchConfig"]["properties"]["openSearch"]["$ref"],
    "#/definitions/OpenSearchConfig"
  );
}

#[test]
fn syndication_semantics_reject_invalid_dates_text_provider_and_page_ids() {
  for (pointer, value, field) in [
    ("/seo/atom/updated", json!("2026-10-18"), "seo.atom.updated"),
    (
      "/seo/atom/updated",
      json!("2026-02-30T00:00:00Z"),
      "seo.atom.updated",
    ),
    (
      "/seo/atom/updated",
      json!("0000-01-01T00:00:00Z"),
      "seo.atom.updated",
    ),
    ("/seo/atom/author", json!("  "), "seo.atom.author"),
    ("/seo/atom/author", json!("author\u{1}"), "seo.atom.author"),
    (
      "/search/openSearch/shortName",
      json!("abcdefghijklmnopq"),
      "search.openSearch.shortName",
    ),
    (
      "/search/openSearch/description",
      json!("a".repeat(1025)),
      "search.openSearch.description",
    ),
    (
      "/search/openSearch/pageId",
      json!("/search/"),
      "search.openSearch.pageId",
    ),
    (
      "/search/openSearch/pageId",
      json!("../search"),
      "search.openSearch.pageId",
    ),
    (
      "/search/openSearch/pageId",
      json!("search?q=bad"),
      "search.openSearch.pageId",
    ),
  ] {
    let mut raw = input();
    *raw.pointer_mut(pointer).unwrap() = value;
    let error = parse_config(&raw.to_string(), &root())
      .err()
      .expect("invalid syndication config");
    assert_eq!(error.code, "DOCS_CONFIG_SYNDICATION", "{pointer}: {error}");
    assert_eq!(error.field, field);
  }
  let mut raw = input();
  raw["search"]["provider"] = json!("none");
  raw["search"].as_object_mut().unwrap().remove("options");
  let error = parse_config(&raw.to_string(), &root()).err().unwrap();
  assert_eq!(error.field, "search.openSearch");
}

#[test]
fn syndication_objects_reject_null_sequences_unknown_and_duplicate_fields() {
  for pointer in ["/seo/atom", "/search/openSearch"] {
    for value in [Value::Null, json!([]), json!({"unknown":true})] {
      let mut raw = input();
      *raw.pointer_mut(pointer).unwrap() = value;
      let error = parse_config(&raw.to_string(), &root())
        .err()
        .expect("strict object");
      assert_eq!(error.code, "DOCS_CONFIG_PARSE");
    }
  }
  for (needle, replacement) in [
    (
      r#""author":"Docs contributors""#,
      r#""author":"Docs contributors","author":"other""#,
    ),
    (
      r#""pageId":"search""#,
      r#""pageId":"search","pageId":"other""#,
    ),
  ] {
    let raw = input().to_string();
    assert!(raw.contains(needle));
    assert!(parse_config(&raw.replace(needle, replacement), &root()).is_err());
  }
}
