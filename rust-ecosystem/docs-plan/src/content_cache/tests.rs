use super::*;
use farmfe_docs_content::{emit_content, parse_content};

fn parsed(source: &str) -> ParsedContent {
  parse_content(source, &mdxjs::Options::default(), None, ContentFormat::Mdx).unwrap()
}

#[test]
fn presentation_options_are_part_of_cache_identity() {
  let mut cache = ContentCache::default();
  cache.begin();
  for representation in ["jsx", "static-html"] {
    let mut called = false;
    cache
      .parse_with_options(
        "/one.md",
        ContentFormat::Markdown,
        "same",
        representation,
        || {
          called = true;
          Ok(parsed("# Same"))
        },
      )
      .unwrap();
    assert!(
      called,
      "changed representation must not reuse another syntax tree"
    );
  }
  cache
    .parse_with_options(
      "/one.md",
      ContentFormat::Markdown,
      "same",
      "static-html",
      || panic!("same representation and bytes should reuse cached tree"),
    )
    .unwrap();
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
    assert!(
      emit_content(content, false)
        .unwrap()
        .code
        .contains(text.trim_start_matches("# "))
    );
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
  assert!(
    emit_content(restored, false)
      .unwrap()
      .code
      .contains("Working")
  );
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

#[test]
fn shared_cold_lookups_parse_once_and_release_cache_ownership() {
  let mut cache = ContentCache::transient();
  cache.begin();
  let first = cache
    .shared_parse_with_options(
      "/one.mdx",
      ContentFormat::Mdx,
      "hash",
      "static-html",
      || Ok(parsed("# Shared")),
    )
    .unwrap();
  let second = cache
    .shared_parse_with_options(
      "/one.mdx",
      ContentFormat::Mdx,
      "hash",
      "static-html",
      || panic!("same bytes/options must parse once"),
    )
    .unwrap();
  assert_eq!(cache.parse_calls(), 1);
  assert!(Arc::ptr_eq(&first, &second));
  assert_eq!(
    Arc::strong_count(&first),
    3,
    "two consumers plus cache owner"
  );
  cache.release_transient_originals();
  assert_eq!(Arc::strong_count(&first), 2);
  drop(first);
  assert!(
    Arc::try_unwrap(second).is_ok(),
    "last cold owner can move without clone"
  );
}

#[test]
fn persistent_original_remains_shared_and_unmodified_after_route_consumption() {
  let mut cache = ContentCache::default();
  cache.begin();
  let original = cache
    .shared_parse_with_options("/one.mdx", ContentFormat::Mdx, "hash", "jsx", || {
      Ok(parsed(":::note\nOriginal\n:::"))
    })
    .unwrap();
  cache.release_transient_originals();
  assert_eq!(
    Arc::strong_count(&original),
    2,
    "warm cache retains its original"
  );
  let mut private = Arc::unwrap_or_clone(original);
  private.apply_admonition_labels(&[("note".into(), "Private label".into())].into());
  assert!(
    emit_content(private, false)
      .unwrap()
      .code
      .contains("Private label")
  );
  cache.commit();
  cache.begin();
  let restored = cache
    .shared_parse_with_options("/one.mdx", ContentFormat::Mdx, "hash", "jsx", || {
      panic!("warm cache must not parse again")
    })
    .unwrap();
  assert_eq!(cache.parse_calls(), 1);
  assert!(
    !emit_content(Arc::unwrap_or_clone(restored), false)
      .unwrap()
      .code
      .contains("Private label")
  );
}

#[test]
fn palette_and_custom_token_bytes_are_part_of_content_cache_identity() {
  let mut cache = ContentCache::default();
  cache.begin();
  let configurations = [
    serde_json::json!({"light":"vscode-light","dark":"vscode-dark"}),
    serde_json::json!({"light":"farm-light","dark":"farm-dark"}),
    serde_json::json!({"light":"farm-light","dark":"farm-dark","tokens":{"dark":{"type":"#aabbcc"}}}),
  ];
  for value in configurations {
    let options: farmfe_docs_content::HighlightOptions = serde_json::from_value(value).unwrap();
    let identity = serde_json::to_string(&options).unwrap();
    cache
      .parse_with_options(
        "/same.md",
        ContentFormat::Markdown,
        "same-bytes",
        &identity,
        || Ok(parsed("# Same")),
      )
      .unwrap();
  }
  assert_eq!(cache.parse_calls(), 3);
}
