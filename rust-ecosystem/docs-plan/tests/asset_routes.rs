use farmfe_docs_config::parse_config;
use farmfe_docs_plan::prepare;
use serde_json::json;
use std::fs;

#[test]
fn actual_public_page_with_file_extension_is_not_a_download_asset() {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(
    root.path().join("docs/index.md"),
    "# Home\n\n[Page](/guide/docs/file.pdf)\n",
  )
  .unwrap();
  fs::write(
    root.path().join("docs/page.md"),
    "---\nslug: file.pdf\n---\n# Actual page\n",
  )
  .unwrap();
  let config = parse_config(&json!({"title":"Routes","url":"https://example.com","basePath":"/guide/","trailingSlash":"never","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}}).to_string(), root.path()).unwrap();
  let plan = prepare(&config).unwrap();
  assert!(plan.assets.is_empty());
  assert!(
    !plan
      .diagnostics
      .iter()
      .any(|value| value.contains("DOCS_RESOURCE_PENDING")),
    "{:?}",
    plan.diagnostics
  );
  assert!(plan.modules[0].code.contains("/guide/docs/file.pdf"));
}
