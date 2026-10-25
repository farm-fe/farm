use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{prepare, verify, DocPlan};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

fn fixture() -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(root.path().join("docs/index.md"), "# Document\n").unwrap();
  root
}
fn config() -> Value {
  json!({"title":"Docs & 中文","url":"https://example.com","basePath":"/guide/",
    "locales":{"default":"zh","items":["zh","en"]},
    "docs":{"collections":[{"id":"main","dir":"docs","locale":"zh","routeBase":"/docs/"}]}})
}
fn plan(root: &Path, value: Value) -> DocPlan {
  prepare(&parse_config(&value.to_string(), root).unwrap()).unwrap()
}

#[test]
fn every_declared_locale_has_a_separate_auxiliary_output_and_real_home() {
  let root = fixture();
  let raw = json!({"title":"Localized","url":"https://example.com","basePath":"/guide/",
  "locales":{"default":"en","items":["en","zh"]},
  "docs":{"collections":[
    {"id":"en","dir":"docs","locale":"en","routeBase":"/docs/"},
    {"id":"zh","dir":"docs","locale":"zh","routeBase":"/zh/docs/"}
  ]}});
  let prepared = plan(root.path(), raw);
  let value = serde_json::to_value(&prepared).unwrap();
  let auxiliary = value["notFound"]
    .as_array()
    .expect("NotFound must be a Rust-planned list for all declared locales");
  assert_eq!(auxiliary.len(), 2);
  for (locale, output, mount, home) in [
    ("en", "404.html", "/guide/", "/guide/docs/"),
    ("zh", "zh/404.html", "/guide/zh/", "/guide/zh/docs/"),
  ] {
    let page = auxiliary
      .iter()
      .find(|page| page["locale"] == locale)
      .unwrap();
    assert_eq!(page["outputPath"], output);
    assert_eq!(page["routeBase"], mount);
    assert_eq!(page["home"], home);
    assert!(page.get("key").is_none());
  }
  assert_eq!(prepared.routes.len(), 2);
  assert_eq!(prepared.modules.len(), 2);
  assert_eq!(prepared.seo.len(), 2);
}

#[test]
fn physical_not_found_is_a_required_deterministic_separate_rust_plan() {
  let root = fixture();
  let prepared = plan(root.path(), config());
  let value = serde_json::to_value(&prepared).unwrap();
  let auxiliary = value["notFound"].as_array().unwrap();
  assert_eq!(auxiliary.len(), 2);
  assert_eq!(auxiliary[0]["locale"], "zh");
  assert_eq!(auxiliary[0]["outputPath"], "404.html");
  assert_eq!(auxiliary[1]["outputPath"], "en/404.html");
  assert_eq!(auxiliary[1]["home"], "/guide/docs/");
  assert_eq!(auxiliary[0]["title"], "Page not found | Docs & 中文");
  assert_eq!(
    auxiliary[0]["headHtml"],
    "<meta name=\"robots\" content=\"noindex,follow\">"
  );
  assert_eq!(prepared.routes.len(), 1);
  assert_eq!(prepared.modules.len(), 1);
  assert_eq!(prepared.seo.len(), 1);
  assert!(prepared
    .routes
    .iter()
    .all(|route| route.output_path != "404.html"));
  assert_eq!(
    serde_json::to_value(plan(root.path(), config())).unwrap(),
    value
  );
  verify(&prepared).unwrap();
}

#[test]
fn missing_not_found_cannot_deserialize_as_a_current_private_plan() {
  let root = fixture();
  let mut value = serde_json::to_value(plan(root.path(), config())).unwrap();
  value.as_object_mut().unwrap().remove("notFound");
  assert!(serde_json::from_value::<DocPlan>(value).is_err());
}

