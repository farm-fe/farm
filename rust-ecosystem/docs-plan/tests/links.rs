use farmfe_docs_config::parse_config;
use farmfe_docs_plan::prepare;
use serde_json::json;
use std::fs;

fn fixture(target: &str, extra: bool) -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(
    root.path().join("docs/index.md"),
    format!("# Intro\n\n[Next]({target})\n"),
  )
  .unwrap();
  if extra {
    fs::write(root.path().join("docs/next.md"), "# Next\n\n## Present\n").unwrap();
  }
  root
}
fn plan(root: &std::path::Path) -> Result<farmfe_docs_plan::DocPlan, String> {
  let config=parse_config(&json!({"title":"Links","url":"https://example.com","basePath":"/guide/","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}}).to_string(),root).unwrap();
  prepare(&config)
}

#[test]
fn public_source_alias_uses_real_slug_and_keeps_query_and_anchor() {
  for target in [
    "/docs/next.md?from=source#present",
    "/guide/docs/next.md?from=source#present",
  ] {
    let root = fixture(target, true);
    fs::write(
      root.path().join("docs/next.md"),
      "---\nid: stable-id\nslug: renamed\n---\n# Next\n\n## Present\n",
    )
    .unwrap();
    let prepared = plan(root.path()).unwrap();
    let index = prepared
      .modules
      .iter()
      .find(|module| module.source_path.ends_with("/index.md"))
      .unwrap();
    assert!(
      index
        .code
        .contains("/guide/docs/renamed/?from=source#present")
    );
    assert!(
      prepared
        .routes
        .iter()
        .all(|route| !route.page.route_path.ends_with("next.md/"))
    );
  }
}

#[test]
fn explicit_public_route_takes_precedence_over_a_source_alias() {
  let root = fixture("/docs/next.md#actual", true);
  fs::write(
    root.path().join("docs/next.md"),
    "---\nslug: renamed\n---\n# Next\n",
  )
  .unwrap();
  fs::write(
    root.path().join("docs/actual.md"),
    "---\nslug: next.md\n---\n# Actual\n",
  )
  .unwrap();
  let prepared = plan(root.path()).unwrap();
  let index = prepared
    .modules
    .iter()
    .find(|module| module.source_path.ends_with("/index.md"))
    .unwrap();
  assert!(index.code.contains("/guide/docs/next.md/#actual"));
}

#[test]
fn colliding_source_aliases_report_ambiguity_instead_of_selecting_an_owner() {
  let root = fixture("/docs/next.md", true);
  fs::create_dir(root.path().join("other")).unwrap();
  fs::write(
    root.path().join("docs/next.md"),
    "---\nslug: one\n---\n# One\n",
  )
  .unwrap();
  fs::write(
    root.path().join("other/next.md"),
    "---\nslug: two\n---\n# Two\n",
  )
  .unwrap();
  let config = parse_config(&json!({"title":"Links","url":"https://example.com","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"},{"id":"other","dir":"other","routeBase":"/docs/"}]}}).to_string(), root.path()).unwrap();
  let error = prepare(&config).unwrap_err();
  assert!(
    error.contains("DOCS_LINK_AMBIGUOUS")
      && error.contains("index.md:3:1")
      && error.contains("/docs/one/")
      && error.contains("/docs/two/"),
    "{error}"
  );
}

#[test]
fn fallback_source_alias_is_indexed_under_the_target_collection_namespace() {
  let root = fixture("/docs/next.md#next", true);
  fs::write(
    root.path().join("docs/next.md"),
    "---\nslug: renamed\n---\n# Next\n",
  )
  .unwrap();
  fs::create_dir(root.path().join("zh")).unwrap();
  fs::write(
    root.path().join("zh/index.md"),
    "# 首页\n\n[Next](/zh/docs/next.md#next)\n",
  )
  .unwrap();
  let config = parse_config(&json!({"title":"Links","url":"https://example.com","basePath":"/guide/","locales":{"default":"en","items":["en","zh"]},"docs":{"collections":[{"id":"main","group":"guide","dir":"docs","routeBase":"/docs/"},{"id":"zh","group":"guide","dir":"zh","locale":"zh","routeBase":"/zh/docs/","fallbackLocale":"en"}]}}).to_string(), root.path()).unwrap();
  let prepared = prepare(&config).unwrap();
  let index = prepared
    .modules
    .iter()
    .find(|module| module.source_path.ends_with("/zh/index.md"))
    .unwrap();
  assert!(index.code.contains("/guide/zh/docs/renamed/#next"));
}

