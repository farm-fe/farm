use farmfe_docs_config::parse_config;
use farmfe_docs_plan::prepare;
use serde_json::{Value, json};
use std::{fs, path::Path};

fn fixture() -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("blog")).unwrap();
  fs::write(root.path().join("blog/index.md"), "# Blob\n\n## 2024\n").unwrap();
  root
}
fn config() -> Value {
  json!({"title":"Blog","url":"https://example.com","basePath":"/guide/","blog":{"collections":[{"id":"news","dir":"blog","routeBase":"/blog/"}]}})
}
fn plan(root: &Path, raw: Value) -> Result<farmfe_docs_plan::DocPlan, String> {
  let config = parse_config(&raw.to_string(), root).map_err(|error| error.to_string())?;
  prepare(&config)
}

#[test]
fn blog_index_article_is_distinct_from_the_collection_landing_page() {
  let root = fixture();
  let prepared = plan(root.path(), config()).unwrap();
  let page = &prepared.routes[0].page;
  assert_eq!(page.kind, "blog");
  assert_eq!(page.route_path, "/guide/blog/index/");
  assert_eq!(page.version, None);
  let value = serde_json::to_value(&prepared).unwrap();
  assert!(value["routes"][0]["page"]["blog"].get("date").is_none());
  assert_eq!(value["blogIndexes"][0]["posts"][0], page.key);
  assert_eq!(value["blogIndexes"][0]["undated"][0], page.key);
  assert!(prepared.sidebars.is_empty());
  assert!(prepared.modules[0].code.contains("Blob"));
}

#[test]
fn dated_posts_expose_static_authors_tags_and_descending_indexes() {
  let root = fixture();
  fs::write(root.path().join("blog/2024-02-29-release.mdx"), "---\nid: release\ntitle: Release\nauthors:\n  - Alice\n  - name: Bob\n    url: https://example.com/bob\ntags:\n  - Rust\n  - id: farm-v2\n    label: Farm v2\n---\n# Release\n").unwrap();
  fs::write(
    root.path().join("blog/older.md"),
    "---\ndate: 2023-01-02\ntags: [Rust]\n---\n# Older\n",
  )
  .unwrap();
  let prepared = plan(root.path(), config()).unwrap();
  let release = prepared
    .routes
    .iter()
    .find(|route| route.page.id == "release")
    .unwrap();
  assert_eq!(release.page.route_path, "/guide/blog/2024/02/29/release/");
  let value = serde_json::to_value(&prepared).unwrap();
  let post = &value["routes"]
    .as_array()
    .unwrap()
    .iter()
    .find(|route| route["page"]["id"] == "release")
    .unwrap()["page"]["blog"];
  assert_eq!(post["date"], "2024-02-29");
  assert_eq!(post["authors"][1]["name"], "Bob");
  assert_eq!(post["tags"][0]["id"], "rust");
  let index = &value["blogIndexes"][0];
  assert_eq!(index["posts"][0], release.page.key);
  assert_eq!(index["years"][0]["year"], "2024");
  assert_eq!(index["years"][1]["year"], "2023");
  assert_eq!(index["tags"][1]["id"], "rust");
  assert_eq!(index["tags"][1]["posts"].as_array().unwrap().len(), 2);
  assert_eq!(
    prepared.snapshot_id,
    plan(root.path(), config()).unwrap().snapshot_id
  );
}

#[test]
fn invalid_blog_metadata_is_rejected_instead_of_silently_dropped() {
  for (metadata, code) in [
    ("date: 2023-02-29", "DOCS_BLOG_DATE"),
    ("date: 2024-13-01", "DOCS_BLOG_DATE"),
    ("date: 1", "DOCS_BLOG_DATE"),
    ("authors: [1]", "DOCS_BLOG_AUTHOR"),
    (
      "authors: [{name: Bob, url: 'javascript:alert(1)'}]",
      "DOCS_BLOG_AUTHOR",
    ),
    ("authors: [{name: Bob, secret: true}]", "DOCS_BLOG_AUTHOR"),
    ("tags: [Rust, Rust]", "DOCS_BLOG_TAG"),
    ("tags: [C++, C#]", "DOCS_BLOG_TAG"),
    ("slug: archive", "DOCS_BLOG_ROUTE"),
    ("slug: /", "DOCS_BLOG_ROUTE"),
  ] {
    let root = fixture();
    fs::write(
      root.path().join("blog/index.md"),
      format!("---\n{metadata}\n---\n# Post\n"),
    )
    .unwrap();
    let error = plan(root.path(), config()).unwrap_err();
    assert!(
      error.contains(code) && error.contains("index.md"),
      "{metadata}: {error}"
    );
    fs::write(root.path().join("blog/index.md"), "# Fixed\n").unwrap();
    assert!(plan(root.path(), config()).is_ok());
  }
}

