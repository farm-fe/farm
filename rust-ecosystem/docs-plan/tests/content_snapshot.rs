use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{DocPlan, prepare, verify};
use serde_json::json;
use std::fs;

fn fixture() -> (tempfile::TempDir, DocPlan) {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(root.path().join("docs/index.md"), "# Original\n").unwrap();
  let config = parse_config(
    &json!({"title":"Snapshot","url":"https://example.com","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}}).to_string(),
    root.path(),
  ).unwrap();
  let plan = prepare(&config).unwrap();
  verify(&plan).unwrap();
  (root, plan)
}

#[test]
fn content_snapshot_rejects_new_nested_document() {
  let (root, plan) = fixture();
  fs::create_dir(root.path().join("docs/nested")).unwrap();
  fs::write(root.path().join("docs/nested/new.md"), "# New\n").unwrap();
  let error = verify(&plan).unwrap_err();
  assert!(error.contains("DOCS_SOURCE_SNAPSHOT"), "{error}");
}

#[test]
fn content_snapshot_rejects_previously_empty_directory_becoming_content() {
  let (root, plan) = fixture();
  fs::create_dir(root.path().join("docs/empty")).unwrap();
  verify(&plan).unwrap();
  fs::write(root.path().join("docs/empty/new.mdx"), "# New\n").unwrap();
  let error = verify(&plan).unwrap_err();
  assert!(error.contains("DOCS_SOURCE_SNAPSHOT"), "{error}");
}

#[test]
fn content_snapshot_rejects_removed_or_renamed_document() {
  for rename in [false, true] {
    let (root, plan) = fixture();
    if rename {
      fs::rename(
        root.path().join("docs/index.md"),
        root.path().join("docs/renamed.md"),
      )
      .unwrap();
    } else {
      fs::remove_file(root.path().join("docs/index.md")).unwrap();
    }
    assert!(verify(&plan).is_err());
  }
}

#[test]
fn content_snapshot_keeps_private_and_non_content_files_out_of_discovery() {
  let (root, plan) = fixture();
  for directory in [".farm-docs", ".FARM-DOCS-LOCKS", ".git", "node_modules"] {
    let directory = root.path().join("docs").join(directory);
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("ignored.md"), "# Private\n").unwrap();
  }
  fs::write(root.path().join("docs/helper.tsx"), "export default 1").unwrap();
  verify(&plan).unwrap();
}

#[test]
fn content_snapshot_is_required_in_private_plan() {
  let (_, plan) = fixture();
  let mut value = serde_json::to_value(plan).unwrap();
  value.as_object_mut().unwrap().remove("contentInputs");
  assert!(serde_json::from_value::<DocPlan>(value).is_err());
}

#[test]
fn page_discovery_seals_script_candidates_but_excludes_underscore_helpers() {
  let root = tempfile::tempdir().unwrap();
  let pages = root.path().join("pages");
  fs::create_dir(&pages).unwrap();
  fs::write(
    pages.join("index.tsx"),
    "export default function Home(){return <h1>Home</h1>}",
  )
  .unwrap();
  let config = parse_config(
    &json!({"title":"Pages","url":"https://example.com","pages":{"dir":"pages"}}).to_string(),
    root.path(),
  )
  .unwrap();
  let plan = prepare(&config).unwrap();
  assert!(plan.content_inputs.iter().all(|input| input.scripts));
  fs::write(pages.join("_Helper.tsx"), "export default 1").unwrap();
  verify(&plan).unwrap();
  fs::write(
    pages.join("next.tsx"),
    "export default function Next(){return <h1>Next</h1>}",
  )
  .unwrap();
  let error = verify(&plan).unwrap_err();
  assert!(error.contains("DOCS_SOURCE_SNAPSHOT"), "{error}");
}
