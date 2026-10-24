use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{application, prepare};
use serde_json::{Value, json};
use std::fs;

#[test]
fn equivalent_origins_cannot_bypass_route_and_anchor_validation() {
  let root = fixture();
  for (href, expected) in [
    (
      "https://EXAMPLE.com:443/docs/one/#one-section",
      Some("/guide/docs/one/#one-section"),
    ),
    ("https://example.com:443/missing", None),
    ("https://EXAMPLE.com/docs/one/#missing", None),
    (
      "https://example.com.evil.example/missing",
      Some("https://example.com.evil.example/missing"),
    ),
  ] {
    let mut value = site();
    value["theme"] = json!({"logo":{"src":"https://assets.example.com/logo.svg","href":href}});
    let config = parse_config(&value.to_string(), root.path()).unwrap();
    let result = prepare(&config);
    if let Some(expected) = expected {
      assert_eq!(result.unwrap().site.theme.logo.unwrap().href, expected);
    } else {
      assert!(result.unwrap_err().contains("DOCS_BRAND_HOME"));
    }
  }
}

fn fixture() -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(
    root.path().join("docs/one.md"),
    "# First\n\n## One section\n",
  )
  .unwrap();
  fs::write(
    root.path().join("docs/two.md"),
    "# Second\n\n## Two section\n",
  )
  .unwrap();
  fs::write(
    root.path().join("sidebar.json"),
    r#"{"items":[{"type":"doc","id":"two"},{"type":"doc","id":"one"}]}"#,
  )
  .unwrap();
  root
}
fn site() -> Value {
  json!({"title":"Brand","url":"https://example.com","basePath":"/guide/","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/","sidebar":"sidebar.json"}]}})
}

#[test]
fn real_site_root_wins_and_sidebar_order_is_the_documentation_fallback() {
  let root = fixture();
  let config = parse_config(&site().to_string(), root.path()).unwrap();
  let plan = prepare(&config).unwrap();
  assert!(
    application(&plan)
      .unwrap()
      .contains("export const home = \"/guide/docs/two/\";")
  );
  fs::create_dir(root.path().join("pages")).unwrap();
  fs::write(root.path().join("pages/index.md"), "# Site home\n").unwrap();
  let mut value = site();
  value["pages"] = json!({"dir":"pages"});
  let config = parse_config(&value.to_string(), root.path()).unwrap();
  assert!(
    application(&prepare(&config).unwrap())
      .unwrap()
      .contains("export const home = \"/guide/\";")
  );
}

#[test]
fn default_locale_and_version_take_priority_over_collection_declaration_order() {
  let root = fixture();
  for directory in ["zh", "old", "current"] {
    fs::create_dir(root.path().join(directory)).unwrap();
    fs::write(root.path().join(directory).join("index.md"), "# Home\n").unwrap();
  }
  let value = json!({"title":"Brand","url":"https://example.com","basePath":"/guide/","locales":{"default":"en","items":["en","zh"]},"docs":{"defaultVersion":"current","collections":[{"id":"zh","dir":"zh","locale":"zh","version":"current","routeBase":"/zh/"},{"id":"old","dir":"old","locale":"en","version":"old","routeBase":"/v1/"},{"id":"current","dir":"current","locale":"en","version":"current","routeBase":"/docs/"}]}});
  let config = parse_config(&value.to_string(), root.path()).unwrap();
  assert!(
    application(&prepare(&config).unwrap())
      .unwrap()
      .contains("export const home = \"/guide/docs/\";")
  );
}

#[test]
fn logo_overrides_use_the_same_real_route_and_anchor_validation_as_content_links() {
  let root = fixture();
  let mut value = site();
  value["theme"] = json!({"logo":{"src":"https://assets.example.com/logo.svg","href":"/docs/one/?source=brand#one-section"}});
  let config = parse_config(&value.to_string(), root.path()).unwrap();
  let plan = prepare(&config).unwrap();
  assert_eq!(
    plan.site.theme.logo.as_ref().unwrap().href,
    "/guide/docs/one/?source=brand#one-section"
  );
  for href in ["/missing/", "/docs/two/#missing"] {
    value["theme"]["logo"]["href"] = json!(href);
    let config = parse_config(&value.to_string(), root.path()).unwrap();
    assert!(prepare(&config).unwrap_err().contains("DOCS_BRAND_HOME"));
  }
}

#[test]
fn logo_root_fragment_and_same_origin_absolute_url_resolve_in_the_selected_home() {
  let root = fixture();
  for (href, expected) in [
    (
      "/?source=brand#two-section",
      "/guide/docs/two/?source=brand#two-section",
    ),
    ("#two-section", "/guide/docs/two/#two-section"),
    (
      "https://example.com/docs/one/#one-section",
      "/guide/docs/one/#one-section",
    ),
  ] {
    let mut value = site();
    value["theme"] = json!({"logo":{"src":"https://assets.example.com/logo.svg","href":href}});
    let config = parse_config(&value.to_string(), root.path()).unwrap();
    assert_eq!(
      prepare(&config).unwrap().site.theme.logo.unwrap().href,
      expected
    );
  }
}