#[test]
fn forged_not_found_is_rejected_even_when_identity_is_recomputed() {
  let root = fixture();
  let prepared = plan(root.path(), config());
  let mut value = serde_json::to_value(&prepared).unwrap();
  value["notFound"][0]["title"] = json!("Forged");
  let mut changed: DocPlan = serde_json::from_value(value).expect("current NotFound contract");
  changed.snapshot_id.clear();
  changed.snapshot_id = format!(
    "{:x}",
    Sha256::digest(serde_json::to_vec(&changed).unwrap())
  );
  let error = verify(&changed).unwrap_err();
  assert!(error.contains("DOCS_PLAN_NOT_FOUND"), "{error}");
}

fn rehash(plan: &mut DocPlan) {
  plan.snapshot_id.clear();
  plan.snapshot_id = format!("{:x}", Sha256::digest(serde_json::to_vec(&plan).unwrap()));
}

#[test]
fn all_auxiliary_fields_membership_order_and_old_schema_are_revalidated() {
  let root = fixture();
  let original = plan(root.path(), config());
  for field in [
    "locale",
    "routeBase",
    "outputPath",
    "home",
    "heading",
    "title",
    "description",
    "homeLabel",
    "headHtml",
  ] {
    let mut value = serde_json::to_value(&original).unwrap();
    value["notFound"][0][field] = json!("forged");
    let mut changed: DocPlan = serde_json::from_value(value).unwrap();
    rehash(&mut changed);
    assert!(
      verify(&changed)
        .unwrap_err()
        .contains("DOCS_PLAN_NOT_FOUND"),
      "{field}"
    );
  }
  for kind in ["missing", "extra", "order", "empty"] {
    let mut changed = original.clone();
    match kind {
      "missing" => {
        changed.not_found.pop();
      }
      "extra" => changed.not_found.push(changed.not_found[0].clone()),
      "order" => changed.not_found.reverse(),
      _ => changed.not_found.clear(),
    }
    rehash(&mut changed);
    assert!(
      verify(&changed)
        .unwrap_err()
        .contains("DOCS_PLAN_NOT_FOUND"),
      "{kind}"
    );
  }
  let mut old = original.clone();
  old.schema_version = 1;
  rehash(&mut old);
  assert!(verify(&old).unwrap_err().contains("DOCS_PLAN_HASH"));
  let mut old = serde_json::to_value(&original).unwrap();
  old["notFound"] = old["notFound"][0].clone();
  assert!(serde_json::from_value::<DocPlan>(old).is_err());
}

#[test]
fn unicode_mount_three_locales_and_explicit_labels_are_deterministic() {
  let root = fixture();
  let mut raw = config();
  raw["basePath"] = json!("/指南/");
  raw["locales"]["items"] = json!(["fr", "zh", "en"]);
  raw["notFound"] = json!({"labels":{"zh":{"title":"未找到页面","description":"页面不存在","homeLabel":"回到首页"}}});
  let prepared = plan(root.path(), raw);
  assert_eq!(
    prepared
      .not_found
      .iter()
      .map(|page| page.locale.as_str())
      .collect::<Vec<_>>(),
    ["zh", "en", "fr"]
  );
  assert_eq!(prepared.not_found[0].route_base, "/指南/");
  assert_eq!(prepared.not_found[1].route_base, "/指南/en/");
  assert_eq!(prepared.not_found[2].output_path, "fr/404.html");
  assert_eq!(prepared.not_found[0].heading, "未找到页面");
  assert_eq!(prepared.not_found[0].title, "未找到页面 | Docs & 中文");
  assert_eq!(prepared.not_found[0].home_label, "回到首页");
  for auxiliary in &prepared.not_found {
    assert!(prepared
      .routes
      .iter()
      .any(|route| route.page.route_path == auxiliary.home));
    assert!(!auxiliary.head_html.contains("canonical"));
    assert!(!auxiliary.head_html.contains("hreflang"));
  }
  verify(&prepared).unwrap();
}

