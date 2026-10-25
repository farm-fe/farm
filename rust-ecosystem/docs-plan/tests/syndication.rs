use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{prepare, syndication::open_search_targets, verify};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;

fn fixture() -> (tempfile::TempDir, serde_json::Value) {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("pages")).unwrap();
  fs::write(root.path().join("pages/index.md"), "# Home\n").unwrap();
  let config = json!({"title":"Search resources","url":"https://example.com","basePath":"/guide/",
    "locales":{"default":"en","items":["en","zh"]},"pages":{"dir":"pages"},
    "search":{"provider":"algolia","options":{"appId":"APP","apiKey":"public","indexName":"docs"},
      "openSearch":{"pageId":"search","shortName":"Docs","description":"Search docs"}}});
  (root, config)
}

#[test]
fn opensearch_requires_a_real_rust_indexed_target_in_each_declared_locale() {
  let (root, raw) = fixture();
  let config = parse_config(&raw.to_string(), root.path()).unwrap();
  let error = prepare(&config).expect_err("OpenSearch cannot advertise a missing search page");
  assert!(error.contains("DOCS_SEARCH_PAGE"), "{error}");
}

#[test]
fn opensearch_outputs_reserve_every_locale_before_static_files_can_claim_them() {
  for output in ["opensearch.xml", "zh/opensearch.xml", "ZH/OPENSEARCH.XML"] {
    let (root, mut raw) = fixture();
    fs::write(
      root.path().join("pages/search.tsx"),
      "export const metadata={title:'Search'};export default ()=> <h1>Search</h1>;",
    )
    .unwrap();
    fs::create_dir_all(root.path().join("static").join(output).parent().unwrap()).unwrap();
    fs::write(
      root.path().join("static").join(output),
      "must not overwrite generated search description",
    )
    .unwrap();
    raw["staticDirs"] = json!([{"dir":"static"}]);
    let config = parse_config(&raw.to_string(), root.path()).unwrap();
    let error = prepare(&config).expect_err("Rust auxiliary namespace is reserved");
    assert!(error.contains("DOCS_STATIC_COLLISION"), "{output}: {error}");
  }
}

#[test]
fn targets_use_real_localized_routes_under_unicode_mount_and_revalidate_missing_locales() {
  let (root, mut raw) = fixture();
  fs::write(
    root.path().join("pages/search.tsx"),
    "export const metadata={title:'Search'};export default ()=> <h1>Search</h1>;",
  )
  .unwrap();
  raw["basePath"] = json!("/指南/");
  raw["locales"]["default"] = json!("zh");
  raw["trailingSlash"] = json!("never");
  let plan = prepare(&parse_config(&raw.to_string(), root.path()).unwrap()).unwrap();
  let targets = open_search_targets(&plan).unwrap();
  let en = targets.iter().find(|target| target.locale == "en").unwrap();
  let zh = targets.iter().find(|target| target.locale == "zh").unwrap();
  assert_eq!(
    (&en.output_path, &en.route_path),
    (
      &"en/opensearch.xml".to_string(),
      &"/指南/en/search".to_string()
    )
  );
  assert_eq!(
    (&zh.output_path, &zh.route_path),
    (&"opensearch.xml".to_string(), &"/指南/search".to_string())
  );
  verify(&plan).unwrap();
  let mut forged = plan.clone();
  forged.routes.retain(|route| {
    !(route.page.kind == "page" && route.page.id == "search" && route.page.locale == "en")
  });
  forged.snapshot_id.clear();
  forged.snapshot_id = format!("{:x}", Sha256::digest(serde_json::to_vec(&forged).unwrap()));
  let error = verify(&forged).expect_err("rehashed plans must retain every actual search target");
  assert!(error.contains("DOCS_SEARCH_PAGE"), "{error}");
}

