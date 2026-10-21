use farmfe_docs_config::parse_config;
use farmfe_docs_plan::prepare;
use serde_json::json;
use std::fs;

fn diagnostic(kind: &str, source: &str) -> String {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("content")).unwrap();
  fs::write(root.path().join("content/index.md"), source).unwrap();
  let mut site = json!({"title":"Metadata","url":"https://example.com"});
  site[kind] = if kind == "pages" {
    json!({"dir":"content"})
  } else {
    json!({"collections":[{"id":"main","dir":"content","routeBase":"/"}]})
  };
  let config = parse_config(&site.to_string(), root.path()).unwrap();
  prepare(&config).unwrap_err()
}

#[test]
fn scalar_type_diagnostic_uses_value_position_in_flow_yaml_after_emoji() {
  let error = diagnostic("docs", "---\n{note: \"😀\", title: []}\n---\n# Body\n");
  assert!(error.contains("DOCS_FRONTMATTER"), "{error}");
  assert!(error.contains("index.md:2:21"), "{error}");
  assert!(error.contains("title must be a nonempty string"), "{error}");
}

#[test]
fn slug_and_layout_semantics_point_to_original_yaml_values() {
  for (source, position, code) in [
    (
      "---\ntitle: Home\nslug: '../escape'\n---\n# Home\n",
      "index.md:3:7",
      "DOCS_ROUTE_PATH",
    ),
    (
      "---\nlayout: unknown\n---\n# Home\n",
      "index.md:2:9",
      "DOCS_FRONTMATTER",
    ),
  ] {
    let error = diagnostic("pages", source);
    assert!(error.contains(position) && error.contains(code), "{error}");
  }
}

#[test]
fn blog_date_and_nested_author_url_have_exact_positions() {
  for (source, position, code) in [
    (
      "---\ntitle: Post\ndate: 2025-02-30\n---\n# Post\n",
      "index.md:3:7",
      "DOCS_BLOG_DATE",
    ),
    (
      "---\ntitle: Post\nauthors:\n  - name: Person\n    url: javascript:bad\n---\n# Post\n",
      "index.md:5:10",
      "DOCS_BLOG_AUTHOR",
    ),
  ] {
    let error = diagnostic("blog", source);
    assert!(error.contains(position) && error.contains(code), "{error}");
  }
}

#[test]
fn alias_value_error_points_to_alias_use_without_text_search() {
  let error = diagnostic(
    "docs",
    "---\nbad: &invalid []\ntitle: *invalid\n---\n# Home\n",
  );
  assert!(error.contains("index.md:3:8"), "{error}");
}

#[test]
fn malformed_yaml_reports_source_file_line_and_column() {
  let error = diagnostic("docs", "---\ntitle: [broken\n---\n# Home\n");
  assert!(error.contains("DOCS_FRONTMATTER"), "{error}");
  assert!(error.contains("index.md:3:1"), "{error}");
}

#[test]
fn singleton_and_aliased_author_records_locate_the_offending_property() {
  for (source, position) in [
    (
      "---\nauthors: {name: Person, url: 'javascript:bad'}\n---\n# Post\n",
      "index.md:2:30",
    ),
    (
      "---\nbad: &person {name: Person, url: 'javascript:bad'}\nauthors: [*person]\n---\n# Post\n",
      "index.md:3:11",
    ),
    (
      "---\nauthors:\n  - name: Person\n    url: 42\n---\n# Post\n",
      "index.md:4:10",
    ),
  ] {
    let error = diagnostic("blog", source);
    assert!(
      error.contains(position) && error.contains("DOCS_BLOG_AUTHOR"),
      "{error}"
    );
  }
}

#[test]
fn crlf_and_repeated_nested_title_do_not_shift_top_level_value_location() {
  let error = diagnostic(
    "docs",
    "---\nnotes:\n  title: correct\ntitle: []\n---\n# Home\n"
      .replace('\n', "\r\n")
      .as_str(),
  );
  assert!(error.contains("index.md:4:8"), "{error}");
}
