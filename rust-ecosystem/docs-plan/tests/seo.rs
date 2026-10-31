use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{DocPlan, prepare};
use serde_json::{Value, json};
use std::fs;

fn fixture() -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  for dir in ["en", "zh", "old", "old-zh"] {
    fs::create_dir(root.path().join(dir)).unwrap();
  }
  for dir in ["en", "old"] {
    fs::write(
      root.path().join(dir).join("index.md"),
      "---\nid: intro\n---\n# Intro\n",
    )
    .unwrap();
  }
  root
}

fn config() -> Value {
  json!({"title":"SEO & 中文","url":"https://example.com","basePath":"/guide/",
  "locales":{"default":"en","items":["en","zh"]},"docs":{"collections":[
    {"id":"en","group":"guide","dir":"en","locale":"en","version":"current","routeBase":"/docs/"},
    {"id":"zh","group":"guide","dir":"zh","locale":"zh","version":"current","routeBase":"/zh/docs/","fallbackLocale":"en"},
    {"id":"old","group":"guide","dir":"old","locale":"en","version":"previous","routeBase":"/v1/docs/"},
    {"id":"old-zh","group":"guide","dir":"old-zh","locale":"zh","version":"previous","routeBase":"/zh/v1/docs/","fallbackLocale":"en"}
  ]}})
}

fn plan(root: &std::path::Path, raw: Value) -> DocPlan {
  prepare(&parse_config(&raw.to_string(), root).unwrap()).unwrap()
}

#[test]
fn rust_heads_own_document_titles_and_do_not_repeat_the_site_name() {
  let root = fixture();
  let mut raw = config();
  raw["title"] = json!("Intro");
  let prepared = plan(root.path(), raw);
  let value = serde_json::to_value(&prepared).unwrap();
  for route in &prepared.routes {
    assert_eq!(value["seo"][&route.page.key]["title"], "Intro");
  }
  let prepared = plan(root.path(), config());
  let value = serde_json::to_value(&prepared).unwrap();
  for route in &prepared.routes {
    assert_eq!(value["seo"][&route.page.key]["title"], "Intro | SEO & 中文");
  }
}

#[test]
fn fallback_heads_are_noindex_and_never_advertise_a_fake_translation_or_other_version() {
  let root = fixture();
  let prepared = plan(root.path(), config());
  let value = serde_json::to_value(&prepared).unwrap();
  for route in &prepared.routes {
    let head = &value["seo"][&route.page.key];
    assert_eq!(
      head["canonicalUrl"],
      format!("https://example.com{}", route.page.canonical_path)
    );
    assert_eq!(
      head["robots"],
      if route.page.fallback_source.is_some() {
        "noindex,follow"
      } else {
        "index,follow"
      }
    );
    let alternates = head["alternates"].as_array().expect("Rust SEO registry");
    assert_eq!(alternates.len(), 1);
    assert_eq!(alternates[0]["locale"], "en");
    assert_eq!(
      alternates[0]["href"],
      if route.page.version.as_deref() == Some("previous") {
        "https://example.com/guide/v1/docs/"
      } else {
        "https://example.com/guide/docs/"
      }
    );
  }
}

#[test]
fn explicit_fallback_indexing_changes_snapshot_but_not_translation_truth() {
  let root = fixture();
  let original = plan(root.path(), config());
  let mut raw = config();
  raw["seo"] = json!({"fallbackIndexing":"index"});
  let indexed = plan(root.path(), raw);
  assert_ne!(indexed.snapshot_id, original.snapshot_id);
  let value = serde_json::to_value(&indexed).unwrap();
  let fallback = indexed
    .routes
    .iter()
    .find(|route| route.page.fallback_source.is_some())
    .unwrap();
  assert_eq!(value["seo"][&fallback.page.key]["robots"], "index,follow");
  assert_eq!(
    value["seo"][&fallback.page.key]["alternates"]
      .as_array()
      .unwrap()
      .len(),
    1
  );
}

