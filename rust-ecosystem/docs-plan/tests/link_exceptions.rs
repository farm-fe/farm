use farmfe_docs_config::parse_config;
use farmfe_docs_plan::prepare;
use serde_json::{Value, json};
use std::{fs, path::Path};

fn fixture(target: &str) -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(
    root.path().join("docs/index.md"),
    format!("# Home\n\n[Legacy]({target})\n"),
  )
  .unwrap();
  fs::write(root.path().join("docs/next.md"), "# Next\n").unwrap();
  root
}
fn rule(target: &str, code: &str) -> Value {
  json!({"collection":"main","id":"index","target":target,"code":code,"reason":"Retain the documented baseline issue during migration"})
}
fn config(rules: Vec<Value>) -> Value {
  json!({"title":"Exceptions","url":"https://example.com","basePath":"/guide/","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]},"linkExceptions":rules})
}
fn plan(root: &Path, raw: Value) -> Result<farmfe_docs_plan::DocPlan, String> {
  let validated = parse_config(&raw.to_string(), root).map_err(|error| error.to_string())?;
  prepare(&validated)
}

#[test]
fn exact_anchor_exception_keeps_canonical_url_and_reports_the_original_error() {
  let target = "./next.md?from=old#retired";
  let root = fixture(target);
  let prepared = plan(root.path(), config(vec![rule(target, "DOCS_LINK_ANCHOR")])).unwrap();
  let index = prepared
    .modules
    .iter()
    .find(|module| module.source_path.ends_with("/index.md"))
    .unwrap();
  assert!(index.code.contains("/guide/docs/next/?from=old#retired"));
  let diagnostic = prepared
    .diagnostics
    .iter()
    .find(|value| value.starts_with("[DOCS_LINK_EXCEPTION]"))
    .unwrap();
  assert!(
    diagnostic.contains("linkExceptions[0]")
      && diagnostic.contains("index.md:3:1")
      && diagnostic.contains("DOCS_LINK_ANCHOR")
      && diagnostic.contains("baseline issue"),
    "{diagnostic}"
  );
  let app = farmfe_docs_plan::application(&prepared).unwrap();
  assert!(!app.contains("linkExceptions") && !app.contains("baseline issue"));
}

#[test]
fn exact_missing_document_exception_preserves_the_unresolved_authored_target() {
  let target = "./retired.md#old";
  let root = fixture(target);
  let prepared = plan(root.path(), config(vec![rule(target, "DOCS_LINK_MISSING")])).unwrap();
  assert!(
    prepared
      .modules
      .iter()
      .any(|module| module.code.contains(target))
  );
  assert!(
    prepared
      .diagnostics
      .iter()
      .any(|value| value.contains("DOCS_LINK_EXCEPTION") && value.contains("DOCS_LINK_MISSING"))
  );
}

#[test]
fn public_report_diagnostics_do_not_leak_exception_reasons_or_source_paths() {
  let target = "./next.md#retired";
  let root = fixture(target);
  let prepared = plan(root.path(), config(vec![rule(target, "DOCS_LINK_ANCHOR")])).unwrap();
  let value = serde_json::to_value(&prepared).unwrap();
  let diagnostic = value["publicDiagnostics"][0]
    .as_str()
    .expect("public diagnostic projection is required");
  assert!(
    diagnostic.contains("DOCS_LINK_EXCEPTION")
      && diagnostic.contains("main/index:3:1")
      && diagnostic.contains(target),
    "{diagnostic}"
  );
  assert!(
    !diagnostic.contains("baseline issue")
      && !diagnostic.contains(&prepared.modules[0].source_path)
  );
  assert!(prepared.diagnostics[0].contains("baseline issue"));
}

#[test]
fn exceptions_match_raw_target_and_error_code_exactly() {
  let root = fixture("./next.md#retired");
  for candidate in [
    rule("./next.md#other", "DOCS_LINK_ANCHOR"),
    rule("./next.md#%72etired", "DOCS_LINK_ANCHOR"),
    rule("./next.md#retired", "DOCS_LINK_MISSING"),
  ] {
    let error = plan(root.path(), config(vec![candidate])).unwrap_err();
    assert!(error.starts_with("[DOCS_LINK_ANCHOR]"), "{error}");
  }
}

#[test]
fn canonical_source_ids_match_but_cannot_create_duplicate_exception_slots() {
  let target = "./next.md#retired";
  let root = fixture(target);
  fs::write(
    root.path().join("docs/index.md"),
    format!("---\nid: Café\n---\n# Home\n\n[Legacy]({target})\n"),
  )
  .unwrap();
  let mut composed = rule(target, "DOCS_LINK_ANCHOR");
  composed["id"] = json!("Café");
  let mut decomposed = composed.clone();
  decomposed["id"] = json!("Cafe\u{301}");
  assert!(plan(root.path(), config(vec![decomposed.clone()])).is_ok());
  let error = plan(root.path(), config(vec![composed, decomposed])).unwrap_err();
  assert!(
    error.contains("DOCS_CONFIG_LINK_EXCEPTION") && error.contains("canonical"),
    "{error}"
  );
}

#[test]
fn fixed_or_removed_links_require_removing_the_stale_exception() {
  let target = "./next.md#retired";
  let root = fixture(target);
  let raw = config(vec![rule(target, "DOCS_LINK_ANCHOR")]);
  assert!(plan(root.path(), raw.clone()).is_ok());
  fs::write(
    root.path().join("docs/next.md"),
    "# Next\n\n## Restored {#retired}\n",
  )
  .unwrap();
  let error = plan(root.path(), raw).unwrap_err();
  assert!(
    error.contains("DOCS_LINK_EXCEPTION_UNUSED") && error.contains("linkExceptions[0]"),
    "{error}"
  );
  assert!(plan(root.path(), config(vec![])).is_ok());
}

#[test]
fn allowed_error_codes_cannot_suppress_escape_or_protocol_diagnostics() {
  for (target, code) in [
    ("%2e%2e/outside.md", "DOCS_LINK_ESCAPE"),
    ("javascript:alert(1)", "DOCS_LINK_PROTOCOL"),
  ] {
    let root = fixture(target);
    let error = plan(root.path(), config(vec![rule(target, "DOCS_LINK_MISSING")])).unwrap_err();
    assert!(error.starts_with(&format!("[{code}]")), "{error}");
  }
}

#[test]
fn exception_does_not_apply_to_another_collection_sharing_the_source_file() {
  let target = "./next.md#retired";
  let root = fixture(target);
  let mut raw = config(vec![rule(target, "DOCS_LINK_ANCHOR")]);
  raw["locales"] = json!({"default":"en","items":["en","zh"]});
  raw["docs"]["collections"]
    .as_array_mut()
    .unwrap()
    .push(json!({"id":"other","dir":"docs","locale":"zh","routeBase":"/zh/docs/"}));
  let error = plan(root.path(), raw.clone()).unwrap_err();
  assert!(error.starts_with("[DOCS_LINK_ANCHOR]"), "{error}");
  let mut other = rule(target, "DOCS_LINK_ANCHOR");
  other["collection"] = json!("other");
  raw["linkExceptions"].as_array_mut().unwrap().push(other);
  let prepared = plan(root.path(), raw).unwrap();
  assert_eq!(
    prepared
      .diagnostics
      .iter()
      .filter(|value| value.starts_with("[DOCS_LINK_EXCEPTION]"))
      .count(),
    2
  );
}

#[test]
fn missing_source_id_and_duplicate_rules_are_explicit_errors() {
  let root = fixture("./next.md#retired");
  let mut missing = rule("./next.md#retired", "DOCS_LINK_ANCHOR");
  missing["id"] = json!("missing-source");
  let error = plan(root.path(), config(vec![missing])).unwrap_err();
  assert!(error.contains("DOCS_LINK_EXCEPTION_SOURCE"), "{error}");
  let repeated = rule("./next.md#retired", "DOCS_LINK_ANCHOR");
  let error = plan(root.path(), config(vec![repeated.clone(), repeated])).unwrap_err();
  assert!(
    error.contains("DOCS_CONFIG_LINK_EXCEPTION") && error.contains("linkExceptions[1]"),
    "{error}"
  );
}

#[test]
fn exception_configuration_requires_a_reason_and_known_collection_and_code() {
  let root = fixture("./next.md#retired");
  for (field, value, expected) in [
    ("reason", json!(" "), "DOCS_CONFIG_LINK_EXCEPTION"),
    ("collection", json!("absent"), "DOCS_CONFIG_LINK_EXCEPTION"),
    ("target", json!(""), "DOCS_CONFIG_LINK_EXCEPTION"),
    ("code", json!("DOCS_LINK_ESCAPE"), "DOCS_CONFIG_PARSE"),
    ("unknown", json!(true), "DOCS_CONFIG_PARSE"),
  ] {
    let mut invalid = rule("./next.md#retired", "DOCS_LINK_ANCHOR");
    invalid[field] = value;
    let error = plan(root.path(), config(vec![invalid])).unwrap_err();
    assert!(error.contains(expected), "{field}: {error}");
  }
}
