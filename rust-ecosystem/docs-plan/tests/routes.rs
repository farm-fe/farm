use farmfe_docs_config::parse_config;
use farmfe_docs_plan::prepare;
use serde_json::json;
use std::{fs, path::Path};

fn source(root: &Path, name: &str, id: &str, slug: &str) {
  let id = serde_json::to_string(id).unwrap();
  let slug = serde_json::to_string(slug).unwrap();
  fs::create_dir_all(root.join("docs")).unwrap();
  fs::write(
    root.join("docs").join(name),
    format!("---\nid: {id}\nslug: {slug}\n---\n# Test\n"),
  )
  .unwrap();
}
fn plan(root: &Path) -> Result<farmfe_docs_plan::DocPlan, String> {
  let config = parse_config(
    &json!({"title":"Routes","url":"https://example.com","basePath":"/guide/",
    "docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}})
    .to_string(),
    root,
  )
  .unwrap();
  prepare(&config)
}

#[test]
fn duplicate_logical_ids_fail_even_with_distinct_urls() {
  let directory = tempfile::tempdir().unwrap();
  source(directory.path(), "a.md", "same", "first");
  source(directory.path(), "b.md", "same", "second");
  let error = plan(directory.path())
    .err()
    .expect("duplicate id must fail");
  assert!(error.contains("DOCS_CONTENT_ID"), "{error}");
  assert!(error.contains("a.md") && error.contains("b.md"), "{error}");
}

#[test]
fn case_equivalent_urls_fail_with_both_sources() {
  let directory = tempfile::tempdir().unwrap();
  source(directory.path(), "a.md", "a", "Intro");
  source(directory.path(), "b.md", "b", "intro");
  let error = plan(directory.path())
    .err()
    .expect("case collision must fail");
  assert!(error.contains("DOCS_ROUTE_COLLISION"), "{error}");
  assert!(error.contains("a.md") && error.contains("b.md"), "{error}");
}

#[test]
fn unicode_normalization_collisions_fail_without_rewriting_public_urls() {
  let directory = tempfile::tempdir().unwrap();
  source(directory.path(), "a.md", "a", "é");
  source(directory.path(), "b.md", "b", "e\u{301}");
  let error = plan(directory.path())
    .err()
    .expect("NFC collision must fail");
  assert!(error.contains("DOCS_ROUTE_COLLISION"), "{error}");
  assert!(error.contains("a.md") && error.contains("b.md"), "{error}");
}

#[test]
fn logical_id_comparison_rejects_unicode_equivalent_duplicates() {
  let directory = tempfile::tempdir().unwrap();
  source(directory.path(), "a.md", "é", "first");
  source(directory.path(), "b.md", "e\u{301}", "second");
  let error = plan(directory.path())
    .err()
    .expect("NFC id collision must fail");
  assert!(error.contains("DOCS_CONTENT_ID"), "{error}");
}

#[test]
fn portable_outputs_reject_reserved_names_and_ambiguous_segments() {
  for slug in [
    "CON", "con.txt", "NUL.bin", "LPT1", "x.", "a//b", "a|b", "a<b",
  ] {
    let directory = tempfile::tempdir().unwrap();
    source(directory.path(), "a.md", "a", slug);
    let error = plan(directory.path())
      .err()
      .unwrap_or_else(|| panic!("unsafe output accepted: {slug}"));
    assert!(error.contains("DOCS_OUTPUT_PATH"), "{slug}: {error}");
    assert!(error.contains("a.md"), "{slug}: {error}");
  }
}

#[test]
fn explicit_id_keeps_key_when_source_file_is_renamed() {
  let directory = tempfile::tempdir().unwrap();
  source(directory.path(), "before.md", "stable", "hello/世界");
  let before = plan(directory.path()).unwrap();
  fs::rename(
    directory.path().join("docs/before.md"),
    directory.path().join("docs/after.md"),
  )
  .unwrap();
  let after = plan(directory.path()).unwrap();
  assert_eq!(before.routes[0].page.key, after.routes[0].page.key);
  assert_ne!(before.snapshot_id, after.snapshot_id);
  assert_eq!(after.routes[0].page.route_path, "/guide/docs/hello/世界/");
  assert_eq!(after.routes[0].output_path, "docs/hello/世界/index.html");
}

#[test]
fn collisions_are_global_but_logical_ids_are_scoped_to_collections() {
  let directory = tempfile::tempdir().unwrap();
  source(directory.path(), "a.md", "shared", "entry");
  fs::rename(
    directory.path().join("docs"),
    directory.path().join("docs-en"),
  )
  .unwrap();
  source(directory.path(), "a.md", "shared", "entry");
  fs::rename(
    directory.path().join("docs"),
    directory.path().join("docs-zh"),
  )
  .unwrap();
  let mut input = json!({"title":"Collections","url":"https://example.com","basePath":"/guide/",
  "locales":{"default":"en","items":["en","zh"]},
  "docs":{"collections":[
      {"id":"en","dir":"docs-en","locale":"en","version":"current","routeBase":"/docs/"},
      {"id":"zh","dir":"docs-zh","locale":"zh","version":"previous","routeBase":"/docs/"}
  ]}});
  let config = parse_config(&input.to_string(), directory.path()).unwrap();
  let error = prepare(&config)
    .err()
    .expect("cross-collection URL collision must fail");
  assert!(
    error.contains("DOCS_ROUTE_COLLISION")
      && error.contains("docs-en/a.md")
      && error.contains("docs-zh/a.md"),
    "{error}"
  );
  input["docs"]["collections"][1]["routeBase"] = json!("/zh/docs/");
  let config = parse_config(&input.to_string(), directory.path()).unwrap();
  let prepared = prepare(&config).unwrap();
  assert_eq!(prepared.routes[0].page.id, prepared.routes[1].page.id);
  assert_ne!(prepared.routes[0].page.key, prepared.routes[1].page.key);
  assert_eq!(prepared.routes[1].page.version.as_deref(), Some("previous"));
  assert_eq!(prepared.routes[0].page.route_path, "/guide/docs/entry/");
  assert_eq!(prepared.routes[1].page.route_path, "/guide/zh/docs/entry/");
}

#[test]
fn encoded_and_literal_slug_escapes_report_the_source() {
  for slug in [
    "../outside",
    "a/../outside",
    "%2e%2e/outside",
    "a%2fb",
    "C:/outside",
    "//example.com",
    "..\\outside",
  ] {
    let directory = tempfile::tempdir().unwrap();
    source(directory.path(), "a.md", "a", slug);
    let error = plan(directory.path())
      .err()
      .unwrap_or_else(|| panic!("unsafe slug accepted: {slug}"));
    assert!(
      error.contains("DOCS_ROUTE_PATH") && error.contains("a.md"),
      "{slug}: {error}"
    );
  }
}

#[test]
fn unique_authored_unicode_spelling_is_preserved() {
  let directory = tempfile::tempdir().unwrap();
  let authored = "e\u{301}";
  source(directory.path(), "a.md", "a", authored);
  let prepared = plan(directory.path()).unwrap();
  assert_eq!(
    prepared.routes[0].page.route_path,
    format!("/guide/docs/{authored}/")
  );
  assert_eq!(
    prepared.routes[0].output_path,
    format!("docs/{authored}/index.html")
  );
}
