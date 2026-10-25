use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{prepare, verify};
use serde_json::json;
use std::{fs, path::Path};

fn fixture() -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(root.path().join("docs/index.md"), "# Assets\n\n![Diagram](./diagram.svg?theme=dark#mark)\n\n[Download](./guide.pdf?download=1#page=1)\n").unwrap();
  fs::write(
    root.path().join("docs/diagram.svg"),
    "<svg xmlns=\"http://www.w3.org/2000/svg\"><path id=\"mark\" d=\"M0 0h1\"/></svg>\n",
  )
  .unwrap();
  fs::write(
    root.path().join("docs/guide.pdf"),
    b"%PDF-1.7\nfixture bytes\n",
  )
  .unwrap();
  root
}
fn plan(root: &Path) -> Result<farmfe_docs_plan::DocPlan, String> {
  let config = parse_config(&json!({"title":"Assets","url":"https://example.com","basePath":"/guide/","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}}).to_string(), root).map_err(|error|error.to_string())?;
  prepare(&config)
}

#[test]
fn relative_image_and_download_are_real_inputs_and_changed_bytes_invalidate_snapshot() {
  let root = fixture();
  let plan = plan(root.path()).unwrap();
  for name in ["diagram.svg", "guide.pdf"] {
    assert!(
      plan.inputs.iter().any(|input| input.path.ends_with(name)),
      "missing asset input {name}"
    );
  }
  assert!(
    !plan
      .diagnostics
      .iter()
      .any(|diagnostic| diagnostic.contains("DOCS_RESOURCE_PENDING"))
  );
  verify(&plan).unwrap();
  fs::write(root.path().join("docs/diagram.svg"), "<svg>changed</svg>").unwrap();
  let error = verify(&plan).unwrap_err();
  assert!(
    error.contains("DOCS_SNAPSHOT_CHANGED") && error.contains("diagram.svg"),
    "{error}"
  );
}

#[test]
fn missing_local_image_fails_with_original_reference_and_recovers() {
  let root = fixture();
  fs::remove_file(root.path().join("docs/diagram.svg")).unwrap();
  let error = plan(root.path()).unwrap_err();
  assert!(
    error.contains("DOCS_ASSET_MISSING")
      && error.contains("index.md:3:1")
      && error.contains("diagram.svg"),
    "{error}"
  );
  fs::write(root.path().join("docs/diagram.svg"), "<svg/>").unwrap();
  assert!(plan(root.path()).is_ok());
}

#[test]
fn resources_outside_collection_but_inside_site_keep_query_and_fragment() {
  let root = fixture();
  fs::create_dir(root.path().join("shared")).unwrap();
  fs::write(root.path().join("shared/手册.pdf"), b"download bytes").unwrap();
  fs::write(
    root.path().join("docs/index.md"),
    "# Files\n\n[Manual](../shared/%E6%89%8B%E5%86%8C.pdf?download=1#page=2)\n",
  )
  .unwrap();
  let result = plan(root.path()).unwrap();
  assert_eq!(result.assets.len(), 1);
  assert!(result.modules[0].code.contains("?download=1#page=2"));
  assert!(
    result.modules[0]
      .code
      .contains(&format!("docs:asset:{}", result.assets[0].key))
  );
}

#[test]
fn unsafe_paths_missing_downloads_and_directory_images_do_not_become_valid_assets() {
  for (target, code) in [
    ("../../escape.svg", "DOCS_ASSET_PATH"),
    ("%2e%2e/escape.svg", "DOCS_ASSET_PATH"),
    ("./missing.pdf", "DOCS_ASSET_MISSING"),
    ("./dir.svg", "DOCS_ASSET_PATH"),
  ] {
    let root = fixture();
    fs::create_dir(root.path().join("docs/dir.svg")).unwrap();
    let body = if target.ends_with("pdf") {
      format!("# Files\n\n[File]({target})\n")
    } else {
      format!("# Files\n\n![Image]({target})\n")
    };
    fs::write(root.path().join("docs/index.md"), body).unwrap();
    let error = plan(root.path()).unwrap_err();
    assert!(
      error.contains(code) && error.contains("index.md:3:1"),
      "{error}"
    );
  }
}
