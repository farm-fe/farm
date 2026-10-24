use farmfe_docs_config::parse_config;
use farmfe_docs_plan::{prepare, DocPlan};
use farmfe_docs_seo::generate;
use quick_xml::{events::Event, Reader};
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs};

fn fixture(blog: bool) -> (tempfile::TempDir, Value) {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("docs")).unwrap();
  fs::write(root.path().join("docs/index.md"), "# Intro\n").unwrap();
  fs::write(root.path().join("docs/入门.md"), "# 中文\n").unwrap();
  let mut raw = json!({"title":"SEO & 中文","url":"https://example.com","basePath":"/guide/",
    "docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}});
  if blog {
    fs::create_dir(root.path().join("blog")).unwrap();
    fs::write(
      root.path().join("blog/new.md"),
      "---\ntitle: New\ndate: '2025-01-02T03:04:05+08:00'\n---\n# New\n",
    )
    .unwrap();
    fs::write(root.path().join("blog/old.md"), "---\ntitle: 'Older & <release>'\ndate: 2024-02-29\ndescription: 'Quotes & <b>text</b>'\ntags: [Rust]\n---\n# Old\n").unwrap();
    fs::write(root.path().join("blog/undated.md"), "# Undated\n").unwrap();
    raw["blog"] = json!({"collections":[{"id":"news","dir":"blog","routeBase":"/blog/"}]});
  }
  (root, raw)
}

fn plan(root: &std::path::Path, raw: &Value) -> DocPlan {
  prepare(&parse_config(&raw.to_string(), root).unwrap()).unwrap()
}

fn values(bytes: &[u8], name: &str) -> Vec<String> {
  let mut reader = Reader::from_reader(bytes);
  let mut result = vec![];
  loop {
    match reader.read_event().expect("well-formed XML") {
      Event::Start(element) if element.name().as_ref() == name => {
        let text = reader.read_text(element.name()).unwrap();
        result.push(quick_xml::escape::unescape(&text).unwrap().into_owned());
      }
      Event::Eof => break,
      _ => {}
    }
  }
  result
}

#[test]
fn atom_is_opt_in_and_retains_undated_entries_without_fabricating_publication_dates() {
  let (root, mut raw) = fixture(true);
  raw["seo"] = json!({"atom":{"updated":"2026-10-18T00:00:00Z","author":"Docs & contributors"}});
  let prepared = plan(root.path(), &raw);
  let resources = generate(&prepared).unwrap();
  let bytes = resources
    .get("blog/atom.xml")
    .expect("enabled Atom feed must exist");
  assert_eq!(
    values(bytes, "title"),
    ["SEO & 中文 — news", "New", "Older & <release>", "Undated"]
  );
  assert_eq!(values(bytes, "published").len(), 2);
  assert_eq!(values(bytes, "updated").len(), 4);
  assert_eq!(values(bytes, "name"), ["Docs & contributors"]);
  assert_eq!(generate(&prepared).unwrap(), resources);
}

#[test]
fn opensearch_templates_target_rust_pages_in_each_language_under_the_mount() {
  let (root, mut raw) = fixture(false);
  fs::create_dir(root.path().join("pages")).unwrap();
  fs::write(
    root.path().join("pages/search.tsx"),
    "export const metadata={title:'Search'};export default ()=> <h1>Search</h1>;",
  )
  .unwrap();
  raw["pages"] = json!({"dir":"pages"});
  raw["locales"] = json!({"default":"en","items":["en","zh"]});
  raw["search"] = json!({"provider":"algolia","options":{"appId":"APP","apiKey":"public","indexName":"docs"},"openSearch":{"pageId":"search","shortName":"Docs 中文","description":"Search & discover"}});
  let resources = generate(&plan(root.path(), &raw)).unwrap();
  for (output, target) in [
    (
      "opensearch.xml",
      "https://example.com/guide/search/?q={searchTerms}",
    ),
    (
      "zh/opensearch.xml",
      "https://example.com/guide/zh/search/?q={searchTerms}",
    ),
  ] {
    let bytes = resources
      .get(output)
      .expect("localized OpenSearch resource");
    assert_eq!(values(bytes, "ShortName"), ["Docs 中文"]);
    assert_eq!(values(bytes, "Description"), ["Search & discover"]);
    let mut reader = Reader::from_reader(bytes.as_slice());
    let mut templates = vec![];
    loop {
      match reader.read_event().unwrap() {
        Event::Empty(element) if element.name().as_ref() == "Url" => {
          for attribute in element.attributes() {
            let attribute = attribute.unwrap();
            if attribute.key.as_ref() == "template" {
              templates.push(
                attribute
                  .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                  .unwrap()
                  .into_owned(),
              );
            }
          }
        }
        Event::Eof => break,
        _ => {}
      }
    }
    assert!(
      templates.contains(&target.to_string()),
      "{output}: {templates:?}"
    );
  }
}

#[test]
fn atom_maximum_date_uses_instants_and_retains_boundary_offsets_and_real_authors() {
  let (root, mut raw) = fixture(true);
  raw["seo"] = json!({"atom":{"updated":"2020-01-01T00:00:00Z","author":"Feed owner"}});
  fs::write(root.path().join("blog/new.md"), "---\ntitle: New\ndate: '2025-01-02T03:04:05+08:00'\nauthors: [{name: 'Actual author', url: 'https://author.example'}]\n---\n# New\n").unwrap();
  let resources = generate(&plan(root.path(), &raw)).unwrap();
  let updated = values(&resources["blog/atom.xml"], "updated");
  assert_eq!(
    chrono::DateTime::parse_from_rfc3339(&updated[0]).unwrap(),
    chrono::DateTime::parse_from_rfc3339("2025-01-02T03:04:05+08:00").unwrap()
  );
  assert_eq!(
    values(&resources["blog/atom.xml"], "name"),
    ["Feed owner", "Actual author"]
  );
  for date in ["0001-01-01T00:00:00+23:59", "9999-12-31T23:59:59-23:59"] {
    fs::write(
      root.path().join("blog/new.md"),
      format!("---\ntitle: Boundary\ndate: '{date}'\n---\n# Boundary\n"),
    )
    .unwrap();
    let resources = generate(&plan(root.path(), &raw)).unwrap();
    let dates = values(&resources["blog/atom.xml"], "published");
    let expected = chrono::DateTime::parse_from_rfc3339(date).unwrap();
    assert!(dates
      .iter()
      .any(|date| chrono::DateTime::parse_from_rfc3339(date).unwrap() == expected));
  }
}

#[test]
fn empty_atom_has_declared_update_and_disabled_atom_emits_no_extra_feed() {
  let (root, mut raw) = fixture(false);
  fs::create_dir(root.path().join("empty")).unwrap();
  raw["blog"] = json!({"collections":[{"id":"empty","dir":"empty","routeBase":"/news/"}]});
  raw["seo"] = json!({"atom":{"updated":"2020-01-01T00:00:00+05:30","author":"Feed owner"}});
  let resources = generate(&plan(root.path(), &raw)).unwrap();
  assert_eq!(
    values(&resources["news/atom.xml"], "updated"),
    ["2020-01-01T00:00:00+05:30"]
  );
  assert!(values(&resources["news/atom.xml"], "published").is_empty());
  raw.as_object_mut().unwrap().remove("seo");
  let resources = generate(&plan(root.path(), &raw)).unwrap();
  assert!(!resources.contains_key("news/atom.xml"));
  assert!(resources.contains_key("news/rss.xml"));
}

#[test]
fn sitemap_and_robots_use_real_canonical_urls_and_encoded_unicode_under_one_mount() {
  let (root, raw) = fixture(false);
  let prepared = plan(root.path(), &raw);
  let resources = generate(&prepared).unwrap();
  assert_eq!(
    resources.keys().map(String::as_str).collect::<Vec<_>>(),
    ["robots.txt", "sitemap.xml"]
  );
  let urls = values(&resources["sitemap.xml"], "loc");
  assert_eq!(urls.len(), prepared.routes.len());
  assert!(urls.contains(&"https://example.com/guide/docs/%E5%85%A5%E9%97%A8/".into()));
  assert!(urls.windows(2).all(|pair| pair[0] < pair[1]));
  assert_eq!(
    String::from_utf8(resources["robots.txt"].clone()).unwrap(),
    "User-agent: *\nAllow: /\nSitemap: https://example.com/guide/sitemap.xml\n"
  );
}

#[test]
fn rss_contains_only_articles_with_stable_dates_and_proper_xml_entities() {
  let (root, raw) = fixture(true);
  let resources = generate(&plan(root.path(), &raw)).unwrap();
  let rss = &resources["blog/rss.xml"];
  assert_eq!(
    values(rss, "title"),
    ["SEO & 中文 — news", "New", "Older & <release>", "Undated"]
  );
  assert_eq!(
    values(rss, "pubDate"),
    [
      "Wed, 1 Jan 2025 19:04:05 +0000",
      "Thu, 29 Feb 2024 00:00:00 +0000"
    ]
  );
  assert_eq!(values(rss, "category"), ["Rust"]);
  assert!(values(rss, "description").contains(&"Quotes & <b>text</b>".into()));
  let links = values(rss, "guid");
  assert_eq!(links.len(), 3);
  assert_eq!(links.iter().collect::<BTreeSet<_>>().len(), 3);
  assert!(links
    .iter()
    .all(|url| url.starts_with("https://example.com/guide/blog/")));
}

#[test]
fn xml_bytes_are_deterministic_across_repeated_runs_and_different_source_roots() {
  let (first, raw) = fixture(true);
  let (second, _) = fixture(true);
  let prepared = plan(first.path(), &raw);
  let resources = generate(&prepared).unwrap();
  assert_eq!(resources, generate(&prepared).unwrap());
  assert_eq!(resources, generate(&plan(second.path(), &raw)).unwrap());
}

#[test]
fn generated_names_reject_static_case_and_ancestor_collisions_without_overwrite() {
  for name in ["SITEMAP.xml", "robots.txt/child", "blog/rss.xml"] {
    let (root, mut raw) = fixture(true);
    let file = root.path().join("static").join(name);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "user content").unwrap();
    raw["staticDirs"] = json!([{"dir":"static"}]);
    let prepared = plan(root.path(), &raw);
    let error = generate(&prepared).unwrap_err();
    assert!(error.contains("COLLISION"), "{name}: {error}");
    assert_eq!(fs::read_to_string(file).unwrap(), "user content");
  }
}

