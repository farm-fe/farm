use farmfe_docs_snapshot::{
  Receipt, ResourceRecord, Target, hash, observe, verify_resource, verify_source,
};
use std::fs;

#[cfg(windows)]
#[test]
fn windows_case_variant_watch_root_excludes_generated_directory() {
  let root = tempfile::tempdir().unwrap();
  let output = root.path().join("generated-cache");
  fs::create_dir(&output).unwrap();
  fs::write(output.join("cache.bin"), "first").unwrap();
  let variant = std::path::PathBuf::from(root.path().to_str().unwrap().to_uppercase());
  let source = observe(&variant, &[output.clone()]).unwrap();
  fs::write(output.join("cache.bin"), "changed").unwrap();
  verify_source(&source).expect("case spelling must not include generated cache bytes");
}

#[cfg(windows)]
#[test]
fn windows_canonical_exclusion_matches_ordinary_watch_path() {
  let root = tempfile::tempdir().unwrap();
  let output = root.path().join("generated-cache");
  fs::create_dir(&output).unwrap();
  fs::write(output.join("cache.bin"), "first").unwrap();
  let source = observe(root.path(), &[fs::canonicalize(&output).unwrap()]).unwrap();
  fs::write(output.join("cache.bin"), "changed").unwrap();
  verify_source(&source).expect("extended path prefixes must retain the same exclusion");
}

#[test]
fn missing_exclusion_can_be_created_without_changing_the_receipt() {
  let root = tempfile::tempdir().unwrap();
  let output = root.path().join("generated-cache");
  let source = observe(root.path(), &[output.clone()]).unwrap();
  fs::create_dir(&output).unwrap();
  fs::write(output.join("cache.bin"), "new").unwrap();
  verify_source(&source).expect("a previously missing excluded root stays excluded");
}

#[test]
fn source_changes_and_removal_are_rejected_without_timestamp_assumptions() {
  let root = tempfile::tempdir().unwrap();
  let path = root.path().join("source.tsx");
  fs::write(&path, "first").unwrap();
  let record = observe(&path, &[]).unwrap();
  verify_source(&record).unwrap();
  fs::write(&path, "other").unwrap();
  assert!(
    verify_source(&record)
      .unwrap_err()
      .contains("DOCS_INPUT_SEAL")
  );
  fs::remove_file(path).unwrap();
  assert!(verify_source(&record).is_err());
}

#[test]
fn watch_directory_rejects_empty_membership_changes() {
  for mutation in ["add", "remove", "rename"] {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("empty")).unwrap();
    let record = observe(root.path(), &[]).unwrap();
    match mutation {
      "add" => fs::create_dir(root.path().join("added")).unwrap(),
      "remove" => fs::remove_dir(root.path().join("empty")).unwrap(),
      _ => fs::rename(root.path().join("empty"), root.path().join("renamed")).unwrap(),
    }
    assert!(verify_source(&record).is_err(), "missed {mutation}");
  }
}

#[test]
fn explicitly_watched_directory_is_not_hidden_by_ancestor_pruning() {
  for mutation in ["change", "add", "remove"] {
    let root = tempfile::tempdir().unwrap();
    let dependencies = root.path().join("node_modules");
    let package = dependencies.join("watched-package");
    fs::create_dir_all(&package).unwrap();
    fs::write(package.join("data.txt"), "original").unwrap();
    let record = observe(&package, &[dependencies]).unwrap();
    match mutation {
      "change" => fs::write(package.join("data.txt"), "modified").unwrap(),
      "add" => fs::write(package.join("extra.txt"), "new").unwrap(),
      _ => fs::remove_file(package.join("data.txt")).unwrap(),
    }
    assert!(verify_source(&record).is_err(), "missed {mutation}");
  }
}

#[test]
fn watch_directory_checks_members_bytes_and_explicit_output_exclusions() {
  let root = tempfile::tempdir().unwrap();
  let output = root.path().join("build");
  fs::create_dir(&output).unwrap();
  fs::write(root.path().join("source.scss"), "first").unwrap();
  let record = observe(root.path(), &[output.clone()]).unwrap();
  fs::write(output.join("generated.js"), "ignored output").unwrap();
  verify_source(&record).unwrap();
  fs::write(root.path().join("new.scss"), "new").unwrap();
  assert!(verify_source(&record).is_err());
}

#[test]
fn receipt_identity_duplicates_paths_and_resource_bytes_are_checked() {
  let root = tempfile::tempdir().unwrap();
  let source = root.path().join("source.tsx");
  fs::write(&source, "first").unwrap();
  let mut receipt = Receipt {
    schema_version: 2,
    plan_snapshot_id: "plan".into(),
    target: Target::Browser,
    root: "root".into(),
    sources: vec![observe(&source, &[]).unwrap()],
    input_observations: vec![],
    resources: vec![ResourceRecord {
      name: "entry.js".into(),
      resource_type: "js".into(),
      length: 5,
      hash: hash(b"first"),
    }],
  };
  receipt.validate("plan", Target::Browser, "root").unwrap();
  assert!(receipt.validate("other", Target::Browser, "root").is_err());
  assert!(receipt.validate("plan", Target::Server, "root").is_err());
  for unsupported in [0, 1, 3] {
    receipt.schema_version = unsupported;
    assert!(receipt.validate("plan", Target::Browser, "root").is_err());
  }
  receipt.schema_version = 2;
  receipt.sources.push(receipt.sources[0].clone());
  assert!(receipt.validate("plan", Target::Browser, "root").is_err());
  receipt.sources.pop();
  receipt.resources[0].name = "../escape.js".into();
  assert!(receipt.validate("plan", Target::Browser, "root").is_err());
  receipt.resources[0].name = "entry.js".into();
  verify_resource(&receipt.resources[0], &source).unwrap();
  fs::write(&source, "other").unwrap();
  assert!(verify_resource(&receipt.resources[0], &source).is_err());
  assert!(serde_json::from_str::<Receipt>("{}").is_err());
  let mut value = serde_json::to_value(&receipt).unwrap();
  value["unexpected"] = true.into();
  assert!(serde_json::from_value::<Receipt>(value).is_err());
}