#[test]
fn local_translation_can_link_to_a_real_fallback_by_relative_source_filename() {
  for target in ["./next.md?from=local#present", "./next?from=local#present"] {
    let root = fixture("./next.md#present", true);
    fs::write(
      root.path().join("docs/next.md"),
      "---\nslug: renamed\n---\n# Next\n\n## Present\n",
    )
    .unwrap();
    fs::create_dir(root.path().join("zh")).unwrap();
    fs::write(
      root.path().join("zh/index.md"),
      format!("# 首页\n\n[Next]({target})\n"),
    )
    .unwrap();
    let raw = json!({"title":"Links","url":"https://example.com","basePath":"/guide/","locales":{"default":"en","items":["en","zh"]},"docs":{"collections":[{"id":"main","group":"guide","dir":"docs","routeBase":"/docs/"},{"id":"zh","group":"guide","dir":"zh","locale":"zh","routeBase":"/zh/docs/","fallbackLocale":"en"}]}});
    let config = parse_config(&raw.to_string(), root.path()).unwrap();
    let prepared = prepare(&config).unwrap();
    let index = prepared
      .modules
      .iter()
      .find(|module| module.source_path.ends_with("/zh/index.md"))
      .unwrap();
    assert!(
      index
        .code
        .contains("/guide/zh/docs/renamed/?from=local#present"),
      "{}",
      index.code
    );
    fs::write(
      root.path().join("zh/index.md"),
      "# 首页\n\n[Next](./next.md#absent)\n",
    )
    .unwrap();
    assert!(prepare(&config).unwrap_err().contains("DOCS_LINK_ANCHOR"));
    fs::write(
      root.path().join("docs/next.mdx"),
      "---\nid: alternate\nslug: alternate\n---\n# Alternate\n",
    )
    .unwrap();
    fs::write(
      root.path().join("zh/index.md"),
      "# 首页\n\n[Ambiguous](./next)\n",
    )
    .unwrap();
    assert!(
      prepare(&config)
        .unwrap_err()
        .contains("DOCS_LINK_AMBIGUOUS")
    );
  }
}

#[test]
fn source_aliases_still_validate_the_actual_target_anchor() {
  let root = fixture("/docs/next.md#absent", true);
  let error = plan(root.path()).unwrap_err();
  assert!(
    error.contains("DOCS_LINK_ANCHOR") && error.contains("index.md:3:1"),
    "{error}"
  );
}

#[test]
fn mount_root_without_slash_is_not_prefixed_twice() {
  let root = fixture("/guide?from=nav#intro", false);
  let config = parse_config(&json!({"title":"Links","url":"https://example.com","basePath":"/guide/","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/"}]}}).to_string(), root.path()).unwrap();
  let prepared = prepare(&config).unwrap();
  assert!(prepared.modules[0].code.contains("/guide/?from=nav#intro"));
}

#[test]
fn similar_mount_prefix_still_means_a_site_relative_page() {
  let root = fixture("/guidebook", false);
  fs::write(root.path().join("docs/guidebook.md"), "# Guidebook\n").unwrap();
  let config = parse_config(&json!({"title":"Links","url":"https://example.com","basePath":"/guide/","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/"}]}}).to_string(), root.path()).unwrap();
  let prepared = prepare(&config).unwrap();
  let index = prepared
    .modules
    .iter()
    .find(|module| module.source_path.ends_with("/index.md"))
    .unwrap();
  assert!(index.code.contains("/guide/guidebook/"));
}

#[test]
fn missing_document_reports_original_location_and_recovers_after_creation() {
  let root = fixture("./next.md", false);
  let error = plan(root.path()).err().expect("missing document must fail");
  assert!(
    error.contains("DOCS_LINK_MISSING")
      && error.contains("index.md:3:1")
      && error.contains("./next.md"),
    "{error}"
  );
  fs::write(root.path().join("docs/next.md"), "# Next\n").unwrap();
  assert!(
    plan(root.path()).unwrap().modules[0]
      .code
      .contains("/guide/docs/next/")
  );
}
#[test]
fn missing_anchor_reports_source_and_target() {
  let root = fixture("./next.md#absent", true);
  let error = plan(root.path()).err().expect("missing anchor must fail");
  assert!(
    error.contains("DOCS_LINK_ANCHOR")
      && error.contains("index.md:3:1")
      && error.contains("absent"),
    "{error}"
  );
}
#[test]
fn encoded_escape_cannot_leave_the_collection() {
  let root = fixture("%2e%2e/outside.md", false);
  let error = plan(root.path()).err().expect("escape must fail");
  assert!(
    error.contains("DOCS_LINK_ESCAPE") && error.contains("index.md:3:1"),
    "{error}"
  );
}
#[test]
fn relative_file_link_cannot_silently_switch_language_or_version() {
  let root = fixture("../other/next.md", false);
  fs::create_dir(root.path().join("other")).unwrap();
  fs::write(root.path().join("other/next.md"), "# Next\n").unwrap();
  let config=parse_config(&json!({"title":"Links","url":"https://example.com","locales":{"default":"en","items":["en","zh"]},"docs":{"collections":[{"id":"en","dir":"docs","routeBase":"/docs/"},{"id":"zh-old","dir":"other","locale":"zh","version":"old","routeBase":"/zh/old/"}]}}).to_string(),root.path()).unwrap();
  let error = prepare(&config)
    .err()
    .expect("implicit cross-context file link must fail");
  assert!(
    error.contains("DOCS_LINK_CONTEXT") && error.contains("index.md:3:1"),
    "{error}"
  );
}

