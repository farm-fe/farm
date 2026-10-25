use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{prepare, verify};
use serde_json::{Value, json};
use std::{fs, path::Path};
fn fixture() -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::create_dir_all(root.path().join("static/img")).unwrap();
  fs::write(root.path().join("docs/index.md"),"# Static\n\n![Logo](/img/logo.svg?theme=dark#mark)\n\n[Manual](/guide/manual.pdf?download=1)\n").unwrap();
  fs::write(
    root.path().join("static/img/logo.svg"),
    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\"/>",
  )
  .unwrap();
  fs::write(root.path().join("static/manual.pdf"), b"download bytes").unwrap();
  fs::write(root.path().join("static/unused.txt"), b"not imported").unwrap();
  root
}
fn site() -> Value {
  json!({"title":"Static","url":"https://example.com","basePath":"/guide/","staticDirs":[{"dir":"static"}],"docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}})
}
#[test]
fn compiled_css_static_urls_share_the_scanned_namespace_and_diagnostics() {
  let root = fixture();
  let prepared = plan(root.path(), site()).unwrap();
  for source in [
    "/img/logo.svg?theme=dark#mark",
    "/guide/img/logo.svg?theme=dark#mark",
  ] {
    assert_eq!(
      farmfe_docs_plan::static_resource_url(&prepared, source, "panel.css", 4, 12).unwrap(),
      "/guide/img/logo.svg?theme=dark#mark"
    );
  }
  let missing =
    farmfe_docs_plan::static_resource_url(&prepared, "/missing.svg", "panel.css", 4, 12)
      .unwrap_err();
  assert!(missing.contains("DOCS_STATIC_MISSING") && missing.contains("panel.css:4:12"));
  for source in ["/../secret.svg", "/img%2flogo.svg", "/img/%2e%2e/logo.svg"] {
    assert!(farmfe_docs_plan::static_resource_url(&prepared, source, "panel.css", 4, 12).is_err());
  }
}

#[test]
fn resource_namespace_detects_ancestors_even_with_interposed_sorted_names() {
  for names in [
    vec!["assets", "assets-other", "assets/file.js"],
    vec!["ASSETS", "assets-other", "assets/file.js"],
    vec!["x/entry.css", "x/entry.css-other", "x/entry.css/chunk.js"],
  ] {
    let names = names.into_iter().map(String::from).collect::<Vec<_>>();
    assert!(
      farmfe_docs_plan::validate_resource_names(&names)
        .unwrap_err()
        .contains("DOCS_STATIC_COLLISION")
    );
  }
  assert!(
    farmfe_docs_plan::validate_resource_names(&["assets-other".into(), "assets/file.js".into()])
      .is_ok()
  );
}
fn plan(root: &Path, site: Value) -> Result<farmfe_docs_plan::DocPlan, String> {
  let config = parse_config(&site.to_string(), root).map_err(|e| e.to_string())?;
  prepare(&config)
}
#[test]
fn static_files_keep_public_names_include_unreferenced_files_and_bind_root_urls() {
  let root = fixture();
  let prepared = plan(root.path(), site()).unwrap();
  let value = serde_json::to_value(&prepared).unwrap();
  let names: Vec<_> = value["staticFiles"]
    .as_array()
    .unwrap()
    .iter()
    .map(|f| f["outputPath"].as_str().unwrap())
    .collect();
  assert_eq!(names, vec!["img/logo.svg", "manual.pdf", "unused.txt"]);
  assert_eq!(prepared.assets.len(), 2);
  assert!(prepared.modules[0].code.contains("docs:asset:"));
  assert!(prepared.modules[0].code.contains("?theme=dark#mark"));
  assert!(
    !prepared
      .diagnostics
      .iter()
      .any(|d| d.contains("DOCS_RESOURCE_PENDING"))
  );
  assert!(value["site"].get("staticDirs").is_none());
}
#[test]
fn static_missing_reference_fails_with_source_location_then_recovers() {
  let root = fixture();
  fs::remove_file(root.path().join("static/img/logo.svg")).unwrap();
  let error = plan(root.path(), site()).unwrap_err();
  assert!(
    error.contains("DOCS_STATIC_MISSING") && error.contains("index.md:3:1"),
    "{error}"
  );
  fs::write(root.path().join("static/img/logo.svg"), "<svg/>").unwrap();
  assert!(plan(root.path(), site()).is_ok());
}
#[test]
fn unreferenced_file_changes_and_directory_additions_invalidate_snapshot() {
  let root = fixture();
  let prepared = plan(root.path(), site()).unwrap();
  fs::write(root.path().join("static/unused.txt"), b"changed").unwrap();
  assert!(
    verify(&prepared)
      .unwrap_err()
      .contains("DOCS_SNAPSHOT_CHANGED")
  );
  let prepared = plan(root.path(), site()).unwrap();
  fs::write(root.path().join("static/new.txt"), b"new").unwrap();
  assert!(
    verify(&prepared)
      .unwrap_err()
      .contains("DOCS_SNAPSHOT_CHANGED")
  );
}
#[test]
fn static_outputs_reject_reserved_page_and_case_equivalent_collisions() {
  for file in [
    "docs-build-report.json",
    "DOCS-MANIFEST.JSON",
    "docs/index.html",
  ] {
    let root = fixture();
    let path = root.path().join("static").join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"collision").unwrap();
    let error = plan(root.path(), site()).unwrap_err();
    assert!(error.contains("COLLISION"), "{error}");
  }
  let root = fixture();
  fs::create_dir(root.path().join("other")).unwrap();
  fs::write(root.path().join("other/MANUAL.PDF"), b"different").unwrap();
  let mut config = site();
  config["staticDirs"] = json!([{"dir":"static"},{"dir":"other"}]);
  let error = plan(root.path(), config).unwrap_err();
  assert!(error.contains("COLLISION"), "{error}");
}
#[test]
fn logo_and_blog_author_static_images_are_normalized_by_rust() {
  let root = fixture();
  fs::create_dir(root.path().join("blog")).unwrap();
  fs::write(
    root.path().join("blog/index.md"),
    "---\nauthors: [{name: Writer, image: /img/logo.svg}]\n---\n# Post\n",
  )
  .unwrap();
  let mut config = site();
  config["theme"] = json!({"logo":{"src":"/img/logo.svg"}});
  config["blog"] = json!({"collections":[{"id":"news","dir":"blog","routeBase":"/blog/"}]});
  let result = serde_json::to_value(plan(root.path(), config).unwrap()).unwrap();
  assert_eq!(
    result["site"]["theme"]["logo"]["src"],
    "/guide/img/logo.svg"
  );
  let post = result["routes"]
    .as_array()
    .unwrap()
    .iter()
    .find(|r| r["page"]["kind"] == "blog")
    .unwrap();
  assert_eq!(
    post["page"]["blog"]["authors"][0]["image"],
    "/guide/img/logo.svg"
  );
}

#[test]
fn custom_mount_encodes_filenames_and_relative_static_references_use_named_output() {
  let root = fixture();
  fs::write(root.path().join("static/手册 #1.pdf"), b"unicode").unwrap();
  fs::write(root.path().join("docs/index.md"),"# Files\n\n[Manual](/guide/media/%E6%89%8B%E5%86%8C%20%231.pdf?download=1#page=2)\n\n![Image](../static/img/logo.svg)\n").unwrap();
  let mut config = site();
  config["staticDirs"] = json!([{"dir":"static","routeBase":"/media/"}]);
  let prepared = plan(root.path(), config).unwrap();
  assert!(prepared.assets.iter().all(|file| file.is_static));
  let image = prepared
    .assets
    .iter()
    .find(|file| file.source_path.ends_with("logo.svg"))
    .unwrap();
  assert_eq!(image.output_path, "media/img/logo.svg");
  let unicode = prepared
    .assets
    .iter()
    .find(|file| file.output_path.contains("手册"))
    .unwrap();
  assert_eq!(
    farmfe_docs_plan::asset_url(&prepared.site, unicode),
    "/guide/media/%E6%89%8B%E5%86%8C%20%231.pdf"
  );
}

#[test]
fn static_config_rejects_root_overlaps_and_private_tooling_sources() {
  let root = fixture();
  for dirs in [
    json!([{"dir":"."}]),
    json!([{"dir":"static"},{"dir":"static/img"}]),
  ] {
    let mut config = site();
    config["staticDirs"] = dirs;
    assert!(
      plan(root.path(), config)
        .unwrap_err()
        .contains("DOCS_STATIC_CONFIG")
    );
  }
  fs::create_dir(root.path().join(".farm-docs")).unwrap();
  let mut config = site();
  config["staticDirs"] = json!([{"dir":".farm-docs"}]);
  assert!(
    plan(root.path(), config)
      .unwrap_err()
      .contains("DOCS_STATIC_CONFIG")
  );
}

#[test]
fn private_transaction_inputs_cannot_be_published_or_used_as_content() {
  let root = fixture();
  fs::create_dir(root.path().join(".farm-docs-locks")).unwrap();
  fs::write(
    root.path().join(".farm-docs-locks/private.md"),
    "# Private journal content\n",
  )
  .unwrap();
  for name in [".farm-docs-locks", ".FARM-DOCS-LOCKS"] {
    fs::create_dir_all(root.path().join(name)).unwrap();
    let mut config = site();
    config["staticDirs"] = json!([{"dir":name}]);
    assert!(
      plan(root.path(), config)
        .unwrap_err()
        .contains("DOCS_STATIC_CONFIG")
    );
    let mut config = site();
    config["docs"]["collections"][0]["dir"] = json!(name);
    assert!(
      plan(root.path(), config)
        .unwrap_err()
        .contains("DOCS_CONFIG_PATH")
    );
  }
}

#[test]
fn private_transaction_descendants_are_not_static_resources() {
  let root = fixture();
  fs::create_dir(root.path().join("static/.farm-docs-locks")).unwrap();
  fs::write(
    root.path().join("static/.farm-docs-locks/journal.json"),
    "{\"private\":true}",
  )
  .unwrap();
  assert!(
    plan(root.path(), site())
      .unwrap_err()
      .contains("DOCS_STATIC_PATH")
  );
}

#[test]
fn private_transaction_assets_cannot_be_imported_by_markdown() {
  let root = fixture();
  fs::create_dir(root.path().join(".farm-docs-locks")).unwrap();
  fs::write(
    root.path().join(".farm-docs-locks/journal.json"),
    "{\"private\":true}",
  )
  .unwrap();
  fs::write(
    root.path().join("docs/index.md"),
    "# Private asset\n\n![Private](../.farm-docs-locks/journal.json)\n",
  )
  .unwrap();
  assert!(
    plan(root.path(), site())
      .unwrap_err()
      .contains("DOCS_ASSET_PATH")
  );
}

#[test]
fn private_transaction_children_are_excluded_from_content_discovery() {
  let root = fixture();
  fs::create_dir(root.path().join("docs/.farm-docs-locks")).unwrap();
  fs::write(
    root.path().join("docs/.farm-docs-locks/private.md"),
    "# Private journal content\n",
  )
  .unwrap();
  let prepared = plan(root.path(), site()).unwrap();
  assert_eq!(prepared.routes.len(), 1);
}

#[test]
fn root_resource_escape_and_missing_metadata_do_not_succeed() {
  let root = fixture();
  fs::write(
    root.path().join("docs/index.md"),
    "# Escape\n\n![Image](/img/%2e%2e/manual.pdf)\n",
  )
  .unwrap();
  assert!(
    plan(root.path(), site())
      .unwrap_err()
      .contains("DOCS_STATIC_PATH")
  );
  let root = fixture();
  let mut config = site();
  config["theme"] = json!({"logo":{"src":"/img/absent.svg"}});
  let error = plan(root.path(), config).unwrap_err();
  assert!(
    error.contains("DOCS_STATIC_MISSING") && error.contains("theme.logo.src"),
    "{error}"
  );
}