#[test]
fn changed_sources_and_forged_plan_are_rejected_before_generating_resources() {
  let (root, raw) = fixture(true);
  let prepared = plan(root.path(), &raw);
  let mut forged = prepared.clone();
  forged.snapshot_id = "0".repeat(64);
  assert!(generate(&forged).unwrap_err().contains("DOCS_PLAN_HASH"));
  fs::write(root.path().join("docs/index.md"), "# Changed\n").unwrap();
  assert!(generate(&prepared).unwrap_err().contains("SNAPSHOT"));
}

#[test]
fn canonical_aliases_have_one_sitemap_entry_for_both_trailing_slash_policies() {
  for policy in ["always", "never"] {
    let (root, mut raw) = fixture(false);
    raw["trailingSlash"] = json!(policy);
    raw["canonicalOverrides"] = json!([
      {"path":"/guide/docs/","canonicalPath":"/guide/docs/"},
      {"path":"/guide/docs/入门/","canonicalPath":"/guide/docs/"}
    ]);
    let resources = generate(&plan(root.path(), &raw)).unwrap();
    assert_eq!(
      values(&resources["sitemap.xml"], "loc"),
      ["https://example.com/guide/docs/"]
    );
  }
}

#[test]
fn fallback_indexing_changes_sitemap_but_never_duplicates_rss_articles() {
  let (root, mut raw) = fixture(true);
  fs::create_dir(root.path().join("zh-blog")).unwrap();
  raw["locales"] = json!({"default":"en","items":["en","zh"]});
  raw["blog"]["collections"][0]["group"] = json!("news");
  raw["blog"]["collections"].as_array_mut().unwrap().push(json!({
    "id":"zh-news","group":"news","dir":"zh-blog","locale":"zh","routeBase":"/zh/blog/","fallbackLocale":"en"
  }));
  let noindex = plan(root.path(), &raw);
  let before = generate(&noindex).unwrap();
  let excluded: Vec<_> = noindex
    .routes
    .iter()
    .filter(|route| route.page.fallback_source.is_some())
    .map(|route| noindex.seo[&route.page.key].canonical_url.clone())
    .collect();
  assert_eq!(excluded.len(), 3);
  assert!(excluded
    .iter()
    .all(|url| !values(&before["sitemap.xml"], "loc").contains(url)));
  raw["seo"] = json!({"fallbackIndexing":"index"});
  let indexed = generate(&plan(root.path(), &raw)).unwrap();
  assert!(excluded
    .iter()
    .all(|url| values(&indexed["sitemap.xml"], "loc").contains(url)));
  assert!(values(&indexed["zh/blog/rss.xml"], "guid").is_empty());
  assert_eq!(values(&indexed["blog/rss.xml"], "guid").len(), 3);
}

