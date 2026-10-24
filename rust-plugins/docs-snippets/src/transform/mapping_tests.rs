use super::*;
use farmfe_core::{
  module::ModuleId,
  swc_common::{GLOBALS, Globals},
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_toolkit::{
  script::{codegen_module, parse_module, swc_try_with::resolve_module_mark},
  sourcemap::{SourceMap, build_sourcemap},
};
use std::sync::Arc;

fn position(text: &str, needle: &str) -> (u32, u32) {
  let prefix = &text[..text.find(needle).expect("generated marker")];
  (
    prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
    prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
  )
}

fn assert_mapped(import: &str, snippet: &str) {
  assert_mapped_options(import, snippet, &Options::default());
}

fn assert_mapped_options(import: &str, snippet: &str, options: &Options) {
  let source = format!(
    "---\ntitle: Unicode map\n---\n\nimport {{{import}}} from '@farmfe/docs/snippets';\n\n# 世界\n\n{snippet}\n\n<span>{{'AFTER_SNIPPET_SENTINEL'}}</span>\n"
  );
  let compiled = farmfe_docs_content::compile_content_with_format(
    &source,
    &mdxjs::Options {
      filepath: Some("docs/unicode-map.mdx".into()),
      jsx: true,
      ..Default::default()
    },
    true,
    None,
    farmfe_docs_content::ContentFormat::Auto,
  )
  .unwrap();
  let upstream = SourceMap::from_slice(compiled.source_map.as_ref().unwrap().as_bytes()).unwrap();
  let parsed = parse_module(
    &ModuleId::from("generated/unicode-map.jsx"),
    Arc::new(compiled.code),
    Syntax::Es(EsSyntax {
      jsx: true,
      ..Default::default()
    }),
    EsVersion::latest(),
  )
  .unwrap();
  let mut ast = parsed.ast;
  let globals = Globals::new();
  resolve_module_mark(&mut ast, false, &globals);
  let report = GLOBALS.set(&globals, || lower(&mut ast, options)).unwrap();
  assert_eq!(report.transformed, 1);
  let mut mappings = Vec::new();
  let code = String::from_utf8(
    codegen_module(
      &ast,
      parsed.source_map.clone(),
      Some(&mut mappings),
      Default::default(),
      None,
    )
    .unwrap(),
  )
  .unwrap();
  let generated = build_sourcemap(parsed.source_map, &mappings);
  for (needle, expected_line) in [("farm-code-token", 8), ("AFTER_SNIPPET_SENTINEL", 10)] {
    let (line, col) = position(&code, needle);
    let token = generated.lookup_token(line, col).expect("JSX mapping");
    let original = upstream
      .lookup_token(token.get_src_line(), token.get_src_col())
      .expect("author mapping");
    assert_eq!(original.get_src_line(), expected_line, "{needle}");
    assert_eq!(
      original.get_source().map(|value| value.as_str()),
      Some("docs/unicode-map.mdx")
    );
  }
  assert_eq!(
    upstream.get_source_contents(0).map(|value| value.as_str()),
    Some(source.as_str())
  );
}

#[test]
fn static_html_traces_through_real_mdx_map_to_author_line() {
  let mut options = Options::default();
  options.highlight.representation = farmfe_syntax_highlight::HighlightRepresentation::StaticHtml;
  assert_mapped_options(
    "CodeBlock",
    "<CodeBlock language=\"ts\">{'const answer = 42;'}</CodeBlock>",
    &options,
  );
}

/// Run with FARM_DOCS_HIGHLIGHT_FIXTURE_DIR to export real compiler output for
/// the Node SSR/browser harness. Without it this still verifies both pipelines.
#[test]
fn react_runtime_fixture_uses_real_mdx_and_snippets_outputs() {
  let source = r#"import {CodeBlock, CommandTabs} from '@farmfe/docs/snippets';

# Static highlight fixture

```text title="unsafe.txt" {1} showLineNumbers
</code><script>alert("x")</script>& λ
```

<CodeBlock language="text" title="marker.txt" showLineNumbers highlightLines="1">{'</code><script>alert("x")</script>& λ\n'}</CodeBlock>

<CommandTabs commands={{npm:'npm run dev',pnpm:'pnpm dev',yarn:'yarn dev',bun:'bun run dev'}}/>
"#;
  for (name, representation) in [
    ("jsx", farmfe_syntax_highlight::HighlightRepresentation::Jsx),
    (
      "static-html",
      farmfe_syntax_highlight::HighlightRepresentation::StaticHtml,
    ),
  ] {
    let mut options = Options::default();
    options.highlight.representation = representation;
    let compiled = farmfe_docs_content::compile_content_with_features(
      source,
      &mdxjs::Options {
        filepath: Some("runtime-fixture.mdx".into()),
        jsx: true,
        ..Default::default()
      },
      true,
      Some(&options.highlight),
    )
    .unwrap();
    let parsed = parse_module(
      &ModuleId::from("runtime-fixture.jsx"),
      Arc::new(compiled.code),
      Syntax::Es(EsSyntax {
        jsx: true,
        ..Default::default()
      }),
      EsVersion::latest(),
    )
    .unwrap();
    let mut ast = parsed.ast;
    let globals = Globals::new();
    resolve_module_mark(&mut ast, false, &globals);
    let report = GLOBALS.set(&globals, || lower(&mut ast, &options)).unwrap();
    assert_eq!(report.transformed, 2);
    let code = String::from_utf8(
      codegen_module(&ast, parsed.source_map, None, Default::default(), None).unwrap(),
    )
    .unwrap();
    assert_eq!(
      code.contains("dangerouslySetInnerHTML"),
      name == "static-html"
    );
    if let Some(directory) = std::env::var_os("FARM_DOCS_HIGHLIGHT_FIXTURE_DIR") {
      let directory = std::path::PathBuf::from(directory);
      std::fs::create_dir_all(&directory).unwrap();
      std::fs::write(directory.join(format!("{name}.jsx")), code).unwrap();
      std::fs::write(
        directory.join(format!("{name}.map.json")),
        compiled.source_map.unwrap(),
      )
      .unwrap();
    }
  }
}

#[test]
fn generated_tokens_trace_through_real_mdx_map_to_original_author_line() {
  for (import, snippet) in [
    (
      "CodeBlock",
      r#"<CodeBlock language="ts">{'const 中文 = 1;'}</CodeBlock>"#,
    ),
    (
      "CodeBlock",
      r#"<CodeBlock language="ts" code={'const 中文 = 1;'}/>"#,
    ),
    ("CodeBlock", r#"<CodeBlock>npm install example</CodeBlock>"#),
    (
      "CommandTabs",
      r#"<CommandTabs commands={{npm:'npm run dev',pnpm:'pnpm dev',yarn:'yarn dev',bun:'bun dev'}}/>"#,
    ),
  ] {
    assert_mapped(import, snippet);
  }
}