#[test]
fn localized_home_preserves_configured_collection_priority_before_generated_pages() {
  let root = fixture();
  fs::create_dir(root.path().join("pages")).unwrap();
  fs::write(
    root.path().join("pages/probe.tsx"),
    "export const metadata={title:'Probe'};export default function Probe(){return null}",
  )
  .unwrap();
  let mut raw = config();
  raw["pages"] = json!({"dir":"pages"});
  let prepared = plan(root.path(), raw);
  assert_eq!(prepared.not_found[0].home, "/guide/docs/");
  // The generated standalone page in en is real localized content, so it
  // wins over a default-language fallback when en has no docs collection.
  assert_eq!(prepared.not_found[1].home, "/guide/en/probe/");
}

#[test]
fn auxiliary_locale_directories_must_be_portable_even_without_content_pages() {
  for locale in ["con", "AUX", "LPT1"] {
    let root = fixture();
    let mut raw = config();
    raw["locales"]["items"] = json!(["zh", "en", locale]);
    let validated = parse_config(&raw.to_string(), root.path()).unwrap();
    let error = prepare(&validated)
      .expect_err("auxiliary output must reject Windows device directories on every host");
    assert!(error.contains("DOCS_OUTPUT_PATH"), "{locale}: {error}");
  }
  let root = fixture();
  let mut raw = config();
  raw["locales"] = json!({"default":"con","items":["con","en"]});
  raw["docs"]["collections"][0]["locale"] = json!("con");
  let prepared = plan(root.path(), raw);
  assert_eq!(prepared.not_found[0].locale, "con");
  assert_eq!(prepared.not_found[0].output_path, "404.html");
  verify(&prepared).unwrap();
}

#[test]
fn physical_not_found_reserves_case_and_ancestor_static_names() {
  for name in [
    "404.html",
    "404.HTML",
    "404.html/child.txt",
    "en/404.html",
    "EN/404.HTML",
    "en/404.html/child.txt",
    "en",
  ] {
    let root = fixture();
    let file = root.path().join("static").join(name);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, "conflict").unwrap();
    let mut raw = config();
    raw["staticDirs"] = json!([{"dir":"static"}]);
    let validated = parse_config(&raw.to_string(), root.path()).unwrap();
    let error = prepare(&validated).expect_err("404 output must be reserved");
    assert!(error.contains("DOCS_STATIC_COLLISION"), "{name}: {error}");
  }
}

#[test]
fn forged_content_output_cannot_take_the_auxiliary_not_found_slot() {
  let root = fixture();
  let original = plan(root.path(), config());
  for output in [
    "404.html",
    "404.HTML",
    "404.html/index.html",
    "en/404.html",
    "EN/404.HTML",
    "en",
  ] {
    let mut changed = original.clone();
    changed.routes[0].output_path = output.into();
    changed.snapshot_id.clear();
    changed.snapshot_id = format!(
      "{:x}",
      Sha256::digest(serde_json::to_vec(&changed).unwrap())
    );
    let error = verify(&changed).expect_err("auxiliary output collision must be revalidated");
    assert!(error.contains("DOCS_STATIC_COLLISION"), "{output}: {error}");
  }
}

#[test]
fn browser_and_server_applications_export_only_public_not_found_head_policy() {
  let root = fixture();
  let prepared = plan(root.path(), config());
  for generated in [
    farmfe_docs_plan::application(&prepared).unwrap(),
    farmfe_docs_plan::browser_application(&prepared).unwrap(),
  ] {
    let encoded = generated
      .lines()
      .find_map(|line| line.strip_prefix("export const notFoundPages = "))
      .expect("Rust application must export public NotFound Head");
    let head: Value = serde_json::from_str(encoded.strip_suffix(';').unwrap()).unwrap();
    let heads = head.as_array().unwrap();
    assert_eq!(heads.len(), prepared.not_found.len());
    for (head, page) in heads.iter().zip(&prepared.not_found) {
      assert_eq!(head["locale"], page.locale);
      assert_eq!(head["home"], page.home);
      assert_eq!(head["routeBase"], page.route_base);
      assert_eq!(head["robots"], "noindex,follow");
      assert!(head.get("outputPath").is_none());
      assert!(head.get("headHtml").is_none());
    }
  }
}
