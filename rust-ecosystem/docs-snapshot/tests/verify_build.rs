use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{DocPlan, prepare};
use farmfe_docs_snapshot::{
  Receipt, ResourceRecord, Target, Verification, hash, observe, verify_build,
};
use serde_json::json;
use std::{fs, path::Path};

fn write(path: &Path, bytes: &[u8]) {
  fs::write(path, bytes).unwrap();
}

fn resource(name: &str, bytes: &[u8]) -> ResourceRecord {
  ResourceRecord {
    name: name.into(),
    resource_type: "test".into(),
    length: bytes.len(),
    hash: hash(bytes),
  }
}

fn fixture() -> (tempfile::TempDir, DocPlan, Verification) {
  fixture_pages(1)
}

fn fixture_pages(count: usize) -> (tempfile::TempDir, DocPlan, Verification) {
  fixture_pages_with_initial_validation(count, true)
}

fn fixture_pages_with_initial_validation(
  count: usize,
  initial_validation: bool,
) -> (tempfile::TempDir, DocPlan, Verification) {
  let root = tempfile::tempdir().unwrap();
  let path = root.path();
  for name in ["docs", "public", "server"] {
    fs::create_dir(path.join(name)).unwrap();
  }
  let source = path.join("docs/index.md");
  write(&source, b"# Hello\n");
  for index in 1..count {
    write(
      &path.join(format!("docs/page-{index}.md")),
      format!("# Page {index}\n").as_bytes(),
    );
  }
  let config = parse_config(&json!({"title":"Verify","url":"https://example.com","locales":{"default":"en","items":["en","zh"]},"docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}}).to_string(),path).unwrap();
  let plan = prepare(&config).unwrap();
  let template = b"<html></html>";
  let manifest_bytes = serde_json::to_vec(
    &json!({"template":"template.html", "notFound":farmfe_docs_plan::not_found_outputs(&plan)}),
  )
  .unwrap();
  let manifest = manifest_bytes.as_slice();
  let javascript = b"globalThis.test=true;";
  let server = b"exports.render=()=>'';";
  write(&path.join("template.html"), template);
  write(&path.join("public/docs-manifest.json"), manifest);
  write(&path.join("public/entry.js"), javascript);
  write(&path.join("server/server.cjs"), server);
  write(
    &path.join("server/package.json"),
    b"{\"type\":\"commonjs\"}\n",
  );
  let root_text = path.to_str().unwrap().to_string();
  let browser = Receipt {
    schema_version: 2,
    plan_snapshot_id: plan.snapshot_id.clone(),
    target: Target::Browser,
    root: root_text.clone(),
    sources: vec![observe(&source, &[]).unwrap()],
    input_observations: vec![],
    resources: vec![
      resource("template.html", template),
      resource("docs-manifest.json", manifest),
      resource("entry.js", javascript),
    ],
  };
  let server = Receipt {
    target: Target::Server,
    resources: vec![resource("server.cjs", server)],
    ..browser.clone()
  };
  let browser_bytes = serde_json::to_vec(&browser).unwrap();
  let server_bytes = serde_json::to_vec(&server).unwrap();
  write(&path.join("browser.json"), &browser_bytes);
  write(&path.join("server.json"), &server_bytes);
  let mut written = vec![];
  for route in &plan.routes {
    let file = path.join("public").join(&route.output_path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    write(&file, b"<h1>Hello</h1>");
    written.push(
      json!({"key":route.page.key,"outputPath":route.output_path,"hash":hash(b"<h1>Hello</h1>")}),
    );
  }
  let pages=plan.routes.iter().map(|route|json!({"key":route.page.key,"routePath":route.page.route_path,"outputPath":route.output_path})).collect::<Vec<_>>();
  let auxiliary = b"<h1>Page not found</h1>";
  let mut auxiliary_writes = vec![];
  let mut auxiliary_report = vec![];
  for page in &plan.not_found {
    let file = path.join("public").join(&page.output_path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    write(&file, auxiliary);
    auxiliary_writes
      .push(json!({"locale":page.locale,"outputPath":page.output_path,"hash":hash(auxiliary)}));
    auxiliary_report.push(
      json!({"locale":page.locale,"routeBase":page.route_base,"outputPath":page.output_path}),
    );
  }
  let report=serde_json::to_vec(&json!({"schemaVersion":2,"snapshotId":plan.snapshot_id,"pages":pages,"notFound":auxiliary_report,"preview":{"basePath":plan.site.base_path},"diagnostics":plan.public_diagnostics})).unwrap();
  write(&path.join("public/docs-build-report.json"), &report);
  let writes=serde_json::to_vec(&json!({"schemaVersion":3,"snapshotId":plan.snapshot_id,"pages":written,"notFound":auxiliary_writes,"reportHash":hash(&report)})).unwrap();
  write(&path.join("writes.json"), &writes);
  let text = |name: &str| path.join(name).to_str().unwrap().to_string();
  let verification = Verification {
    browser_receipt: text("browser.json"),
    browser_hash: hash(&browser_bytes),
    server_receipt: text("server.json"),
    server_hash: hash(&server_bytes),
    writes_path: text("writes.json"),
    writes_hash: hash(&writes),
    public_directory: text("public"),
    server_directory: text("server"),
    template_path: text("template.html"),
  };
  if initial_validation {
    verify_build(&plan, &root_text, &verification).unwrap();
  }
  (root, plan, verification)
}

#[test]
fn precommit_checks_sources_compiled_resources_template_html_report_and_receipts() {
  for name in [
    "docs/index.md",
    "public/entry.js",
    "server/server.cjs",
    "template.html",
    "public/docs-build-report.json",
    "browser.json",
    "server.json",
    "writes.json",
    "server/package.json",
  ] {
    let (root, plan, verification) = fixture();
    write(&root.path().join(name), b"changed");
    assert!(
      verify_build(&plan, root.path().to_str().unwrap(), &verification).is_err(),
      "missed {name}"
    );
  }
  let (root, plan, verification) = fixture();
  write(
    &root.path().join("public").join(&plan.routes[0].output_path),
    b"changed HTML",
  );
  assert!(verify_build(&plan, root.path().to_str().unwrap(), &verification).is_err());
}

#[test]
fn precommit_rejects_new_content_and_unexpected_files_in_both_trees() {
  for name in ["docs/new.md", "public/extra.js", "server/extra.cjs"] {
    let (root, plan, verification) = fixture();
    write(&root.path().join(name), b"# Added");
    assert!(
      verify_build(&plan, root.path().to_str().unwrap(), &verification).is_err(),
      "missed {name}"
    );
  }
}

#[test]
fn precommit_rejects_missing_resources_and_conflicting_render_identity() {
  let (root, plan, mut verification) = fixture();
  fs::remove_file(root.path().join("public/entry.js")).unwrap();
  assert!(verify_build(&plan, root.path().to_str().unwrap(), &verification).is_err());
  let (root, plan, other) = fixture();
  verification = other;
  let mut writes: serde_json::Value =
    serde_json::from_slice(&fs::read(&verification.writes_path).unwrap()).unwrap();
  writes["pages"][0]["outputPath"] = "../escape.html".into();
  let bytes = serde_json::to_vec(&writes).unwrap();
  write(Path::new(&verification.writes_path), &bytes);
  verification.writes_hash = hash(&bytes);
  assert!(verify_build(&plan, root.path().to_str().unwrap(), &verification).is_err());
}

#[test]
fn not_found_is_required_and_its_actual_html_is_sealed() {
  for name in ["404.html", "zh/404.html"] {
    for remove in [false, true] {
      let (root, plan, verification) = fixture();
      let file = root.path().join("public").join(name);
      if remove {
        fs::remove_file(file).unwrap();
      } else {
        write(&file, b"altered NotFound HTML");
      }
      assert!(verify_build(&plan, root.path().to_str().unwrap(), &verification).is_err());
    }
  }
}

#[test]
fn auxiliary_receipt_cannot_be_missing_forged_or_an_old_protocol() {
  for mutation in [
    "missing",
    "path",
    "case",
    "hash",
    "old",
    "unknown",
    "locale",
    "duplicate",
    "fewer",
    "order",
    "object",
  ] {
    let (root, plan, mut verification) = fixture();
    let mut writes: serde_json::Value =
      serde_json::from_slice(&fs::read(&verification.writes_path).unwrap()).unwrap();
    match mutation {
      "missing" => {
        writes.as_object_mut().unwrap().remove("notFound");
      }
      "path" => writes["notFound"][1]["outputPath"] = "docs/index.html".into(),
      "case" => writes["notFound"][1]["outputPath"] = "ZH/404.HTML".into(),
      "hash" => writes["notFound"][1]["hash"] = hash(b"not the written HTML").into(),
      "old" => writes["schemaVersion"] = 2.into(),
      "locale" => writes["notFound"][1]["locale"] = "fr".into(),
      "duplicate" => {
        let extra = writes["notFound"][0].clone();
        writes["notFound"].as_array_mut().unwrap().push(extra);
      }
      "fewer" => {
        writes["notFound"].as_array_mut().unwrap().pop();
      }
      "order" => writes["notFound"].as_array_mut().unwrap().reverse(),
      "object" => writes["notFound"] = writes["notFound"][0].clone(),
      _ => writes["notFound"][1]["key"] = "forged-content-key".into(),
    }
    let bytes = serde_json::to_vec(&writes).unwrap();
    write(Path::new(&verification.writes_path), &bytes);
    verification.writes_hash = hash(&bytes);
    assert!(
      verify_build(&plan, root.path().to_str().unwrap(), &verification).is_err(),
      "{mutation}"
    );
  }
}

#[test]
fn auxiliary_output_cannot_also_be_a_content_page() {
  let (root, plan, mut verification) = fixture();
  let mut writes: serde_json::Value =
    serde_json::from_slice(&fs::read(&verification.writes_path).unwrap()).unwrap();
  writes["pages"].as_array_mut().unwrap().push(
    json!({"key":"invented","outputPath":"404.html","hash":hash(b"<h1>Page not found</h1>")}),
  );
  let bytes = serde_json::to_vec(&writes).unwrap();
  write(Path::new(&verification.writes_path), &bytes);
  verification.writes_hash = hash(&bytes);
  assert!(verify_build(&plan, root.path().to_str().unwrap(), &verification).is_err());
}

#[test]
fn public_auxiliary_table_is_exact_even_after_report_and_receipt_are_resealed() {
  for mutation in [
    "locale", "path", "mount", "extra", "fewer", "order", "private", "object", "old",
  ] {
    let (root, plan, mut verification) = fixture();
    let report_path = root.path().join("public/docs-build-report.json");
    let mut report: serde_json::Value =
      serde_json::from_slice(&fs::read(&report_path).unwrap()).unwrap();
    match mutation {
      "locale" => report["notFound"][1]["locale"] = "fr".into(),
      "path" => report["notFound"][1]["outputPath"] = "404.html".into(),
      "mount" => report["notFound"][1]["routeBase"] = "/zhish/".into(),
      "extra" => {
        let page = report["notFound"][0].clone();
        report["notFound"].as_array_mut().unwrap().push(page);
      }
      "fewer" => {
        report["notFound"].as_array_mut().unwrap().pop();
      }
      "order" => report["notFound"].as_array_mut().unwrap().reverse(),
      "private" => report["notFound"][1]["key"] = "invented".into(),
      "object" => report["notFound"] = report["notFound"][0].clone(),
      _ => report["schemaVersion"] = 1.into(),
    }
    let bytes = serde_json::to_vec(&report).unwrap();
    write(&report_path, &bytes);
    let mut writes: serde_json::Value =
      serde_json::from_slice(&fs::read(&verification.writes_path).unwrap()).unwrap();
    writes["reportHash"] = hash(&bytes).into();
    let bytes = serde_json::to_vec(&writes).unwrap();
    write(Path::new(&verification.writes_path), &bytes);
    verification.writes_hash = hash(&bytes);
    assert!(
      verify_build(&plan, root.path().to_str().unwrap(), &verification).is_err(),
      "{mutation}"
    );
  }
}

#[test]
fn manifest_auxiliary_table_must_match_rust_even_after_resource_receipt_is_resealed() {
  let (root, plan, mut verification) = fixture();
  let manifest_path = root.path().join("public/docs-manifest.json");
  let mut manifest: serde_json::Value =
    serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
  manifest["notFound"][1]["routeBase"] = "/wrong/".into();
  let bytes = serde_json::to_vec(&manifest).unwrap();
  write(&manifest_path, &bytes);
  let mut receipt: Receipt =
    serde_json::from_slice(&fs::read(&verification.browser_receipt).unwrap()).unwrap();
  *receipt
    .resources
    .iter_mut()
    .find(|record| record.name == "docs-manifest.json")
    .unwrap() = resource("docs-manifest.json", &bytes);
  let bytes = serde_json::to_vec(&receipt).unwrap();
  write(Path::new(&verification.browser_receipt), &bytes);
  verification.browser_hash = hash(&bytes);
  assert!(
    verify_build(&plan, root.path().to_str().unwrap(), &verification)
      .unwrap_err()
      .contains("manifest auxiliary outputs differ")
  );
}

#[test]
fn preview_metadata_is_required_and_matches_the_rust_site_even_with_new_receipt_hashes() {
  for mutation in ["missing", "base", "unknown", "null"] {
    let (root, plan, mut verification) = fixture();
    let report_path = root.path().join("public/docs-build-report.json");
    let mut report: serde_json::Value =
      serde_json::from_slice(&fs::read(&report_path).unwrap()).unwrap();
    match mutation {
      "missing" => {
        report.as_object_mut().unwrap().remove("preview");
      }
      "base" => report["preview"]["basePath"] = "/wrong/".into(),
      "unknown" => report["preview"]["guessRoute"] = true.into(),
      _ => report["preview"] = serde_json::Value::Null,
    }
    let report_bytes = serde_json::to_vec(&report).unwrap();
    write(&report_path, &report_bytes);
    let mut writes: serde_json::Value =
      serde_json::from_slice(&fs::read(&verification.writes_path).unwrap()).unwrap();
    writes["reportHash"] = hash(&report_bytes).into();
    let bytes = serde_json::to_vec(&writes).unwrap();
    write(Path::new(&verification.writes_path), &bytes);
    verification.writes_hash = hash(&bytes);
    assert!(
      verify_build(&plan, root.path().to_str().unwrap(), &verification).is_err(),
      "{mutation}"
    );
  }
}

// Environment policy is exercised in isolated test processes: no unsafe global
// environment mutation can race other verifier tests.
#[test]
fn bounded_verification_parent_process() {
  let executable = std::env::current_exe().unwrap();
  for concurrency in ["1", "4"] {
    for case in [
      "valid",
      "corrupt",
      "missing",
      "duplicate",
      "traversal",
      "parent-symlink",
      "leaf-symlink",
    ] {
      let status = std::process::Command::new(&executable)
        .args([
          "--exact",
          "bounded_verification_child",
          "--nocapture",
          "--test-threads=1",
        ])
        .env("FARM_DOCS_VERIFY_CONCURRENCY", concurrency)
        .env("FARM_DOCS_VERIFY_TEST_CASE", case)
        .status()
        .unwrap();
      assert!(status.success(), "concurrency={concurrency} case={case}");
    }
  }
  for invalid in ["0", "5", "01", " 4", "invalid"] {
    let status = std::process::Command::new(&executable)
      .args([
        "--exact",
        "bounded_verification_child",
        "--nocapture",
        "--test-threads=1",
      ])
      .env("FARM_DOCS_VERIFY_CONCURRENCY", invalid)
      .env("FARM_DOCS_VERIFY_TEST_CASE", "invalid-policy")
      .status()
      .unwrap();
    assert!(status.success(), "invalid policy {invalid}");
  }
}

#[test]
fn bounded_verification_child() {
  let Ok(case) = std::env::var("FARM_DOCS_VERIFY_TEST_CASE") else {
    return;
  };
  let (root, plan, mut verification) =
    fixture_pages_with_initial_validation(8, case != "invalid-policy");
  let marker = root.path().join("old-output-marker");
  write(&marker, b"unchanged");
  let html = root.path().join("public").join(&plan.routes[0].output_path);
  match case.as_str() {
    "valid" | "invalid-policy" => {}
    "corrupt" => write(&html, b"changed"),
    "missing" => fs::remove_file(&html).unwrap(),
    "duplicate" | "traversal" => {
      let mut writes: serde_json::Value =
        serde_json::from_slice(&fs::read(&verification.writes_path).unwrap()).unwrap();
      if case == "duplicate" {
        writes["pages"][1] = writes["pages"][0].clone();
      } else {
        writes["pages"][0]["outputPath"] = "../outside.html".into();
      }
      let bytes = serde_json::to_vec(&writes).unwrap();
      write(Path::new(&verification.writes_path), &bytes);
      verification.writes_hash = hash(&bytes);
    }
    "leaf-symlink" => {
      let target = root.path().join("leaf-target.html");
      fs::copy(&html, &target).unwrap();
      fs::remove_file(&html).unwrap();
      #[cfg(unix)]
      let result = std::os::unix::fs::symlink(&target, &html);
      #[cfg(windows)]
      let result = std::os::windows::fs::symlink_file(&target, &html);
      if let Err(failure) = result {
        if failure.kind() == std::io::ErrorKind::PermissionDenied
          || (cfg!(windows) && failure.raw_os_error() == Some(1314))
        {
          eprintln!("symlink test unavailable on this host");
          return;
        }
        panic!("create test symlink: {failure}");
      }
    }
    "parent-symlink" => {
      let parent = html.parent().unwrap();
      let target = root.path().join("parent-target");
      fs::rename(parent, &target).unwrap();
      #[cfg(unix)]
      let result = std::os::unix::fs::symlink(&target, parent);
      #[cfg(windows)]
      let result = std::os::windows::fs::symlink_dir(&target, parent);
      if let Err(failure) = result {
        if failure.kind() == std::io::ErrorKind::PermissionDenied
          || (cfg!(windows) && failure.raw_os_error() == Some(1314))
        {
          eprintln!("symlink test unavailable on this host");
          return;
        }
        panic!("create test symlink: {failure}");
      }
    }
    _ => panic!("unknown test case"),
  }
  let result = verify_build(&plan, root.path().to_str().unwrap(), &verification);
  if case == "valid" {
    result.unwrap();
  } else if case == "invalid-policy" {
    assert!(result.unwrap_err().contains("FARM_DOCS_VERIFY_CONCURRENCY"));
  } else {
    assert!(result.is_err(), "{case}");
  }
  assert_eq!(fs::read(marker).unwrap(), b"unchanged");
}