#[test]
fn real_translation_heads_are_same_version_deterministic_and_public_application_exports_them() {
  let root = fixture();
  fs::write(
    root.path().join("zh/index.md"),
    "---\nid: intro\n---\n# 中文\n",
  )
  .unwrap();
  let prepared = plan(root.path(), config());
  let repeated = plan(root.path(), config());
  assert_eq!(prepared.snapshot_id, repeated.snapshot_id);
  let value = serde_json::to_value(&prepared).unwrap();
  for route in prepared
    .routes
    .iter()
    .filter(|route| route.page.version.as_deref() == Some("current"))
  {
    let head = &value["seo"][&route.page.key];
    assert_eq!(head["alternates"].as_array().unwrap().len(), 2);
    assert!(head["html"].as_str().unwrap().contains("hreflang=\"zh\""));
    assert!(!head["html"].as_str().unwrap().contains("v1"));
  }
  for code in [
    farmfe_docs_plan::application(&prepared).unwrap(),
    farmfe_docs_plan::browser_application(&prepared).unwrap(),
  ] {
    assert!(code.contains("export const seoByKey = "));
    assert!(!code.contains(&root.path().to_string_lossy().replace('\\', "/")));
  }
}

#[test]
fn seo_urls_serialize_unicode_and_reject_escaped_or_outside_site_routes() {
  let root = fixture();
  let prepared = plan(root.path(), config());
  assert_eq!(
    farmfe_docs_plan::seo_url(&prepared.site, "/guide/入门/").unwrap(),
    "https://example.com/guide/%E5%85%A5%E9%97%A8/"
  );
  for path in [
    "https://other.example/",
    "//other.example/",
    "/outside/",
    "/guide/%2e%2e/",
    "/guide/a?b",
    "/guide/a#b",
    "/guide/../escape",
    "/guide/a\\b",
    "/guide/a\nb",
  ] {
    assert!(
      farmfe_docs_plan::seo_url(&prepared.site, path).is_err(),
      "{path:?}"
    );
  }
}

#[test]
fn forged_head_policy_is_rejected_even_with_a_recomputed_plan_digest() {
  use sha2::{Digest, Sha256};
  let root = fixture();
  let mut prepared = plan(root.path(), config());
  farmfe_docs_plan::verify(&prepared).unwrap();
  let fallback = prepared
    .routes
    .iter()
    .find(|route| route.page.fallback_source.is_some())
    .unwrap()
    .page
    .key
    .clone();
  prepared.seo.get_mut(&fallback).unwrap().robots = "index,follow".into();
  prepared.snapshot_id.clear();
  prepared.snapshot_id = format!(
    "{:x}",
    Sha256::digest(serde_json::to_vec(&prepared).unwrap())
  );
  let error = farmfe_docs_plan::verify(&prepared).unwrap_err();
  assert!(error.contains("DOCS_PLAN_SEO"), "{error}");
}

#[test]
fn blog_heads_retain_author_publication_time_without_inventing_dates_for_views() {
  let root = fixture();
  fs::create_dir(root.path().join("blog")).unwrap();
  for (name, source) in [
    (
      "timed.md",
      "---\ndate: 2025-08-30T12:37:05.000Z\n---\n# Timed\n",
    ),
    ("day.md", "---\ndate: 2024-01-02\n---\n# Day\n"),
    ("undated.md", "# Undated\n"),
  ] {
    fs::write(root.path().join("blog").join(name), source).unwrap();
  }
  let mut raw = config();
  raw["blog"] = json!({"collections":[{"id":"news","dir":"blog","routeBase":"/blog/"}]});
  let prepared = plan(root.path(), raw);
  let heads = serde_json::to_value(&prepared).unwrap();
  let mut dated_articles = 0;
  for route in &prepared.routes {
    let html = heads["seo"][&route.page.key]["html"].as_str().unwrap();
    match route.page.route_path.as_str() {
      "/guide/blog/timed/" => {
        dated_articles += 1;
        assert!(html.contains(
          "<meta property=\"article:published_time\" content=\"2025-08-30T12:37:05.000Z\">"
        ));
      }
      "/guide/blog/day/" => {
        dated_articles += 1;
        assert!(html.contains("<meta property=\"article:published_time\" content=\"2024-01-02\">"))
      }
      _ => assert!(
        !html.contains("article:published_time"),
        "{}",
        route.page.route_path
      ),
    }
  }
  assert_eq!(
    dated_articles, 2,
    "both authored date cases must be exercised"
  );
  farmfe_docs_plan::verify(&prepared).unwrap();
}
