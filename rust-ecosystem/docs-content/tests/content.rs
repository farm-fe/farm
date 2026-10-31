use farmfe_docs_content::compile_content;
use farmfe_docs_content::compile_content_with_map;

#[test]
fn markdown_template_and_indented_directives_preserve_authored_content() {
  let options = mdxjs::Options {
    filepath: Some("boundaries.md".into()),
    ..Default::default()
  };
  let template =
    compile_content("<template><b>Template sentinel</b></template>\n", &options).unwrap();
  assert!(
    template.code.contains("Template sentinel"),
    "template content was dropped"
  );
  let indented =
    compile_content("    :::note Literal\n    body\n", &options).expect("indented Markdown code");
  assert!(indented.code.contains(":::note Literal"));
  assert!(!indented.code.contains("farm-admonition-note"));
}

#[test]
fn explicit_mdx_format_preserves_legacy_md_expressions() {
  let options = mdxjs::Options {
    filepath: Some("legacy.md".into()),
    ..Default::default()
  };
  let source = "export const answer = 42;\n\nValue: {answer}\n";
  let result = farmfe_docs_content::compile_content_with_format(
    source,
    &options,
    false,
    None,
    farmfe_docs_content::ContentFormat::Mdx,
  )
  .unwrap();
  assert!(result
    .code
    .lines()
    .any(|line| line.starts_with("export const answer = 42")));
}

#[test]
fn plain_markdown_supports_html_void_tags_and_literal_braces() {
  let options = mdxjs::Options {
    filepath: Some("plain.md".into()),
    ..Default::default()
  };
  let source = "# HTML\n\nLiteral {notJavaScript}.\n\nBefore <br> after.\n\n<div class=\"notice\" style=\"color: red\"><input disabled>Raw &amp; HTML</div>\n";
  let result = compile_content(source, &options).expect("CommonMark HTML is valid Markdown");
  assert!(result.code.contains("notJavaScript"));
  assert!(result.code.contains("notice"));
  assert!(result.code.contains("disabled"));
}

#[test]
fn yaml_diagnostic_uses_original_document_line_and_snippet() {
  let error = compile_content("---\ntitle: *missing\n---\n", &Default::default())
    .err()
    .expect("invalid alias");
  assert!(error.contains("<content>:2:8:"), "{error}");
  assert!(error.contains("title: *missing"), "{error}");
}

#[test]
fn empty_and_long_fenced_admonitions_keep_structure() {
  for source in [
    ":::note Empty\n:::\n",
    "::::note Outer\n\n:::tip Inner\n:::\n\n::::\n",
  ] {
    let result = compile_content(source, &Default::default()).expect("valid admonitions");
    assert!(result.code.contains("farm-admonition-note"));
    if source.contains("Inner") {
      assert!(result.code.contains("farm-admonition-tip"));
    }
  }
  assert!(compile_content("::::note\nbody\n:::\n", &Default::default()).is_err());
}

#[test]
fn fenced_directive_text_is_not_interpreted_as_an_admonition() {
  let result = compile_content(
    "# Demo\n\n```md\n:::warning Literal\nbody\n:::\n```\n",
    &Default::default(),
  )
  .expect("valid fenced content");
  assert!(result.code.contains(":::warning Literal"));
  assert!(!result.code.contains("farm-admonition-warning"));
  assert_eq!(result.toc[0].id, "demo");
}

#[test]
fn nested_admonitions_keep_their_types_and_titles() {
  let result = compile_content(
    ":::note Outer\n\nFirst\n\n:::warning Inner\n\nSecond\n\n:::\n\n:::\n",
    &Default::default(),
  )
  .expect("valid nested directives");
  assert!(result.code.contains("farm-admonition-note"));
  assert!(result.code.contains("farm-admonition-warning"));
  assert!(result.code.contains("Outer"));
  assert!(result.code.contains("Inner"));
}

#[test]
fn a_titled_admonition_does_not_wrap_each_descendant_list_item_again() {
  let source = "# Start\n\n:::warning Nightly Warning\nBefore list.\n\n- First item.\n- Second item.\n- Third item.\n- Fourth item.\n\nAfter list.\n:::\n\n:::note\nIndependent note.\n:::\n";
  let result = compile_content(source, &Default::default()).unwrap();
  assert_eq!(result.code.matches("farm-admonition-warning").count(), 1);
  assert_eq!(result.code.matches("Nightly Warning").count(), 1);
  assert_eq!(result.code.matches("farm-admonition-note").count(), 1);
  for item in [
    "First item.",
    "Second item.",
    "Third item.",
    "Fourth item.",
    "After list.",
  ] {
    assert_eq!(result.code.matches(item).count(), 1);
  }
}

#[test]
fn default_admonition_titles_keep_the_kind_but_explicit_empty_titles_remain_empty() {
  for kind in [
    "note",
    "tip",
    "info",
    "warning",
    "danger",
    "caution",
    "important",
    "success",
  ] {
    let default =
      compile_content(&format!(":::{kind}\nBody.\n:::\n"), &Default::default()).unwrap();
    assert_eq!(
      default.code.matches("farm-admonition-title").count(),
      1,
      "{kind}"
    );
    let empty = compile_content(
      &format!(":::{kind}{{title=\"\"}}\nBody.\n:::\n"),
      &Default::default(),
    )
    .unwrap();
    assert!(
      !empty.code.contains("farm-admonition-title"),
      "explicit empty title: {kind}"
    );
  }
}

