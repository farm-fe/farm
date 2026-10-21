use farmfe_docs_config::parse_config;
use farmfe_docs_plan::prepare;
use serde_json::{Value, json};
use std::{fs, path::Path};

fn fixture() -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  for name in ["pages", "docs", "blog"] {
    fs::create_dir(root.path().join(name)).unwrap();
  }
  fs::write(
    root.path().join("pages/index.md"),
    "# Home\n\n[Next](/docs/next/?q=one#next)\n",
  )
  .unwrap();
  fs::write(root.path().join("docs/index.md"), "# Intro\n").unwrap();
  fs::write(root.path().join("docs/next.md"), "# Next\n").unwrap();
  fs::write(root.path().join("blog/index.md"), "# Post\n").unwrap();
  root
}
fn site() -> Value {
  json!({"title":"URLs","url":"https://example.com","basePath":"/guide/","locales":{"default":"en","items":["en","zh"]},"pages":{"dir":"pages"},"docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]},"blog":{"collections":[{"id":"news","dir":"blog","routeBase":"/blog/"}]}})
}
fn plan(root: &Path, site: Value) -> Result<Value, String> {
  let config = parse_config(&site.to_string(), root).map_err(|e| e.to_string())?;
  serde_json::to_value(prepare(&config)?).map_err(|e| e.to_string())
}

#[test]
fn default_routes_keep_trailing_slashes_with_explicit_canonical_paths() {
  let root = fixture();
  let plan = plan(root.path(), site()).unwrap();
  for route in plan["routes"].as_array().unwrap() {
    assert!(route["page"]["routePath"].as_str().unwrap().ends_with('/'));
    assert_eq!(route["page"]["canonicalPath"], route["page"]["routePath"]);
  }
  assert_eq!(plan["site"]["trailingSlash"], "always");
}

#[test]
fn browser_application_defers_each_content_edge_and_server_keeps_sync_rendering() {
  let root = fixture();
  let config = parse_config(&site().to_string(), root.path()).unwrap();
  let plan = prepare(&config).unwrap();
  let browser = farmfe_docs_plan::browser_application(&plan).unwrap();
  let server = farmfe_docs_plan::application(&plan).unwrap();
  assert!(!browser.contains("import Content"));
  assert!(server.contains("import Content0 from"));
  assert!(browser.contains("export function loadPage(key)"));
  for module in &plan.modules {
    let key = serde_json::to_string(&module.key).unwrap();
    let specifier = serde_json::to_string(&format!("docs:content:{}", module.key)).unwrap();
    let entry = serde_json::to_string(&format!("docs:page:{}", module.key)).unwrap();
    assert!(browser.contains(&format!("{key}: () => import({entry})")));
    assert!(server.contains(&format!("from {specifier}")));
  }
}

#[test]
fn application_registers_both_slash_variants_in_rust_without_empty_root_aliases() {
  let root = fixture();
  for policy in ["always", "never"] {
    for base in ["/guide/", "/"] {
      let mut value = site();
      value["trailingSlash"] = json!(policy);
      value["basePath"] = json!(base);
      let config = parse_config(&value.to_string(), root.path()).unwrap();
      let prepared = prepare(&config).unwrap();
      let code = farmfe_docs_plan::application(&prepared).unwrap();
      let serialized = code
        .split_once("export const routeAliases = ")
        .expect("native application must export explicit runtime aliases")
        .1
        .split_once(";\n")
        .unwrap()
        .0;
      let aliases: std::collections::BTreeMap<String, String> =
        serde_json::from_str(serialized).unwrap();
      assert!(!aliases.contains_key(""));
      for route in &prepared.routes {
        let bare = route.page.route_path.trim_end_matches('/');
        let paths = if bare.is_empty() {
          vec!["/".into()]
        } else {
          vec![bare.to_string(), format!("{bare}/")]
        };
        for path in paths {
          assert_eq!(aliases.get(&path), Some(&route.page.key), "{policy} {path}");
        }
      }
    }
  }
}

#[test]
fn never_policy_changes_all_public_routes_but_preserves_directory_outputs() {
  let root = fixture();
  let before = plan(root.path(), site()).unwrap();
  let mut config = site();
  config["trailingSlash"] = json!("never");
  let after = plan(root.path(), config).unwrap();
  for (old, new) in before["routes"]
    .as_array()
    .unwrap()
    .iter()
    .zip(after["routes"].as_array().unwrap())
  {
    let path = old["page"]["routePath"].as_str().unwrap();
    let expected = if path == "/guide/" {
      path
    } else {
      path.trim_end_matches('/')
    };
    assert_eq!(new["page"]["routePath"], expected);
    assert_eq!(new["page"]["canonicalPath"], expected);
    assert_eq!(new["outputPath"], old["outputPath"]);
  }
  let home = after["routes"]
    .as_array()
    .unwrap()
    .iter()
    .position(|r| r["page"]["routePath"] == "/guide/")
    .unwrap();
  assert!(
    after["modules"][home]["code"]
      .as_str()
      .unwrap()
      .contains("/guide/docs/next?q=one#next")
  );
  for route in after["routes"].as_array().unwrap() {
    for field in ["previous", "next"] {
      if let Some(href) = route["page"][field]["href"].as_str() {
        assert!(!href.ends_with('/'), "{href}");
      }
    }
  }
}

#[test]
fn locale_home_canonical_override_does_not_change_route_or_output() {
  let root = fixture();
  let mut config = site();
  config["trailingSlash"] = json!("never");
  config["canonicalOverrides"] = json!([{"path":"/guide/zh","canonicalPath":"/guide/zh/"}]);
  let result = plan(root.path(), config).unwrap();
  let zh = result["routes"]
    .as_array()
    .unwrap()
    .iter()
    .find(|r| r["page"]["routePath"] == "/guide/zh")
    .unwrap();
  assert_eq!(zh["page"]["canonicalPath"], "/guide/zh/");
  assert_eq!(zh["outputPath"], "zh/index.html");
  assert!(result["site"].get("canonicalOverrides").is_none());
}

#[test]
fn canonical_overrides_reject_unknown_duplicate_escaped_and_stale_targets() {
  let root = fixture();
  for overrides in [
    json!([{"path":"/guide/absent","canonicalPath":"/guide/"}]),
    json!([{"path":"/guide/","canonicalPath":"/guide/absent"}]),
    json!([{"path":"/guide/zh","canonicalPath":"/guide/zh/"},{"path":"/guide/zh/","canonicalPath":"/guide/zh"}]),
    json!([{"path":"/guide/zh","canonicalPath":"/outside"}]),
    json!([{"path":"/guide/zh","canonicalPath":"/guide/%2e%2e/escape"}]),
    json!([{"path":"/guide/zh","canonicalPath":"https://other.example/"}]),
  ] {
    let mut config = site();
    config["trailingSlash"] = json!("never");
    config["canonicalOverrides"] = overrides;
    let error = plan(root.path(), config).unwrap_err();
    assert!(error.contains("DOCS_CANONICAL"), "{error}");
  }
}

#[test]
fn never_policy_preserves_real_translation_homes_and_fallback_provenance() {
  let root = tempfile::tempdir().unwrap();
  let mut collections = vec![];
  for (locale, version, directory, route_base) in [
    ("en", "current", "en", "/docs/"),
    ("zh", "current", "zh", "/zh/docs/"),
    ("en", "old", "old", "/v1/"),
    ("zh", "old", "zh-old", "/zh/v1/"),
  ] {
    fs::create_dir(root.path().join(directory)).unwrap();
    fs::write(root.path().join(directory).join("index.md"), "# Home\n").unwrap();
    let mut collection = json!({"id":directory,"group":"guide","locale":locale,"version":version,"dir":directory,"routeBase":route_base});
    if locale == "zh" && version == "current" {
      collection["fallbackLocale"] = json!("en");
    }
    collections.push(collection);
  }
  fs::write(root.path().join("en/aaa.md"), "# Extra\n").unwrap();
  let result = plan(root.path(), json!({"title":"Matrix","url":"https://example.com","basePath":"/guide/","trailingSlash":"never","locales":{"default":"en","items":["en","zh"]},"docs":{"collections":collections}})).unwrap();
  let fallback = result["routes"]
    .as_array()
    .unwrap()
    .iter()
    .find(|r| r["page"]["routePath"] == "/guide/zh/docs/aaa")
    .unwrap();
  assert_eq!(
    fallback["page"]["fallbackSource"]["routePath"],
    "/guide/docs/aaa"
  );
  let alternates = fallback["page"]["alternates"].as_array().unwrap();
  assert!(
    alternates
      .iter()
      .any(|a| a["missing"] == true && a["routePath"] == "/guide/v1")
  );
  assert!(
    alternates
      .iter()
      .any(|a| a["missing"] == true && a["routePath"] == "/guide/zh/v1")
  );
}

#[test]
fn root_mount_and_explicit_cross_locale_canonical_use_actual_pages() {
  let root = fixture();
  let mut config = site();
  config["basePath"] = json!("/");
  config["trailingSlash"] = json!("never");
  config["canonicalOverrides"] =
    json!([{"path":"/zh","canonicalPath":"/zh/"},{"path":"/docs","canonicalPath":"/zh/"}]);
  let result = plan(root.path(), config).unwrap();
  let pages = result["routes"].as_array().unwrap();
  assert!(
    pages
      .iter()
      .any(|r| r["page"]["routePath"] == "/" && r["outputPath"] == "index.html")
  );
  assert!(
    pages
      .iter()
      .any(|r| r["page"]["routePath"] == "/docs" && r["page"]["canonicalPath"] == "/zh/")
  );
  assert!(
    pages
      .iter()
      .any(|r| r["page"]["routePath"] == "/zh" && r["outputPath"] == "zh/index.html")
  );
}

#[test]
fn canonical_chains_cycles_and_wrong_case_do_not_publish_unreachable_targets() {
  let root = fixture();
  for overrides in [
    json!([{"path":"/guide/docs","canonicalPath":"/guide/zh"},{"path":"/guide/zh","canonicalPath":"/guide/"}]),
    json!([{"path":"/guide/docs","canonicalPath":"/guide/zh"},{"path":"/guide/zh","canonicalPath":"/guide/docs"}]),
    json!([{"path":"/guide/docs","canonicalPath":"/guide/ZH"}]),
    json!([{"path":"/guide/ZH","canonicalPath":"/guide/zh"}]),
  ] {
    let mut config = site();
    config["trailingSlash"] = json!("never");
    config["canonicalOverrides"] = overrides;
    let error = plan(root.path(), config).unwrap_err();
    assert!(error.contains("DOCS_CANONICAL"), "{error}");
  }
}
