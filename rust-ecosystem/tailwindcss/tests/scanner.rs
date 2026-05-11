use farmfe_ecosystem_tailwindcss::scanner::extract_candidates;
use farmfe_testing_helpers::assert_snapshot;
use std::fs;
use std::path::PathBuf;

fn fixture_path(rel: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("fixtures")
    .join(rel)
}

// ── Snapshot tests ───────────────────────────────────────────────────────────

#[test]
fn scanner_fixture_outputs_are_snapshotted() {
  let html = fs::read_to_string(fixture_path("scanner/html.html")).unwrap();
  let jsx = fs::read_to_string(fixture_path("scanner/jsx.tsx")).unwrap();
  let css = fs::read_to_string(fixture_path("scanner/styles.css")).unwrap();

  let html_candidates = extract_candidates(&html);
  let jsx_candidates = extract_candidates(&jsx);
  let css_candidates = extract_candidates(&css);

  let output = format!(
    "html candidates:\n{}\n\njsx candidates:\n{}\n\ncss candidates:\n{}",
    html_candidates.join("\n"),
    jsx_candidates.join("\n"),
    css_candidates.join("\n"),
  );

  assert_snapshot!(output);
}

// ── Unit tests ───────────────────────────────────────────────────────────────

#[test]
fn scanner_extracts_class_names_from_html_attribute() {
  let content = r#"<div class="flex items-center bg-blue-500 p-4">"#;
  let candidates = extract_candidates(content);
  assert!(candidates.contains(&"flex".to_string()), "should contain 'flex'");
  assert!(
    candidates.contains(&"items-center".to_string()),
    "should contain 'items-center'"
  );
  assert!(
    candidates.contains(&"bg-blue-500".to_string()),
    "should contain 'bg-blue-500'"
  );
  assert!(
    candidates.contains(&"p-4".to_string()),
    "should contain 'p-4'"
  );
}

#[test]
fn scanner_extracts_class_names_from_jsx_classname() {
  let content = r#"<button className="bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2">"#;
  let candidates = extract_candidates(content);
  assert!(candidates.contains(&"bg-indigo-600".to_string()));
  assert!(candidates.contains(&"hover:bg-indigo-700".to_string()));
  assert!(candidates.contains(&"text-white".to_string()));
}

#[test]
fn scanner_extracts_responsive_and_state_variants() {
  let content =
    r#"class="sm:flex md:grid lg:block xl:hidden 2xl:inline focus:outline-none active:scale-95""#;
  let candidates = extract_candidates(content);
  assert!(candidates.contains(&"sm:flex".to_string()));
  assert!(candidates.contains(&"md:grid".to_string()));
  assert!(candidates.contains(&"focus:outline-none".to_string()));
  assert!(candidates.contains(&"active:scale-95".to_string()));
}

#[test]
fn scanner_deduplicates_repeated_candidates() {
  let content = "flex flex flex items-center flex";
  let candidates = extract_candidates(content);
  let flex_count = candidates.iter().filter(|c| c.as_str() == "flex").count();
  assert_eq!(flex_count, 1, "duplicate candidates should be removed");
}

#[test]
fn scanner_skips_url_protocols() {
  let content = r#"src="https://example.com/image.png" class="w-full""#;
  let candidates = extract_candidates(content);
  assert!(
    !candidates.iter().any(|c| c.contains("://")),
    "should not include URL protocols"
  );
  assert!(candidates.contains(&"w-full".to_string()));
}

#[test]
fn scanner_returns_empty_for_blank_content() {
  let candidates = extract_candidates("");
  assert!(candidates.is_empty());
  let candidates2 = extract_candidates("   \n\t  ");
  assert!(candidates2.is_empty());
}
