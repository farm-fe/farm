use farmfe_docs_config::{PublicSiteConfig, parse_config, schema};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site")
}

#[test]
fn admonition_labels_are_strict_private_collection_compilation_inputs() {
  let mut raw = input();
  raw["docs"]["collections"][0]["admonitionLabels"] = json!({"note":"备注", "warning":""});
  let config = parse_config(&raw.to_string(), &root()).unwrap();
  let labels = &config.collections[0].admonition_labels;
  assert_eq!(labels["note"], "备注");
  assert_eq!(labels["warning"], "");
  assert_eq!(labels["tip"], "tip");
  assert!(
    !serde_json::to_string(&config.public)
      .unwrap()
      .contains("admonitionLabels")
  );
  for invalid in [
    json!({"unsupported":"Wrong"}),
    json!({"note":42}),
    json!({"note":null}),
    json!([]),
  ] {
    raw["docs"]["collections"][0]["admonitionLabels"] = invalid;
    let error = parse_config(&raw.to_string(), &root())
      .err()
      .expect("invalid label type/key");
    assert_eq!(error.code, "DOCS_CONFIG_PARSE");
  }
  let error = serde_json::from_str::<farmfe_docs_config::AdmonitionLabels>(
    r#"{"note":"first","note":"second"}"#,
  )
  .unwrap_err();
  assert!(error.to_string().contains("duplicate admonition label"));
}
fn input() -> Value {
  json!({ "title": "Farm", "url": "https://farmfe.org", "docs": { "collections": [
    { "id": "main", "dir": "docs", "routeBase": "/docs" }
  ] } })
}
fn reject(value: Value, code: &str, field: &str) {
  let error = parse_config(&value.to_string(), &root())
    .err()
    .expect("expected invalid configuration");
  assert_eq!(error.code, code, "{error}");
  assert!(error.to_string().contains(field), "{error}");
}

#[test]
fn ssg_limits_are_validated_without_becoming_public_browser_config() {
  let mut raw = input();
  raw["ssg"] = json!({"concurrency":2,"timeoutMs":1000});
  let config = parse_config(&raw.to_string(), &root()).expect("valid bounded SSG settings");
  assert!(
    serde_json::to_value(&config.public)
      .unwrap()
      .get("ssg")
      .is_none()
  );
  for (field, value) in [
    ("concurrency", 0),
    ("concurrency", 33),
    ("timeoutMs", 99),
    ("timeoutMs", 300001),
  ] {
    let mut invalid = raw.clone();
    invalid["ssg"][field] = json!(value);
    reject(invalid, "DOCS_CONFIG_SSG", &format!("ssg.{field}"));
  }
  for invalid in [
    json!({"concurrency":1.5}),
    json!({"timeoutMs":"1000"}),
    json!({"unknown":1}),
    json!(null),
  ] {
    raw["ssg"] = invalid;
    reject(raw.clone(), "DOCS_CONFIG_PARSE", "config");
  }
}

#[test]
fn ssg_partial_defaults_and_inclusive_boundaries_are_deterministic() {
  for (settings, concurrency, timeout_ms) in [
    (json!({}), 4, 30_000),
    (json!({"concurrency":1}), 1, 30_000),
    (json!({"timeoutMs":100}), 4, 100),
    (json!({"concurrency":32,"timeoutMs":300000}), 32, 300_000),
  ] {
    let mut raw = input();
    raw["ssg"] = settings;
    let config = parse_config(&raw.to_string(), &root()).unwrap();
    assert_eq!(config.ssg.concurrency, concurrency);
    assert_eq!(config.ssg.timeout_ms, timeout_ms);
  }
}

#[test]
fn search_options_match_provider_and_public_credentials_are_preserved() {
  let mut raw = input();
  raw["search"] = json!({"provider":"algolia","options":{"appId":"app","apiKey":"public-search-key","indexName":"docs"}});
  let config = parse_config(&raw.to_string(), &root()).unwrap();
  let public = serde_json::to_value(&config.public).unwrap();
  assert_eq!(public["search"], raw["search"]);
  raw["search"]["options"]["apiKey"] = json!("");
  reject(raw.clone(), "DOCS_CONFIG_VALUE", "apiKey");
  raw["search"] = json!({"provider":"algolia"});
  reject(raw.clone(), "DOCS_CONFIG_SEARCH", "search.options");
  raw["search"] =
    json!({"provider":"none","options":{"appId":"app","apiKey":"key","indexName":"docs"}});
  reject(raw.clone(), "DOCS_CONFIG_SEARCH", "search.options");
  raw["search"] = json!({"provider":"none"});
  assert!(parse_config(&raw.to_string(), &root()).is_ok());
}

