use farmfe_docs_content::{ContentFormat, emit_content, parse_content};

#[test]
fn resource_imports_bind_jsx_without_capturing_user_identifiers_or_rewriting_prose() {
  let source = "export const _farmDocsAsset0 = 'kept';\n\n# Assets\n\n![Image](./image.svg)\n\nLiteral /guide/assets/docs/test.svg?raw#mark\n";
  let mut parsed = parse_content(
    source,
    &mdxjs::Options {
      filepath: Some("asset.mdx".into()),
      jsx: true,
      ..Default::default()
    },
    None,
    ContentFormat::Mdx,
  )
  .unwrap();
  let url = "/guide/assets/docs/test.svg?raw#mark";
  parsed.rewrite_references(|_| Ok(url.into())).unwrap();
  parsed.bind_resource(url.into(), "docs:asset:test".into(), "?raw#mark".into());
  let emitted = emit_content(parsed, true).unwrap();
  assert!(
    emitted
      .code
      .contains("import _farmDocsAsset1 from \"docs:asset:test\""),
    "{}",
    emitted.code
  );
  assert!(
    emitted
      .code
      .contains("src={_farmDocsAsset1 + \"?raw#mark\"}"),
    "{}",
    emitted.code
  );
  assert!(
    emitted
      .code
      .contains("Literal /guide/assets/docs/test.svg?raw#mark")
  );
  let map: serde_json::Value = serde_json::from_str(emitted.source_map.as_ref().unwrap()).unwrap();
  assert_eq!(map["sourcesContent"][0], source);
}

#[test]
fn saved_ast_exposes_original_references_and_emits_rewritten_links_without_reparsing() {
  let source =
    "---\ntitle: Links\n---\n\n# 标题\n\n[Next](./next.md#anchor)\n\n![Image](./image.png)\n";
  let options = mdxjs::Options {
    filepath: Some("intro.md".into()),
    jsx: true,
    ..Default::default()
  };
  let parsed = parse_content(source, &options, None, ContentFormat::Auto).unwrap();
  assert_eq!(parsed.frontmatter["title"], "Links");
  assert_eq!(parsed.toc[0].title, "标题");
  let references = parsed.references();
  assert_eq!(references.len(), 2);
  assert_eq!(references[0].target, "./next.md#anchor");
  assert_eq!((references[0].line, references[0].column), (7, 1));
  let mut rewritten = parsed.clone();
  rewritten
    .rewrite_references(|reference| {
      Ok(if reference.target.starts_with("./next") {
        "/guide/next/#anchor".into()
      } else {
        reference.target.clone()
      })
    })
    .unwrap();
  let emitted = emit_content(rewritten, true).unwrap();
  assert!(emitted.code.contains("/guide/next/#anchor"));
  assert!(emitted.source_map.as_ref().unwrap().contains("intro.md"));
  let untouched = emit_content(parsed, true).unwrap();
  assert!(untouched.code.contains("./next.md#anchor"));
}

#[test]
fn reference_columns_count_utf16_in_original_unicode_source() {
  let parsed = parse_content(
    "😀 [Next](./next.md)\n",
    &mdxjs::Options {
      filepath: Some("intro.md".into()),
      ..Default::default()
    },
    None,
    ContentFormat::Auto,
  )
  .unwrap();
  assert_eq!(
    (parsed.references()[0].line, parsed.references()[0].column),
    (1, 4)
  );
}

#[test]
fn raw_html_links_keep_their_real_source_location_and_static_ids() {
  let source = "# Intro\n\n<div>\n  <a href=\"./next.md\">Next</a>\n  <span id=\"raw-anchor\">Target</span>\n</div>\n";
  let parsed = parse_content(
    source,
    &mdxjs::Options {
      filepath: Some("intro.md".into()),
      ..Default::default()
    },
    None,
    ContentFormat::Auto,
  )
  .unwrap();
  let references = parsed.references();
  assert_eq!((references[0].line, references[0].column), (4, 3));
  assert!(parsed.anchors().unwrap().contains("raw-anchor"));
}
