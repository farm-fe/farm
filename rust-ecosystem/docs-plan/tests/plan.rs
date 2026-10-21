use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{prepare, verify};
use serde_json::json;
use std::path::Path;

#[test]
fn ssg_settings_are_private_snapshot_inputs_with_deterministic_defaults() {
  let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site");
  let mut raw = json!({"title":"Example","url":"https://example.com","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/"}]}});
  let omitted = prepare(&parse_config(&raw.to_string(), &root).unwrap()).unwrap();
  let wire = serde_json::to_value(&omitted).unwrap();
  assert_eq!(wire["ssg"], json!({"concurrency":4,"timeoutMs":30000}));
  assert!(wire["site"].get("ssg").is_none());
  raw["ssg"] = json!({"concurrency":4,"timeoutMs":30000});
  let explicit = prepare(&parse_config(&raw.to_string(), &root).unwrap()).unwrap();
  assert_eq!(omitted.snapshot_id, explicit.snapshot_id);
  raw["ssg"] = json!({"concurrency":2,"timeoutMs":1000});
  let changed = prepare(&parse_config(&raw.to_string(), &root).unwrap()).unwrap();
  assert_ne!(omitted.snapshot_id, changed.snapshot_id);
  assert_eq!(serde_json::to_value(&changed).unwrap()["ssg"], raw["ssg"]);
  verify(&changed).unwrap();
  let mut wire = serde_json::to_value(&changed).unwrap();
  wire["ssg"]["concurrency"] = json!(33);
  let invalid: farmfe_docs_plan::DocPlan = serde_json::from_value(wire.clone()).unwrap();
  assert!(verify(&invalid).unwrap_err().contains("DOCS_PLAN_SSG"));
  wire.as_object_mut().unwrap().remove("ssg");
  let error = serde_json::from_value::<farmfe_docs_plan::DocPlan>(wire).unwrap_err();
  assert!(error.to_string().contains("ssg"));
}

#[test]
fn prepares_real_content_once_into_a_deterministic_private_plan() {
  let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site");
  let site = parse_config(&json!({"title":"Example","url":"https://example.com","basePath":"/guide/","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/"}]}}).to_string(), &root).unwrap();
  let plan = prepare(&site).unwrap();
  assert_eq!(plan.routes.len(), 1);
  assert_eq!(plan.routes[0].page.route_path, "/guide/");
  assert_eq!(plan.routes[0].output_path, "index.html");
  assert_eq!(plan.routes[0].page.title, "Hello Farm Docs");
  assert_eq!(plan.routes[0].page.headings[0].id, "hello-farm");
  assert!(plan.modules[0].code.contains("../src/Counter"));
  assert!(plan.modules[0].code.contains("Rust-compiled"));
  assert_eq!(prepare(&site).unwrap().snapshot_id, plan.snapshot_id);
  verify(&plan).unwrap();
  let mut edited = plan;
  edited.modules[0].code.push_str("\n// changed");
  assert!(verify(&edited).unwrap_err().contains("DOCS_PLAN_HASH"));
}