#[test]
fn defaults_are_applied_by_rust_and_private_paths_are_not_public() {
  let mut raw = input();
  raw["basePath"] = json!("/guide");
  raw["theme"] = json!({ "customCss": ["custom.css"], "components": "layout.tsx", "navbar": [{ "label": "Docs", "to": "/docs" }] });
  let config = parse_config(&raw.to_string(), &root()).unwrap();
  let public = serde_json::to_value(&config.public).unwrap();
  assert_eq!(public["schemaVersion"], 1);
  assert_eq!(public["url"], "https://farmfe.org");
  assert_eq!(public["basePath"], "/guide/");
  assert_eq!(public["locales"], json!({"default": "en", "items": ["en"]}));
  assert_eq!(public["docs"]["defaultVersion"], "current");
  assert_eq!(public["collections"][0]["locale"], "en");
  assert_eq!(public["collections"][0]["version"], "current");
  assert!(config.collections[0].directory.is_absolute());
  assert_eq!(config.custom_css.len(), 1);
  for private in [
    "custom.css",
    "layout.tsx",
    "directory",
    "customCss",
    "components",
  ] {
    assert!(
      !public.to_string().contains(private),
      "leaked private input {private}: {public}"
    );
  }
}

#[test]
fn schema_and_deserializer_reject_unknown_fields() {
  for (pointer, value) in [
    ("", json!({"serverToken": "private"})),
    ("/docs/collections/0", json!({"unknown": true})),
    ("/theme", json!({"unknown": true})),
  ] {
    let mut raw = input();
    if pointer == "/theme" {
      raw["theme"] = json!({});
    }
    raw
      .pointer_mut(pointer)
      .unwrap()
      .as_object_mut()
      .unwrap()
      .extend(value.as_object().unwrap().clone());
    reject(raw, "DOCS_CONFIG_PARSE", "unknown field");
  }
  let generated = serde_json::to_value(schema()).unwrap();
  assert_eq!(generated["additionalProperties"], false);
  assert!(generated["properties"]["schemaVersion"].is_object());
}

#[test]
fn version_and_duplicate_collection_ids_are_rejected() {
  let mut raw = input();
  raw["schemaVersion"] = json!(2);
  reject(raw, "DOCS_CONFIG_VERSION", "schemaVersion");
  let mut raw = input();
  let duplicate = raw["docs"]["collections"][0].clone();
  raw["docs"]["collections"]
    .as_array_mut()
    .unwrap()
    .push(duplicate);
  reject(raw, "DOCS_CONFIG_COLLECTION", "id");
}

#[test]
fn locales_and_default_version_are_checked() {
  let mut raw = input();
  raw["locales"] = json!({"default": "en", "items": ["zh"]});
  reject(raw, "DOCS_CONFIG_LOCALE", "locales.default");
  let mut raw = input();
  raw["docs"]["collections"][0]["locale"] = json!("xx");
  reject(raw, "DOCS_CONFIG_LOCALE", "locale");
  let mut raw = input();
  raw["docs"]["collections"][0]["version"] = json!("old");
  reject(raw, "DOCS_CONFIG_VERSION", "docs.defaultVersion");
}

#[test]
fn unsafe_url_paths_and_directories_are_rejected() {
  for url in [
    "javascript:alert(1)",
    "https://user:password@example.com",
    "https://example.com/path?key=private",
  ] {
    let mut raw = input();
    raw["url"] = json!(url);
    reject(raw, "DOCS_CONFIG_URL", "url");
  }
  for path in ["/../escape/", "/%2e%2e/", "//host/", "/docs?x=1"] {
    let mut raw = input();
    raw["basePath"] = json!(path);
    reject(raw, "DOCS_CONFIG_PATH", "basePath");
  }
  for directory in ["../outside", "missing", "custom.css", "C:\\outside"] {
    let mut raw = input();
    raw["docs"]["collections"][0]["dir"] = json!(directory);
    reject(raw, "DOCS_CONFIG_PATH", "dir");
  }
}

#[test]
fn public_navigation_cannot_expose_file_inputs_or_script_urls() {
  for target in [
    "C:/private/layout.tsx",
    "javascript:alert(1)",
    "./layout.tsx",
  ] {
    let mut raw = input();
    raw["theme"] = json!({"navbar": [{"label": "Bad", "to": target}]});
    reject(raw, "DOCS_CONFIG_URL", "theme.navbar");
  }
}

#[test]
fn public_navigation_supports_real_anchor_and_query_targets() {
  let mut raw = input();
  raw["theme"] = json!({"navbar": [
    {"label": "Heading", "to": "/docs/#stable-id"},
    {"label": "Search", "to": "/docs/?q=rust"}
  ]});
  let config = parse_config(&raw.to_string(), &root()).unwrap();
  assert_eq!(config.public.theme.navbar[0].to, "/docs/#stable-id");
  assert_eq!(config.public.theme.navbar[1].to, "/docs/?q=rust");
}

#[test]
fn public_payloads_do_not_silently_reapply_input_defaults() {
  let mut raw = input();
  raw["theme"] = json!({"logo": {"src": "/logo.svg"}});
  let config = parse_config(&raw.to_string(), &root()).unwrap();
  let public = serde_json::to_value(&config.public).unwrap();
  for (parent, field) in [
    ("/locales", "default"),
    ("/locales", "items"),
    ("/theme/footer", "links"),
    ("/theme/logo", "alt"),
    ("/theme/logo", "href"),
  ] {
    let mut broken = public.clone();
    broken
      .pointer_mut(parent)
      .unwrap()
      .as_object_mut()
      .unwrap()
      .remove(field);
    assert!(
      serde_json::from_value::<PublicSiteConfig>(broken).is_err(),
      "missing public {parent}/{field} was silently defaulted"
    );
  }
}
