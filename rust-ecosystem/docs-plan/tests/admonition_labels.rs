use farmfe_docs_config::parse_config;
use farmfe_docs_plan::prepare;
use serde_json::json;
use std::fs;

#[test]
fn omitted_and_explicit_kind_defaults_keep_the_same_snapshot() {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(
    root.path().join("docs/index.md"),
    "# Defaults\n\n:::note\nOriginal body.\n:::\n",
  )
  .unwrap();
  let mut raw = json!({"title":"Defaults","url":"https://example.com","docs":{"collections":[
    {"id":"main","dir":"docs","routeBase":"/docs/"}
  ]}});
  let omitted = prepare(&parse_config(&raw.to_string(), root.path()).unwrap()).unwrap();
  raw["docs"]["collections"][0]["admonitionLabels"] = json!({"note":"note"});
  let explicit = prepare(&parse_config(&raw.to_string(), root.path()).unwrap()).unwrap();
  assert_eq!(omitted.snapshot_id, explicit.snapshot_id);
}

#[test]
fn final_collection_labels_are_private_and_change_fallback_module_identity() {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("en")).unwrap();
  fs::create_dir(root.path().join("zh")).unwrap();
  fs::write(
    root.path().join("en/index.md"),
    "# Fallback\n\n:::note\nOriginal body.\n:::\n",
  )
  .unwrap();
  let mut raw = json!({"title":"Labels","url":"https://example.com",
    "locales":{"default":"en","items":["en","zh"]},"docs":{"collections":[
    {"id":"en","group":"docs","dir":"en","locale":"en","routeBase":"/docs/","admonitionLabels":{"note":"Source label"}},
    {"id":"zh","group":"docs","dir":"zh","locale":"zh","fallbackLocale":"en","routeBase":"/zh/docs/","admonitionLabels":{"note":"目标标签"}}
  ]}});
  let plan = prepare(&parse_config(&raw.to_string(), root.path()).unwrap()).unwrap();
  for (collection, expected, forbidden) in [
    ("en", "Source label", "目标标签"),
    ("zh", "目标标签", "Source label"),
  ] {
    let page = &plan
      .routes
      .iter()
      .find(|route| route.page.collection == collection)
      .unwrap()
      .page;
    let module = plan
      .modules
      .iter()
      .find(|module| module.key == page.key)
      .unwrap();
    assert!(module.code.contains(expected));
    assert!(!module.code.contains(forbidden));
    assert!(module.code.contains("Original body."));
  }
  assert!(
    !serde_json::to_string(&plan.site)
      .unwrap()
      .contains("admonitionLabels")
  );
  raw["docs"]["collections"][1]["admonitionLabels"]["note"] = json!("新标签");
  let changed = prepare(&parse_config(&raw.to_string(), root.path()).unwrap()).unwrap();
  assert_ne!(plan.snapshot_id, changed.snapshot_id);
  assert_eq!(
    serde_json::to_value(&plan.inputs).unwrap(),
    serde_json::to_value(&changed.inputs).unwrap(),
    "author file bytes were unchanged"
  );
  let original_source = plan
    .routes
    .iter()
    .find(|route| route.page.collection == "en")
    .unwrap();
  assert_eq!(
    plan
      .modules
      .iter()
      .find(|module| module.key == original_source.page.key)
      .unwrap()
      .code,
    changed
      .modules
      .iter()
      .find(|module| module.key == original_source.page.key)
      .unwrap()
      .code
  );
}
