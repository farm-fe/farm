use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{prepare, prepare_with_cache, ContentCache};
use serde_json::{json, Value};
use std::{fs, path::Path};

fn site() -> (tempfile::TempDir, Value) {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(
    root.path().join("docs/index.mdx"),
    "# First\n\n:::note\nOriginal body.\n:::\n",
  )
  .unwrap();
  (
    root,
    json!({"title":"Cache","url":"https://example.com","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}}),
  )
}

fn equal_to_fresh(root: &Path, raw: &Value, cache: &mut ContentCache) -> farmfe_docs_plan::DocPlan {
  let config = parse_config(&raw.to_string(), root).unwrap();
  let cached = prepare_with_cache(&config, cache).unwrap();
  let fresh = prepare(&config).unwrap();
  assert_eq!(
    serde_json::to_value(&cached).unwrap(),
    serde_json::to_value(&fresh).unwrap()
  );
  cached
}

#[test]
fn cold_warm_and_same_length_source_edits_match_the_full_uncached_plan() {
  let (root, raw) = site();
  let mut cache = ContentCache::default();
  let cold = equal_to_fresh(root.path(), &raw, &mut cache);
  let warm = equal_to_fresh(root.path(), &raw, &mut cache);
  assert_eq!(cold.snapshot_id, warm.snapshot_id);
  fs::write(
    root.path().join("docs/index.mdx"),
    "# Other\n\n:::note\nOriginal body.\n:::\n",
  )
  .unwrap();
  let changed = equal_to_fresh(root.path(), &raw, &mut cache);
  assert_ne!(warm.snapshot_id, changed.snapshot_id);
}

#[test]
fn add_remove_and_recreate_routes_keep_complete_sidebars_and_identity() {
  let (root, raw) = site();
  let mut cache = ContentCache::default();
  let original = equal_to_fresh(root.path(), &raw, &mut cache);
  fs::write(root.path().join("docs/added.md"), "# Added\n").unwrap();
  let added = equal_to_fresh(root.path(), &raw, &mut cache);
  assert_eq!(added.routes.len(), 2);
  assert!(serde_json::to_string(&added.sidebars)
    .unwrap()
    .contains("Added"));
  fs::remove_file(root.path().join("docs/added.md")).unwrap();
  let removed = equal_to_fresh(root.path(), &raw, &mut cache);
  assert_eq!(original.snapshot_id, removed.snapshot_id);
  fs::write(root.path().join("docs/added.md"), "# Recreated\n").unwrap();
  let recreated = equal_to_fresh(root.path(), &raw, &mut cache);
  assert_ne!(added.snapshot_id, recreated.snapshot_id);
}

#[test]
fn warm_trees_keep_target_collection_labels_and_fallback_independent() {
  let (root, mut raw) = site();
  fs::create_dir(root.path().join("zh")).unwrap();
  raw["locales"] = json!({"default":"en","items":["en","zh"]});
  raw["docs"]["collections"][0]["group"] = json!("docs");
  raw["docs"]["collections"][0]["locale"] = json!("en");
  raw["docs"]["collections"][0]["admonitionLabels"] = json!({"note":"Source"});
  raw["docs"]["collections"].as_array_mut().unwrap().push(json!({"id":"zh","group":"docs","dir":"zh","locale":"zh","fallbackLocale":"en","routeBase":"/zh/docs/","admonitionLabels":{"note":"目标"}}));
  let mut cache = ContentCache::default();
  let original = equal_to_fresh(root.path(), &raw, &mut cache);
  raw["docs"]["collections"][1]["admonitionLabels"] = json!({"note":"新目标"});
  let changed = equal_to_fresh(root.path(), &raw, &mut cache);
  assert_ne!(original.snapshot_id, changed.snapshot_id);
  for route in &changed.routes {
    let module = changed
      .modules
      .iter()
      .find(|module| module.key == route.page.key)
      .unwrap();
    assert!(module.code.contains(if route.page.collection == "main" {
      "Source"
    } else {
      "新目标"
    }));
  }
}

#[test]
fn warm_source_rechecks_link_targets_and_restores_after_errors() {
  let (root, raw) = site();
  fs::write(
    root.path().join("docs/index.mdx"),
    "# First\n\n[Target](./target.md)\n",
  )
  .unwrap();
  fs::write(root.path().join("docs/target.md"), "# Target\n").unwrap();
  let config = parse_config(&raw.to_string(), root.path()).unwrap();
  let mut cache = ContentCache::default();
  equal_to_fresh(root.path(), &raw, &mut cache);
  fs::remove_file(root.path().join("docs/target.md")).unwrap();
  assert!(prepare_with_cache(&config, &mut cache)
    .err()
    .unwrap()
    .contains("DOCS_LINK_MISSING"));
  fs::write(root.path().join("docs/target.md"), "# Repaired\n").unwrap();
  equal_to_fresh(root.path(), &raw, &mut cache);
  fs::write(root.path().join("docs/index.mdx"), "import {\n").unwrap();
  assert!(prepare_with_cache(&config, &mut cache)
    .err()
    .unwrap()
    .contains("DOCS_CONTENT_COMPILE"));
  fs::write(root.path().join("docs/index.mdx"), "# Restored\n").unwrap();
  equal_to_fresh(root.path(), &raw, &mut cache);
}

#[test]
fn warm_source_rebinds_changed_resources_instead_of_reusing_old_urls() {
  let (root, raw) = site();
  fs::write(
    root.path().join("docs/index.mdx"),
    "# First\n\n![Mark](./mark.svg)\n",
  )
  .unwrap();
  fs::write(root.path().join("docs/mark.svg"), "<svg>old</svg>").unwrap();
  let mut cache = ContentCache::default();
  let original = equal_to_fresh(root.path(), &raw, &mut cache);
  fs::write(root.path().join("docs/mark.svg"), "<svg>new</svg>").unwrap();
  let changed = equal_to_fresh(root.path(), &raw, &mut cache);
  assert_ne!(original.snapshot_id, changed.snapshot_id);
  assert_ne!(
    original.assets[0].output_path,
    changed.assets[0].output_path
  );
}

#[test]
fn changing_collection_format_cannot_reuse_a_markdown_parse_as_mdx() {
  let (root, mut raw) = site();
  fs::remove_file(root.path().join("docs/index.mdx")).unwrap();
  fs::write(root.path().join("docs/index.md"), "import {\n").unwrap();
  raw["docs"]["collections"][0]["format"] = json!("markdown");
  let mut cache = ContentCache::default();
  let original = equal_to_fresh(root.path(), &raw, &mut cache);
  raw["docs"]["collections"][0]["format"] = json!("mdx");
  let config = parse_config(&raw.to_string(), root.path()).unwrap();
  assert!(prepare_with_cache(&config, &mut cache)
    .err()
    .unwrap()
    .contains("DOCS_CONTENT_COMPILE"));
  raw["docs"]["collections"][0]["format"] = json!("markdown");
  let restored = equal_to_fresh(root.path(), &raw, &mut cache);
  assert_eq!(original.snapshot_id, restored.snapshot_id);
}