#[test]
fn xml_forbidden_metadata_characters_fail_instead_of_producing_invalid_rss() {
  for title in ["Bad\u{1}title", "Bad\u{FFFE}title"] {
    let (root, mut raw) = fixture(true);
    raw["title"] = json!(title);
    let error = generate(&plan(root.path(), &raw)).unwrap_err();
    assert!(error.contains("DOCS_SEO_XML"), "{error}");
  }
}

fn boundary_date_keeps_original_offset(value: &str, offset: &str) {
  let (root, raw) = fixture(true);
  fs::write(
    root.path().join("blog/new.md"),
    format!("---\ndate: '{value}'\n---\n# Boundary\n"),
  )
  .unwrap();
  let resources =
    generate(&plan(root.path(), &raw)).expect("valid local calendar date must build without panic");
  let dates = values(&resources["blog/rss.xml"], "pubDate");
  let serialized = dates
    .iter()
    .find(|date| date.ends_with(offset))
    .expect("UTC rollover must retain the original representable offset");
  let actual = chrono::DateTime::parse_from_rfc2822(serialized).unwrap();
  let expected = chrono::DateTime::parse_from_rfc3339(value).unwrap();
  assert_eq!(actual.timestamp(), expected.timestamp());
}

#[test]
fn rss_utc_upper_year_rollover_preserves_the_valid_original_offset() {
  boundary_date_keeps_original_offset("9999-12-31T23:59:59-23:59", "-2359");
}

#[test]
fn rss_utc_lower_year_rollover_preserves_the_valid_original_offset() {
  boundary_date_keeps_original_offset("0001-01-01T00:00:00+23:59", "+2359");
}