#[test]
fn malformed_frontmatter_and_unclosed_directives_are_errors() {
  let yaml = compile_content(
    "---\ntitle: [invalid\n---\n# Heading\n",
    &Default::default(),
  );
  assert!(yaml
    .err()
    .is_some_and(|error| error.contains("DOCS_FRONTMATTER")));
  let directive = compile_content(":::note\nBody\n", &Default::default());
  assert!(directive
    .err()
    .is_some_and(|error| error.contains("DOCS_ADMONITION")));
}

#[test]
fn unicode_duplicate_and_colliding_headings_have_unique_ids() {
  let result = compile_content(
    "# 世界\n\n## Same\n\n## Same\n\n## Same-1\n",
    &Default::default(),
  )
  .expect("valid headings");
  assert_eq!(
    result
      .toc
      .iter()
      .map(|heading| heading.id.as_str())
      .collect::<Vec<_>>(),
    ["世界", "same", "same-1", "same-1-1"]
  );
}

#[test]
fn explicit_ids_are_reserved_and_removed_from_visible_titles() {
  let result = compile_content(
    "# Stable\n\n## Custom {#stable}\n\n## Custom\n",
    &Default::default(),
  )
  .expect("valid explicit ID");
  assert_eq!(result.toc[0].id, "stable-1");
  assert_eq!(result.toc[1].id, "stable");
  assert_eq!(result.toc[1].title, "Custom");
  assert_eq!(result.toc[2].id, "custom");
  assert!(!result.code.contains("{#stable}"));
  let duplicate = compile_content("# One {#same}\n\n## Two {#same}\n", &Default::default());
  assert!(duplicate
    .err()
    .is_some_and(|error| error.contains("duplicate explicit ID")));
}

#[test]
fn source_map_preserves_expression_position_inside_an_admonition() {
  let source = ":::note Title\n<div>{SOURCEMAP_DIRECTIVE_SENTINEL}</div>\n:::\n";
  let options = mdxjs::Options {
    filepath: Some("docs/admonition-map.mdx".into()),
    ..Default::default()
  };
  let result = compile_content_with_map(source, &options, true).expect("mapped admonition");
  let map =
    sourcemap::SourceMap::from_slice(result.source_map.as_ref().unwrap().as_bytes()).unwrap();
  let offset = result.code.find("SOURCEMAP_DIRECTIVE_SENTINEL").unwrap();
  let prefix = &result.code[..offset];
  let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
  let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32;
  let token = map
    .lookup_token(line, column)
    .expect("directive expression mapping");
  assert_eq!(token.get_src_line(), 1);
  assert_eq!(token.get_src_col(), 6);
  assert_eq!(token.get_source(), Some("docs/admonition-map.mdx"));
  assert_eq!(map.get_source_contents(0), Some(source));
}

#[test]
fn source_map_points_generated_expression_back_to_original_mdx_after_frontmatter() {
  let source = "---\ntitle: Map test\n---\n\n# 世界\n\nexport function fail() {\n  throw new Error(\"SOURCEMAP_SENTINEL\");\n}\n";
  let options = mdxjs::Options {
    filepath: Some("docs/source-map.mdx".into()),
    ..Default::default()
  };
  let result = compile_content_with_map(source, &options, true).expect("mapped MDX");
  let map =
    sourcemap::SourceMap::from_slice(result.source_map.as_ref().expect("source map").as_bytes())
      .expect("valid source map");
  let offset = result
    .code
    .find("SOURCEMAP_SENTINEL")
    .expect("generated sentinel");
  let prefix = &result.code[..offset];
  let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
  let column = prefix
    .rsplit('\n')
    .next()
    .expect("line")
    .encode_utf16()
    .count() as u32;
  let token = map.lookup_token(line, column).expect("expression mapping");
  assert_eq!(token.get_src_line(), 7);
  assert_eq!(token.get_source(), Some("docs/source-map.mdx"));
  assert_eq!(map.get_source_contents(0), Some(source));
}

#[test]
fn highlighted_code_compiles_with_metadata_and_unknown_language_diagnostics() {
  let source = "# Code\n\n```ts title=\"demo.ts\" {1} showLineNumbers\nconst 世界 = 42;\n```\n\n```unknown-language\n<literal>&text\n```\n";
  let result = farmfe_docs_content::compile_content_with_features(
    source,
    &mdxjs::Options::default(),
    true,
    Some(&farmfe_docs_content::HighlightOptions::default()),
  )
  .expect("highlighted MDX");
  assert!(result.code.contains("farm-code-token"));
  assert!(result.code.contains("--code-light"));
  assert!(result.code.contains("demo.ts"));
  assert!(result.code.contains("data-line-numbers"));
  assert!(result.code.contains("data-highlighted"));
  assert!(result
    .diagnostics
    .iter()
    .any(|d| d.contains("DOCS_UNKNOWN_LANGUAGE")));
  assert!(result.code.contains("<literal>&text"));
}
