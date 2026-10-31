use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{prepare, verify};
use serde_json::json;
use std::fs;

fn fixture(empty: bool) -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("blog")).unwrap();
  if !empty {
    fs::write(
      root.path().join("blog/index.md"),
      "---\ndate: 2025-01-02\ntags: [Rust]\ntitle: 'A [title] <unsafe>'\n---\n# Article\n",
    )
    .unwrap();
  }
  root
}
fn plan(root: &std::path::Path) -> farmfe_docs_plan::DocPlan {
  let config = parse_config(&json!({"title":"Blog","url":"https://example.com","basePath":"/guide/","blog":{"collections":[{"id":"news","dir":"blog","routeBase":"/blog/"}]}}).to_string(),root).unwrap();
  prepare(&config).unwrap()
}
#[test]
fn actual_generated_views_have_bodies_and_verified_virtual_sources() {
  let root = fixture(false);
  let prepared = plan(root.path());
  let paths: Vec<_> = prepared
    .routes
    .iter()
    .map(|route| route.page.route_path.as_str())
    .collect();
  for path in [
    "/guide/blog/",
    "/guide/blog/archive/",
    "/guide/blog/tags/",
    "/guide/blog/tags/rust/",
    "/guide/blog/index/",
  ] {
    assert!(paths.contains(&path), "{paths:?}");
  }
  let index = prepared
    .routes
    .iter()
    .position(|route| route.page.route_path == "/guide/blog/")
    .unwrap();
  assert!(prepared.modules[index].code.contains("/guide/blog/index/"));
  assert!(prepared.modules[index]
    .code
    .contains("\"A [title] <unsafe>\""));
  verify(&prepared).unwrap();
  assert!(!prepared
    .diagnostics
    .iter()
    .any(|value| value.contains("VIEWS_PENDING")));
  fs::write(root.path().join("blog/index.md"), "# Changed\n").unwrap();
  assert!(verify(&prepared)
    .unwrap_err()
    .contains("DOCS_SNAPSHOT_CHANGED"));
}
#[test]
fn an_empty_blog_has_real_listing_and_archive_without_empty_tag_routes() {
  let root = fixture(true);
  let prepared = plan(root.path());
  assert_eq!(prepared.routes.len(), 2);
  assert_eq!(prepared.routes[0].page.route_path, "/guide/blog/");
  assert!(prepared.modules[0].code.contains("No posts"));
  verify(&prepared).unwrap();
}
#[test]
fn source_articles_cannot_claim_the_generated_id_namespace() {
  let root = fixture(false);
  fs::write(
    root.path().join("blog/index.md"),
    "---\nid: '@blog/index'\n---\n# Bad\n",
  )
  .unwrap();
  let config=parse_config(&json!({"title":"Blog","url":"https://example.com","blog":{"collections":[{"id":"news","dir":"blog","routeBase":"/blog/"}]}}).to_string(),root.path()).unwrap();
  assert!(prepare(&config).unwrap_err().contains("DOCS_BLOG_ID"));
}

#[test]
fn generated_source_marker_cannot_disable_real_file_validation() {
  use sha2::{Digest, Sha256};
  let root = fixture(false);
  let prepared = plan(root.path());
  let position = prepared
    .modules
    .iter()
    .position(|module| module.generated_source.is_some())
    .unwrap();
  let mut corrupt = prepared.clone();
  corrupt.modules[position].source_path = prepared.modules[0].source_path.clone();
  let mut data = corrupt.clone();
  data.snapshot_id.clear();
  corrupt.snapshot_id = format!("{:x}", Sha256::digest(serde_json::to_vec(&data).unwrap()));
  assert!(verify(&corrupt).unwrap_err().contains("DOCS_PLAN_SOURCE"));
  let mut corrupt = prepared.clone();
  corrupt.modules[position].generated_source = Some("# Changed".into());
  let mut data = corrupt.clone();
  data.snapshot_id.clear();
  corrupt.snapshot_id = format!("{:x}", Sha256::digest(serde_json::to_vec(&data).unwrap()));
  assert!(verify(&corrupt).unwrap_err().contains("DOCS_PLAN_SOURCE"));
}

#[test]
fn empty_localized_blogs_have_real_alternate_home_pages() {
  let root = fixture(true);
  fs::create_dir(root.path().join("zh-blog")).unwrap();
  let config=parse_config(&json!({"title":"Blog","url":"https://example.com","locales":{"default":"en","items":["en","zh"]},"blog":{"collections":[{"id":"news","group":"news","dir":"blog","routeBase":"/blog/"},{"id":"zh-news","group":"news","dir":"zh-blog","locale":"zh","routeBase":"/zh/blog/"}]}}).to_string(),root.path()).unwrap();
  let prepared = prepare(&config).unwrap();
  let home = &prepared
    .routes
    .iter()
    .find(|route| route.page.route_path == "/blog/")
    .unwrap()
    .page;
  assert_eq!(home.alternates[1].route_path, "/zh/blog/");
  assert!(!home.alternates[1].missing);
}

