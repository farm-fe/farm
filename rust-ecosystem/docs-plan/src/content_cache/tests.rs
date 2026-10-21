use super::*;
use farmfe_docs_content::{emit_content, parse_content};

fn parsed(source: &str) -> ParsedContent {
  parse_content(source, &mdxjs::Options::default(), None, ContentFormat::Mdx).unwrap()
}

#[test]
fn unchanged_bytes_skip_the_parser_and_return_independent_original_trees() {
  let mut cache = ContentCache::default();
  cache.begin();
  let mut first = cache
    .parse("/one.mdx", ContentFormat::Mdx, "hash", || {
      Ok(parsed(":::note\nBody\n:::"))
    })
    .unwrap();
  first.apply_admonition_labels(&[("note".to_owned(), "Changed".to_owned())].into());
  assert!(emit_content(first, false).unwrap().code.contains("Changed"));
  let original = cache
    .parse("/one.mdx", ContentFormat::Mdx, "hash", || {
      panic!("unchanged bytes must not parse again")
    })
    .unwrap();
  let emitted = emit_content(original, false).unwrap().code;
  assert!(emitted.contains("note"));
  assert!(!emitted.contains("Changed"));
}

#[test]
fn content_format_and_filepath_are_separate_parse_identities() {
  let mut cache = ContentCache::default();
  cache.begin();
  for (path, format, hash, text) in [
    ("/one.md", ContentFormat::Auto, "old", "# Old"),
    ("/one.md", ContentFormat::Auto, "new", "# New"),
    ("/one.md", ContentFormat::Mdx, "new", "# Format"),
    ("/two.md", ContentFormat::Mdx, "new", "# Path"),
  ] {
    let mut called = false;
    let content = cache
      .parse(path, format, hash, || {
        called = true;
        Ok(parsed(text))
      })
      .unwrap();
    assert!(called, "changed parse identity must invoke the parser");
    assert!(emit_content(content, false)
      .unwrap()
      .code
      .contains(text.trim_start_matches("# ")));
  }
}

#[test]
fn a_failed_new_parse_never_returns_or_replaces_the_old_tree() {
  let mut cache = ContentCache::default();
  cache.begin();
  cache
    .parse("/one.mdx", ContentFormat::Mdx, "old", || {
      Ok(parsed("# Working"))
    })
    .unwrap();
  let error = cache
    .parse("/one.mdx", ContentFormat::Mdx, "bad", || {
      Err("invalid current source".into())
    })
    .err()
    .unwrap();
  assert_eq!(error, "invalid current source");
  let restored = cache
    .parse("/one.mdx", ContentFormat::Mdx, "old", || {
      panic!("last successfully parsed bytes remain reusable")
    })
    .unwrap();
  assert!(emit_content(restored, false)
    .unwrap()
    .code
    .contains("Working"));
}

#[test]
fn only_a_completed_generation_prunes_no_longer_seen_documents() {
  let mut cache = ContentCache::default();
  cache.begin();
  for path in ["/one.mdx", "/two.mdx"] {
    cache
      .parse(path, ContentFormat::Mdx, "hash", || Ok(parsed("# Page")))
      .unwrap();
  }
  cache.commit();
  cache.begin();
  cache
    .parse("/one.mdx", ContentFormat::Mdx, "bad", || Err("bad".into()))
    .err()
    .unwrap();
  // A failed prepare does not call commit; both previous documents stay available.
  cache.begin();
  cache
    .parse("/one.mdx", ContentFormat::Mdx, "hash", || {
      panic!("reuse retained page")
    })
    .unwrap();
  cache.commit();
  let mut called = false;
  cache
    .parse("/two.mdx", ContentFormat::Mdx, "hash", || {
      called = true;
      Ok(parsed("# Recreated"))
    })
    .unwrap();
  assert!(
    called,
    "a removed source must have been evicted after successful prepare"
  );
}