#[test]
fn blog_fallback_metadata_and_navigation_remain_in_the_target_collection() {
  let root = fixture();
  fs::write(
    root.path().join("blog/next.md"),
    "---\ndate: 2024-01-01\ntags: [Rust]\n---\n# Next\n",
  )
  .unwrap();
  fs::create_dir(root.path().join("zh-blog")).unwrap();
  fs::write(root.path().join("zh-blog/index.md"), "# 中文文章\n").unwrap();
  let mut raw = config();
  raw["locales"] = json!({"default":"en","items":["en","zh"]});
  raw["blog"]["collections"][0]["group"] = json!("news");
  raw["blog"]["collections"].as_array_mut().unwrap().push(json!({"id":"zh-news","group":"news","dir":"zh-blog","routeBase":"/zh/blog/","locale":"zh","fallbackLocale":"en"}));
  let prepared = plan(root.path(), raw).unwrap();
  let fallback = &prepared
    .routes
    .iter()
    .find(|route| route.page.collection == "zh-news" && route.page.id == "next")
    .unwrap()
    .page;
  assert!(fallback.fallback_source.is_some());
  assert_eq!(
    fallback.next.as_ref().unwrap().href,
    "/guide/zh/blog/index/"
  );
  let value = serde_json::to_value(&prepared).unwrap();
  assert_eq!(value["blogIndexes"].as_array().unwrap().len(), 2);
}

#[test]
fn docs_and_blog_indexes_do_not_mix_or_share_sidebar_navigation() {
  let root = fixture();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(root.path().join("docs/index.md"), "# Docs\n").unwrap();
  let mut raw = config();
  raw["docs"] = json!({"collections":[{"id":"guide","dir":"docs","routeBase":"/docs/"}]});
  let prepared = plan(root.path(), raw).unwrap();
  assert_eq!(prepared.sidebars.len(), 1);
  assert_eq!(prepared.sidebars[0].collection, "guide");
  let value = serde_json::to_value(&prepared).unwrap();
  assert_eq!(
    value["blogIndexes"][0]["posts"].as_array().unwrap().len(),
    1
  );
}

#[test]
fn invalid_dated_filenames_and_global_tag_id_collisions_fail() {
  let root = fixture();
  fs::write(root.path().join("blog/2023-02-29-bad.md"), "# Bad\n").unwrap();
  let error = plan(root.path(), config()).unwrap_err();
  assert!(
    error.contains("DOCS_BLOG_DATE") && error.contains("2023-02-29-bad.md"),
    "{error}"
  );
  fs::remove_file(root.path().join("blog/2023-02-29-bad.md")).unwrap();
  fs::write(
    root.path().join("blog/index.md"),
    "---\ntags: [C++]\n---\n# First\n",
  )
  .unwrap();
  fs::write(
    root.path().join("blog/other.md"),
    "---\ntags: [C#]\n---\n# Second\n",
  )
  .unwrap();
  let error = plan(root.path(), config()).unwrap_err();
  assert!(
    error.contains("DOCS_BLOG_TAG") && error.contains("conflicting labels"),
    "{error}"
  );
}

#[test]
fn explicit_publication_timestamp_preserves_the_baseline_time_and_archive_day() {
  let root = fixture();
  fs::write(
    root.path().join("blog/index.md"),
    "---\ndate: '2025-08-30T12:37:05.000Z'\n---\n# Blob\n",
  )
  .unwrap();
  fs::write(
    root.path().join("blog/offset.md"),
    "---\ndate: '2025-08-31T01:00:00+14:00'\n---\n# Offset\n",
  )
  .unwrap();
  let prepared = plan(root.path(), config()).unwrap();
  let value = serde_json::to_value(&prepared).unwrap();
  assert_eq!(value["routes"][0]["page"]["blog"]["date"], "2025-08-30");
  assert_eq!(
    value["routes"][0]["page"]["blog"]["publishedAt"],
    "2025-08-30T12:37:05.000Z"
  );
  assert_eq!(value["blogIndexes"][0]["years"][0]["year"], "2025");
  assert_eq!(
    value["blogIndexes"][0]["posts"][0], prepared.routes[0].page.key,
    "sort by actual instant, not the offset-local day"
  );
}