#[test]
fn collection_view_labels_are_configurable_without_a_fixed_language_table() {
  let root = fixture(false);
  let config=parse_config(&json!({"title":"Blog","url":"https://example.com","blog":{"collections":[{"id":"news","dir":"blog","routeBase":"/blog/","blogLabels":{"archive":"历史博文","title":"新闻"}}]}}).to_string(),root.path()).unwrap();
  let prepared = prepare(&config).unwrap();
  assert_eq!(
    prepared
      .routes
      .iter()
      .find(|route| route.page.kind == "blog-archive")
      .unwrap()
      .page
      .title,
    "历史博文"
  );
  assert_eq!(
    prepared
      .routes
      .iter()
      .find(|route| route.page.kind == "blog-index")
      .unwrap()
      .page
      .title,
    "新闻"
  );
}

#[test]
fn a_missing_translated_tag_view_uses_the_real_blog_home() {
  let root = fixture(false);
  fs::create_dir(root.path().join("zh-blog")).unwrap();
  fs::write(
    root.path().join("zh-blog/index.md"),
    "---\ntags: [Build]\n---\n# 中文文章\n",
  )
  .unwrap();
  let config=parse_config(&json!({"title":"Blog","url":"https://example.com","blog":{"collections":[{"id":"news","group":"news","dir":"blog","routeBase":"/blog/"},{"id":"zh-news","group":"news","locale":"zh","dir":"zh-blog","routeBase":"/zh/blog/"}]},"locales":{"default":"en","items":["en","zh"]}}).to_string(),root.path()).unwrap();
  let prepared = prepare(&config).unwrap();
  let tag = &prepared
    .routes
    .iter()
    .find(|route| route.page.route_path == "/blog/tags/rust/")
    .unwrap()
    .page;
  let alternate = tag
    .alternates
    .iter()
    .find(|alternate| alternate.locale == "zh")
    .unwrap();
  assert!(alternate.missing && alternate.key.is_none());
  assert_eq!(alternate.route_path, "/zh/blog/");
}

#[test]
fn generated_blog_routes_cannot_overwrite_real_document_routes() {
  let root = fixture(false);
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(root.path().join("docs/index.md"), "# Real document\n").unwrap();
  let config=parse_config(&json!({"title":"Blog","url":"https://example.com","docs":{"collections":[{"id":"guide","dir":"docs","routeBase":"/blog/"}]},"blog":{"collections":[{"id":"news","dir":"blog","routeBase":"/blog/"}]}}).to_string(),root.path()).unwrap();
  let error = prepare(&config).unwrap_err();
  assert!(
    error.contains("DOCS_ROUTE_COLLISION") && error.contains("/blog/"),
    "{error}"
  );
}

#[test]
fn empty_view_labels_are_rejected_before_content_generation() {
  let root = fixture(true);
  let error=parse_config(&json!({"title":"Blog","url":"https://example.com","blog":{"collections":[{"id":"news","dir":"blog","routeBase":"/blog/","blogLabels":{"noPosts":" "}}]}}).to_string(),root.path()).err().unwrap();
  assert_eq!(error.code, "DOCS_CONFIG_BLOG_LABELS");
}

#[test]
fn full_list_content_is_opt_in_and_only_applies_to_the_blog_index() {
  let root = fixture(false);
  for mode in ["links", "full"] {
    let config = parse_config(&json!({"title":"Blog","url":"https://example.com","blog":{"collections":[{"id":"news","dir":"blog","routeBase":"/blog/","blogListContent":mode}]}}).to_string(),root.path()).unwrap();
    let prepared = prepare(&config).unwrap();
    for route in &prepared.routes {
      if let Some(view) = &route.page.blog_view {
        assert_eq!(
          view.full_content,
          mode == "full" && view.kind == "blog-index"
        );
      }
    }
    let browser = farmfe_docs_plan::browser_application(&prepared).unwrap();
    assert!(browser.contains("module['listingPost' + index]"));
    assert!(browser.contains("listingPosts"));
    verify(&prepared).unwrap();
  }
  let config = json!({"title":"Blog","url":"https://example.com","docs":{"collections":[{"id":"news","dir":"blog","routeBase":"/docs/","blogListContent":"full"}]}});
  assert_eq!(
    parse_config(&config.to_string(), root.path())
      .err()
      .unwrap()
      .code,
    "DOCS_CONFIG_BLOG_CONTENT"
  );
}
