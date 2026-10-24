use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{machine_readable, prepare, verify};
use serde_json::json;
use std::fs;

fn fixture(source: &str, ai: serde_json::Value) -> (tempfile::TempDir, farmfe_docs_plan::DocPlan) {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(root.path().join("docs/index.mdx"), source).unwrap();
  let raw = json!({"title":"Fixture","url":"https://example.com","basePath":"/manual/","ai":ai,
    "docs":{"defaultVersion":"v2","collections":[{"id":"v2-en","dir":"docs","routeBase":"/v2/en/docs/","version":"v2","locale":"en"}]}});
  let config = parse_config(&raw.to_string(), root.path()).unwrap();
  let plan = prepare(&config).unwrap();
  (root, plan)
}

#[test]
fn metadata_resources_and_html_discovery_share_the_snapshot() {
  let (root, plan) = fixture(
    "# Setup\n\n:::note\nRead this.\n:::\n\n```rust\nfn main() {}\n```\n",
    json!({}),
  );
  verify(&plan).unwrap();
  let route = &plan.routes[0];
  let resources = machine_readable::resources(&plan).unwrap();
  let summary = machine_readable::summary(&plan).unwrap().unwrap();
  let path = summary["pages"][0]["markdownPath"].as_str().unwrap();
  let bytes = &resources[path];
  let markdown = std::str::from_utf8(bytes).unwrap();
  assert!(markdown.contains("locale: \"en\"") && markdown.contains("version: \"v2\""));
  assert!(markdown.contains("[!NOTE]") && markdown.contains("fn main() {}"));
  assert_eq!(summary["snapshotId"], plan.snapshot_id);
  let manifest: serde_json::Value =
    serde_json::from_slice(&resources["docs-ai-manifest.json"]).unwrap();
  assert_eq!(manifest["snapshotId"], plan.snapshot_id);
  assert_eq!(
    manifest["pages"][0]["sha256"],
    route.page.markdown_sha256.as_deref().unwrap()
  );
  assert!(plan.seo[&route.page.key].html.contains("text/markdown"));
  let published = String::from_utf8(resources.values().flatten().copied().collect()).unwrap();
  assert!(!published.contains(&root.path().to_string_lossy().to_string()));
  assert!(!published.contains("sourcePath") && !published.contains("diagnostics"));
  let mut tampered = plan.clone();
  let original = tampered.modules[0].machine_markdown.as_deref().unwrap();
  tampered.modules[0].machine_markdown = Some(format!("{original}altered").into());
  assert!(verify(&tampered).is_err());
}

#[test]
fn disabled_export_keeps_machine_fields_and_resources_absent() {
  let (_root, plan) = fixture("# Setup\n", json!({"enabled":false}));
  assert!(plan.routes[0].page.markdown_url.is_none());
  assert!(machine_readable::resources(&plan).unwrap().is_empty());
  assert!(machine_readable::summary(&plan).unwrap().is_none());
}

#[test]
fn unknown_component_does_not_publish_an_incomplete_document() {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(
    root.path().join("docs/index.mdx"),
    "# Example\n\n<DynamicCounter />\n",
  )
  .unwrap();
  let raw = json!({"title":"Fixture","url":"https://example.com","ai":{},"docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}});
  let config = parse_config(&raw.to_string(), root.path()).unwrap();
  assert!(
    prepare(&config)
      .unwrap_err()
      .contains("DOCS_AI_UNSUPPORTED")
  );
}