#[test]
fn query_colons_do_not_become_a_fake_scheme_and_fragments_are_validated() {
  let root = fixture("./next.md?return=https://example.com#present", true);
  let prepared = plan(root.path()).unwrap();
  assert!(
    prepared.modules[0]
      .code
      .contains("/guide/docs/next/?return=https://example.com#present")
  );
}
#[test]
fn root_relative_encoded_traversal_is_rejected() {
  let root = fixture("/%2e%2e/outside.md", false);
  let error = plan(root.path()).err().expect("root traversal must fail");
  assert!(
    error.contains("DOCS_LINK_ESCAPE") && error.contains("index.md:3:1"),
    "{error}"
  );
}
#[test]
fn unicode_explicit_and_duplicate_headings_share_the_body_anchor_index() {
  let root = fixture("./next.md#%E9%85%8D%E7%BD%AE", true);
  fs::write(
    root.path().join("docs/next.md"),
    "# Next\n\n## 配置\n\n## Repeat\n\n## Repeat\n\n## Custom {#exact}\n",
  )
  .unwrap();
  let prepared = plan(root.path()).unwrap();
  assert!(
    prepared.modules[0]
      .code
      .contains("/guide/docs/next/#%E9%85%8D%E7%BD%AE")
  );
  assert!(prepared.modules[1].code.contains("repeat-1"));
  assert!(prepared.modules[1].code.contains("exact"));
}

#[test]
fn root_public_asset_requires_a_declared_static_file() {
  let root = fixture("/manual.pdf", false);
  let error = plan(root.path()).unwrap_err();
  assert!(
    error.contains("DOCS_STATIC_MISSING") && error.contains("manual.pdf"),
    "{error}"
  );
}

#[test]
fn non_heading_html_ids_are_valid_anchor_targets() {
  let root = fixture("./next.md#raw-anchor", true);
  fs::write(
    root.path().join("docs/next.md"),
    "# Next\n\n<span id=\"raw-anchor\">Target</span>\n",
  )
  .unwrap();
  assert!(
    plan(root.path()).unwrap().modules[0]
      .code
      .contains("/guide/docs/next/#raw-anchor")
  );
}

#[test]
fn shared_source_directories_resolve_in_the_correct_collection_context() {
  let root = fixture("./next.md", true);
  let config=parse_config(&json!({"title":"Shared","url":"https://example.com","locales":{"default":"en","items":["en","zh"]},"docs":{"collections":[{"id":"en","dir":"docs","routeBase":"/docs/"},{"id":"zh","dir":"docs","locale":"zh","routeBase":"/zh/docs/"}]}}).to_string(),root.path()).unwrap();
  let prepared = prepare(&config).unwrap();
  assert!(prepared.modules[0].code.contains("/docs/next/"));
  assert!(!prepared.modules[0].code.contains("/zh/docs/next/"));
  assert!(prepared.modules[2].code.contains("/zh/docs/next/"));
}

#[test]
fn one_physical_file_can_have_explicit_different_content_formats() {
  let root = fixture("", false);
  fs::write(root.path().join("docs/index.md"), "# Shared\n\n{1 + 1}\n").unwrap();
  let config=parse_config(&json!({"title":"Formats","url":"https://example.com","docs":{"collections":[{"id":"markdown","dir":"docs","routeBase":"/markdown/","format":"markdown"},{"id":"mdx","dir":"docs","routeBase":"/mdx/","format":"mdx"}]}}).to_string(),root.path()).unwrap();
  let prepared = prepare(&config).unwrap();
  assert_ne!(prepared.modules[0].code, prepared.modules[1].code);
  assert!(prepared.modules[0].code.contains("\"{1 + 1}\""));
}
