use farmfe_docs_config::parse_config;
use farmfe_docs_plan::prepare;
use serde_json::{Value, json};
use std::{fs, path::Path};

fn config() -> Value {
  json!({"title":"Translations","url":"https://example.com","basePath":"/guide/",
  "locales":{"default":"en","items":["en","zh"]},
  "docs":{"collections":[
    {"id":"en-current","group":"guide","dir":"en-current","locale":"en","version":"current","routeBase":"/docs/"},
    {"id":"zh-current","group":"guide","dir":"zh-current","locale":"zh","version":"current","routeBase":"/zh/docs/"},
    {"id":"en-old","group":"guide","dir":"en-old","locale":"en","version":"previous","routeBase":"/v1/docs/"},
    {"id":"zh-old","group":"guide","dir":"zh-old","locale":"zh","version":"previous","routeBase":"/zh/v1/docs/"}
  ]}})
}
fn content(root: &Path, collection: &str, file: &str, id: &str) {
  fs::create_dir_all(root.join(collection)).unwrap();
  fs::write(
    root.join(collection).join(file),
    format!("---\nid: {id}\n---\n# {id}\n"),
  )
  .unwrap();
}
fn fixture() -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  for collection in ["en-current", "zh-current", "en-old", "zh-old"] {
    content(root.path(), collection, "index.md", "intro");
  }
  root
}
fn plan(root: &Path, raw: &Value) -> Result<Value, String> {
  let config = parse_config(&raw.to_string(), root).map_err(|error| error.to_string())?;
  serde_json::to_value(prepare(&config)?).map_err(|error| error.to_string())
}

#[test]
fn two_locales_and_versions_link_the_same_logical_id_with_fixed_urls() {
  let root = fixture();
  let plan = plan(root.path(), &config()).unwrap();
  let expected = [
    "/guide/docs/",
    "/guide/zh/docs/",
    "/guide/v1/docs/",
    "/guide/zh/v1/docs/",
  ];
  for (index, path) in expected.iter().enumerate() {
    let page = &plan["routes"][index]["page"];
    assert_eq!(page["routePath"], *path);
    let alternates = page["alternates"].as_array().unwrap();
    assert_eq!(alternates.len(), 4);
    assert_eq!(
      alternates
        .iter()
        .map(|page| page["routePath"].as_str().unwrap())
        .collect::<Vec<_>>(),
      expected
    );
    assert!(
      alternates
        .iter()
        .all(|page| page["missing"] == false && page["key"].is_string())
    );
  }
}

#[test]
fn missing_translation_fails_and_recovers_when_the_real_translation_is_added() {
  let root = fixture();
  content(root.path(), "en-current", "modern.md", "modern");
  let error = plan(root.path(), &config()).unwrap_err();
  assert!(
    error.contains("DOCS_TRANSLATION_MISSING")
      && error.contains("modern")
      && error.contains("zh-current"),
    "{error}"
  );
  content(root.path(), "zh-current", "modern.md", "modern");
  let plan = plan(root.path(), &config()).unwrap();
  let page = &plan["routes"][1]["page"];
  assert_eq!(page["id"], "modern");
  let old = page["alternates"]
    .as_array()
    .unwrap()
    .iter()
    .filter(|target| target["version"] == "previous")
    .collect::<Vec<_>>();
  assert_eq!(old.len(), 2);
  assert!(
    old
      .iter()
      .all(|target| target["missing"] == true && target.get("key").is_none())
  );
  assert_eq!(old[0]["routePath"], "/guide/v1/docs/");
}

#[test]
fn exact_collection_exclusion_omits_only_the_declared_orphan_translation() {
  let root = fixture();
  content(root.path(), "zh-current", "orphan.md", "orphan");
  assert!(plan(root.path(), &config()).unwrap_err().contains("DOCS_TRANSLATION_MISSING"));
  let mut raw = config();
  raw["docs"]["collections"][1]["exclude"] = json!(["orphan.md"]);
  let prepared = plan(root.path(), &raw).unwrap();
  let routes = prepared["routes"].as_array().unwrap();
  assert_eq!(routes.len(), 4);
  assert!(routes.iter().all(|route| route["page"]["id"] != "orphan"));
  // The same logical ID in another version is independent of the exclusion.
  for collection in ["en-old", "zh-old"] {
    content(root.path(), collection, "orphan.md", "orphan");
  }
  assert_eq!(plan(root.path(), &raw).unwrap()["routes"].as_array().unwrap().len(), 6);
  let prepared: farmfe_docs_plan::DocPlan = serde_json::from_value(plan(root.path(), &raw).unwrap()).unwrap();
  farmfe_docs_plan::verify(&prepared).unwrap();
  fs::remove_file(root.path().join("zh-current/orphan.md")).unwrap();
  assert!(farmfe_docs_plan::verify(&prepared).unwrap_err().contains("DOCS_SOURCE_SNAPSHOT"));
  assert!(plan(root.path(), &raw).unwrap_err().contains("DOCS_CONFIG_COLLECTION_EXCLUDE"));
}

