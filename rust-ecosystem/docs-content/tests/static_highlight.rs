use farmfe_docs_content::{
  ContentFormat, HighlightOptions, HighlightRepresentation, compile_content_with_features,
  parse_content,
};

fn static_options() -> HighlightOptions {
  HighlightOptions {
    representation: HighlightRepresentation::StaticHtml,
    ..Default::default()
  }
}

#[test]
fn representation_is_explicit_and_rejects_unknown_options() {
  let default: HighlightOptions = serde_json::from_str("{}").unwrap();
  assert_eq!(default.representation, HighlightRepresentation::Jsx);
  let selected: HighlightOptions =
    serde_json::from_str(r#"{"representation":"static-html"}"#).unwrap();
  assert_eq!(selected.representation, HighlightRepresentation::StaticHtml);
  assert!(serde_json::from_str::<HighlightOptions>(r#"{"representation":"raw"}"#).is_err());
}

#[test]
fn static_fences_keep_metadata_markdown_and_original_source_mapping() {
  let source = "# Example\n\n```text title=\"example.txt\" {1} showLineNumbers\n<script>& Unicode λ\n```\n\nexport const AFTER_FENCE = 42;\n";
  let options = mdxjs::Options {
    filepath: Some("docs/static.mdx".into()),
    jsx: true,
    ..Default::default()
  };
  let static_highlight = static_options();
  // Executable ESM is intentionally rejected by the machine export contract.
  let markdown_source = source.split("export const").next().unwrap();
  let original = parse_content(
    markdown_source,
    &options,
    Some(&HighlightOptions::default()),
    ContentFormat::Mdx,
  )
  .unwrap();
  let compact = parse_content(
    markdown_source,
    &options,
    Some(&static_highlight),
    ContentFormat::Mdx,
  )
  .unwrap();
  assert_eq!(
    original.machine_markdown().unwrap(),
    compact.machine_markdown().unwrap()
  );
  let result =
    compile_content_with_features(source, &options, true, Some(&static_highlight)).unwrap();
  assert!(result.code.contains("dangerouslySetInnerHTML"));
  assert!(result.code.contains("&lt;script&gt;&amp;"));
  for attribute in [
    "data-code",
    "data-title",
    "data-line-numbers",
    "data-highlighted",
    "--code-light",
    "--code-dark",
  ] {
    assert!(result.code.contains(attribute), "{attribute}");
  }
  let map =
    sourcemap::SourceMap::from_slice(result.source_map.as_ref().unwrap().as_bytes()).unwrap();
  let prefix = &result.code[..result.code.find("AFTER_FENCE").unwrap()];
  let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
  let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32;
  let token = map.lookup_token(line, column).unwrap();
  assert_eq!(token.get_src_line(), 6);
  assert_eq!(token.get_source(), Some("docs/static.mdx"));
  assert_eq!(map.get_source_contents(0), Some(source));
}

#[test]
fn static_fences_work_with_lowered_jsx_and_leave_authored_elements_alone() {
  let options = mdxjs::Options {
    filepath: Some("static.mdx".into()),
    ..Default::default()
  };
  let result = compile_content_with_features(
    "```text\nsafe\n```\n\n<code>Authored</code>\n",
    &options,
    false,
    Some(&static_options()),
  )
  .unwrap();
  assert!(result.code.contains("dangerouslySetInnerHTML"));
  assert!(result.code.contains("Authored"));
  assert_eq!(result.code.matches("dangerouslySetInnerHTML").count(), 1);
}
