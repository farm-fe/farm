use farmfe_docs_content::{ContentFormat, HighlightOptions, emit_content, parse_content};

#[test]
fn explicit_inline_signatures_share_native_tokens_and_semantic_markdown() {
  let source = "# API\n\n- **type**: <code data-language=\"ts\">(moduleId: string) =&gt; void | Promise&lt;void&gt;</code>\n\nOrdinary `Promise` and `src/main.ts` stay ordinary.\n";
  for format in [ContentFormat::Markdown, ContentFormat::Mdx] {
    let options = mdxjs::Options {
      jsx: true,
      ..Default::default()
    };
    let parsed =
      parse_content(source, &options, Some(&HighlightOptions::default()), format).unwrap();
    let md = parsed.machine_markdown().unwrap();
    assert!(
      md.contains("(moduleId: string) => void | Promise<void>"),
      "{md}"
    );
    assert!(md.contains("src/main.ts"));
    let output = emit_content(parsed, true).unwrap();
    assert!(output.code.contains("farm-inline-code"));
    assert!(output.code.contains("#4ec9b0"));
    assert!(output.code.contains("#569cd6"));
    assert!(!output.code.contains("highlight.js"));
  }
}

#[test]
fn marked_inline_code_rejects_dynamic_and_unsupported_inputs() {
  let options = mdxjs::Options {
    jsx: true,
    ..Default::default()
  };
  for source in [
    "<code data-language={getLanguage()}>x</code>",
    "<code data-language=\"ts\" onClick={run}>x</code>",
    "<code data-language=\"ts\">{run()}</code>",
    "<code data-language=\"not-a-language\">x</code>",
  ] {
    assert!(
      parse_content(
        source,
        &options,
        Some(&HighlightOptions::default()),
        ContentFormat::Mdx
      )
      .is_err(),
      "{source}"
    );
  }
  let forged = "<code className=\"farm-inline-code\"><span className=\"farm-code-token\" style={run()}>x</span></code>";
  let parsed = parse_content(
    forged,
    &options,
    Some(&HighlightOptions::default()),
    ContentFormat::Mdx,
  )
  .unwrap();
  assert!(parsed.machine_markdown().is_err());
}

#[test]
fn fenced_and_explicit_inline_code_share_the_selected_custom_palette() {
  let source = r#"# API

```ts
const value: Promise<void> = run();
```

<code data-language="ts">(id: string) =&gt; Promise&lt;void&gt;</code>
"#;
  let options: HighlightOptions = serde_json::from_value(serde_json::json!({"light":"farm-light","dark":"farm-dark","tokens":{"light":{"type":"#123456"},"dark":{"type":"#abcdef"}}})).unwrap();
  for format in [ContentFormat::Markdown, ContentFormat::Mdx] {
    let parsed = parse_content(
      source,
      &mdxjs::Options {
        jsx: true,
        ..Default::default()
      },
      Some(&options),
      format,
    )
    .unwrap();
    assert!(parsed.machine_markdown().unwrap().contains("Promise<void>"));
    let output = emit_content(parsed, true).unwrap();
    assert!(output.code.contains("farm-inline-code"));
    assert!(output.code.matches("#123456").count() >= 2);
    assert!(output.code.matches("#abcdef").count() >= 2);
  }
}