#[test]
fn links_to_an_excluded_translation_are_still_rejected() {
  let root = fixture();
  content(root.path(), "zh-current", "orphan.md", "orphan");
  fs::write(root.path().join("zh-current/index.md"), "---\nid: intro\n---\n# Intro\n\n[Orphan](./orphan.md)\n").unwrap();
  let mut raw = config();
  raw["docs"]["collections"][1]["exclude"] = json!(["orphan.md"]);
  assert!(plan(root.path(), &raw).unwrap_err().contains("DOCS_LINK_MISSING"));
}

#[cfg(unix)]
#[test]
fn exclusions_cannot_hide_symlinked_inputs_or_directories() {
  use std::os::unix::fs::symlink;
  let root = fixture();
  symlink(root.path().join("en-current/index.md"), root.path().join("zh-current/alias.md")).unwrap();
  let mut raw = config();
  raw["docs"]["collections"][1]["exclude"] = json!(["alias.md"]);
  assert!(plan(root.path(), &raw).unwrap_err().contains("DOCS_CONFIG_COLLECTION_EXCLUDE"));
}

#[test]
fn explicit_fallback_creates_a_target_page_with_true_source_and_same_compiled_body() {
  let root = fixture();
  content(root.path(), "en-current", "modern.md", "modern");
  let mut raw = config();
  raw["docs"]["collections"][1]["fallbackLocale"] = json!("en");
  let plan = plan(root.path(), &raw).unwrap();
  let routes = plan["routes"].as_array().unwrap();
  let index = routes
    .iter()
    .position(|route| {
      route["page"]["collection"] == "zh-current" && route["page"]["id"] == "modern"
    })
    .unwrap();
  let page = &routes[index]["page"];
  assert_eq!(page["routePath"], "/guide/zh/docs/modern/");
  assert_eq!(page["locale"], "zh");
  assert_eq!(page["fallbackSource"]["locale"], "en");
  assert_eq!(page["fallbackSource"]["collection"], "en-current");
  assert_eq!(page["fallbackSource"]["routePath"], "/guide/docs/modern/");
  let original = routes
    .iter()
    .position(|route| {
      route["page"]["collection"] == "en-current" && route["page"]["id"] == "modern"
    })
    .unwrap();
  assert_eq!(
    plan["modules"][index]["code"],
    plan["modules"][original]["code"]
  );
  assert_eq!(
    plan["modules"][index]["sourcePath"],
    plan["modules"][original]["sourcePath"]
  );
  assert_ne!(page["key"], routes[original]["page"]["key"]);
}

#[test]
fn groups_require_unique_locale_version_slots_and_complete_declared_matrix() {
  let root = fixture();
  let mut raw = config();
  raw["docs"]["collections"][1]["locale"] = json!("en");
  assert!(
    plan(root.path(), &raw)
      .unwrap_err()
      .contains("DOCS_CONFIG_GROUP")
  );
  let mut raw = config();
  raw["docs"]["collections"].as_array_mut().unwrap().pop();
  assert!(
    plan(root.path(), &raw)
      .unwrap_err()
      .contains("DOCS_COLLECTION_MATRIX")
  );
}

#[test]
fn fallback_must_name_a_real_other_locale_in_the_same_group_and_version() {
  let root = fixture();
  for locale in ["zh", "fr"] {
    let mut raw = config();
    raw["docs"]["collections"][1]["fallbackLocale"] = json!(locale);
    assert!(
      plan(root.path(), &raw)
        .unwrap_err()
        .contains("DOCS_CONFIG_FALLBACK")
    );
  }
  let mut raw = config();
  raw["docs"]["collections"][0]["fallbackLocale"] = json!("zh");
  raw["docs"]["collections"][1]["fallbackLocale"] = json!("en");
  assert!(
    plan(root.path(), &raw)
      .unwrap_err()
      .contains("DOCS_CONFIG_FALLBACK")
  );
}

#[test]
fn missing_version_targets_choose_an_existing_home_when_no_root_document_exists() {
  let root = fixture();
  for collection in ["en-old", "zh-old"] {
    fs::rename(
      root.path().join(collection).join("index.md"),
      root.path().join(collection).join("intro.md"),
    )
    .unwrap();
  }
  content(root.path(), "en-current", "modern.md", "modern");
  content(root.path(), "zh-current", "modern.md", "modern");
  let plan = plan(root.path(), &config()).unwrap();
  let page = plan["routes"]
    .as_array()
    .unwrap()
    .iter()
    .find(|route| route["page"]["collection"] == "en-current" && route["page"]["id"] == "modern")
    .unwrap();
  let old = page["page"]["alternates"]
    .as_array()
    .unwrap()
    .iter()
    .find(|alternate| alternate["collection"] == "en-old")
    .unwrap();
  assert_eq!(old["routePath"], "/guide/v1/docs/intro/");
  assert_eq!(old["missing"], true);
  assert!(old.get("key").is_none());
}

#[test]
fn an_empty_grouped_version_cannot_publish_unreachable_home_targets() {
  let root = fixture();
  for collection in ["en-old", "zh-old"] {
    fs::remove_file(root.path().join(collection).join("index.md")).unwrap();
  }
  let error = plan(root.path(), &config()).unwrap_err();
  assert!(
    error.contains("DOCS_COLLECTION_EMPTY") && error.contains("en-old"),
    "{error}"
  );
}