#[test]
fn optional_features_do_not_reserve_disabled_paths_and_atom_reserves_blog_output() {
  let (root, mut raw) = fixture();
  raw["search"].as_object_mut().unwrap().remove("openSearch");
  fs::create_dir(root.path().join("static")).unwrap();
  fs::write(
    root.path().join("static/opensearch.xml"),
    "user-owned description",
  )
  .unwrap();
  raw["staticDirs"] = json!([{"dir":"static"}]);
  prepare(&parse_config(&raw.to_string(), root.path()).unwrap()).unwrap();
  fs::create_dir(root.path().join("blog")).unwrap();
  fs::create_dir(root.path().join("static/blog")).unwrap();
  fs::write(root.path().join("static/blog/atom.xml"), "must collide").unwrap();
  raw["blog"] = json!({"collections":[{"id":"news","dir":"blog","routeBase":"/blog/"}]});
  raw["seo"] = json!({"atom":{"updated":"2026-10-18T00:00:00Z","author":"Contributors"}});
  let error = prepare(&parse_config(&raw.to_string(), root.path()).unwrap())
    .expect_err("Atom output reserved before compilation");
  assert!(error.contains("DOCS_STATIC_COLLISION"), "{error}");
}

#[test]
fn rust_heads_and_public_missing_pages_share_localized_discovery_links() {
  let (root, mut raw) = fixture();
  fs::write(
    root.path().join("pages/search.tsx"),
    "export const metadata={title:'Search'};export default ()=> <h1>Search</h1>;",
  )
  .unwrap();
  fs::create_dir(root.path().join("blog")).unwrap();
  fs::write(
    root.path().join("blog/post.md"),
    "---\ndate: 2026-01-01\n---\n# Post\n",
  )
  .unwrap();
  raw["seo"] = json!({"atom":{"updated":"2026-10-18T00:00:00Z","author":"Contributors"}});
  raw["blog"] = json!({"collections":[
    {"id":"news-en","dir":"blog","locale":"en","routeBase":"/blog/"},
    {"id":"news-zh","dir":"blog","locale":"zh","routeBase":"/zh/blog/"}]});
  let plan = prepare(&parse_config(&raw.to_string(), root.path()).unwrap()).unwrap();
  for route in &plan.routes {
    let head = serde_json::to_value(&plan.seo[&route.page.key]).unwrap();
    let links = head["discoveryLinks"]
      .as_array()
      .expect("Rust discovery registry");
    assert_eq!(links.len(), 3);
    let prefix = if route.page.locale == "zh" {
      "/guide/zh/"
    } else {
      "/guide/"
    };
    for (kind, suffix) in [
      ("application/rss+xml", "blog/rss.xml"),
      ("application/atom+xml", "blog/atom.xml"),
      ("application/opensearchdescription+xml", "opensearch.xml"),
    ] {
      let expected = format!("https://example.com{prefix}{suffix}");
      assert!(links
        .iter()
        .any(|link| link["type"] == kind && link["href"] == expected));
      assert!(head["html"].as_str().unwrap().contains(&expected));
    }
    let missing = plan
      .not_found
      .iter()
      .find(|page| page.locale == route.page.locale)
      .unwrap();
    assert_eq!(
      serde_json::to_value(missing).unwrap()["discoveryLinks"],
      head["discoveryLinks"]
    );
  }
  let application = farmfe_docs_plan::browser_application(&plan).unwrap();
  let encoded = application
    .lines()
    .find_map(|line| line.strip_prefix("export const notFoundPages = "))
    .unwrap();
  let missing: serde_json::Value =
    serde_json::from_str(encoded.strip_suffix(';').unwrap()).unwrap();
  assert!(missing
    .as_array()
    .unwrap()
    .iter()
    .all(|page| page["discoveryLinks"].as_array().unwrap().len() == 3));
}

#[test]
fn rehashed_plans_cannot_bypass_syndication_configuration_semantics() {
  let (root, mut raw) = fixture();
  raw["search"].as_object_mut().unwrap().remove("openSearch");
  raw["seo"] = json!({"atom":{"updated":"2026-10-18T00:00:00Z","author":"Contributors"}});
  let mut plan = prepare(&parse_config(&raw.to_string(), root.path()).unwrap()).unwrap();
  plan.site.seo.atom.as_mut().unwrap().updated = "not-a-timestamp".into();
  plan.snapshot_id.clear();
  plan.snapshot_id = format!("{:x}", Sha256::digest(serde_json::to_vec(&plan).unwrap()));
  let error = verify(&plan).expect_err("normalized public config must retain semantic validity");
  assert!(error.contains("DOCS_CONFIG_SYNDICATION"), "{error}");
}
