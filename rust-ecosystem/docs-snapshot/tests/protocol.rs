use farmfe_docs_snapshot::{Receipt, ResourceRecord, Target, hash, observe, read_json};
use std::{fs, path::Path};

fn receipt(root: &Path) -> Receipt {
  let source = root.join("source.js");
  fs::write(&source, b"hello").unwrap();
  Receipt {
    schema_version: 2,
    plan_snapshot_id: "plan".into(),
    target: Target::Browser,
    root: root.to_str().unwrap().into(),
    sources: vec![observe(&source, &[]).unwrap()],
    input_observations: vec![],
    resources: vec![ResourceRecord {
      name: "entry.js".into(),
      resource_type: "js".into(),
      length: 5,
      hash: hash(b"hello"),
    }],
  }
}

#[test]
fn private_receipt_json_rejects_malformed_missing_and_unknown_fields() {
  let root = tempfile::tempdir().unwrap();
  let valid = serde_json::to_value(receipt(root.path())).unwrap();
  let path = root.path().join("receipt.json");
  fs::write(&path, b"{").unwrap();
  assert!(read_json::<Receipt>(&path).is_err());
  for field in [
    "schemaVersion",
    "sources",
    "resources",
    "target",
    "inputObservations",
  ] {
    let mut value = valid.clone();
    value.as_object_mut().unwrap().remove(field);
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(
      read_json::<Receipt>(&path).is_err(),
      "accepted missing {field}"
    );
  }
  for section in ["receipt", "source", "resource"] {
    let mut value = valid.clone();
    let object = match section {
      "source" => value["sources"][0].as_object_mut().unwrap(),
      "resource" => value["resources"][0].as_object_mut().unwrap(),
      _ => value.as_object_mut().unwrap(),
    };
    object.insert("unexpected".into(), true.into());
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(
      read_json::<Receipt>(&path).is_err(),
      "accepted unknown {section} field"
    );
  }
}

#[test]
fn private_receipt_rejects_empty_sets_and_duplicate_resources() {
  let root = tempfile::tempdir().unwrap();
  let valid = receipt(root.path());
  valid
    .validate("plan", Target::Browser, &valid.root)
    .unwrap();
  let mut value = valid.clone();
  value.sources.clear();
  assert!(
    value
      .validate("plan", Target::Browser, &valid.root)
      .is_err()
  );
  let mut value = valid.clone();
  value.resources.clear();
  assert!(
    value
      .validate("plan", Target::Browser, &valid.root)
      .is_err()
  );
  let mut value = valid.clone();
  value.resources.push(value.resources[0].clone());
  assert!(
    value
      .validate("plan", Target::Browser, &valid.root)
      .is_err()
  );
}

#[cfg(any(windows, unix))]
#[test]
fn watch_source_rejects_directory_links_at_root_and_inside_tree() {
  let root = tempfile::tempdir().unwrap();
  let target = root.path().join("actual");
  let watched = root.path().join("watched");
  let link = watched.join("link");
  fs::create_dir(&target).unwrap();
  fs::create_dir(&watched).unwrap();
  fs::write(target.join("data.txt"), "source").unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(&target, &link).unwrap();
  #[cfg(windows)]
  {
    let output = std::process::Command::new("cmd.exe")
      .args(["/d", "/c", "mklink", "/J"])
      .arg(&link)
      .arg(&target)
      .output()
      .unwrap();
    assert!(
      output.status.success(),
      "junction fixture: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }
  assert!(
    observe(&link, &[]).is_err(),
    "a linked watch root must be rejected"
  );
  assert!(
    observe(&watched, &[]).is_err(),
    "a linked child must not be traversed"
  );
}
