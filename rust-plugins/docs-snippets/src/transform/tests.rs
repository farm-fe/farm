use super::*;
use farmfe_core::{
  module::ModuleId,
  swc_common::{GLOBALS, Globals},
  swc_ecma_ast::{
    EsVersion, Expr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXElement, JSXExpr, Lit,
  },
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_toolkit::{
  script::{parse_module, swc_try_with::resolve_module_mark},
  swc_ecma_visit::{Visit, VisitWith},
};
use std::sync::Arc;

fn compile(source: &str) -> Result<(Module, Report), String> {
  compile_options(source, &Options::default())
}

fn compile_options(source: &str, options: &Options) -> Result<(Module, Report), String> {
  let mut ast = parse_module(
    &ModuleId::from("fixture.jsx"),
    Arc::new(source.into()),
    Syntax::Es(EsSyntax {
      jsx: true,
      ..Default::default()
    }),
    EsVersion::latest(),
  )
  .unwrap()
  .ast;
  let globals = Globals::new();
  resolve_module_mark(&mut ast, false, &globals);
  let report = GLOBALS.set(&globals, || lower(&mut ast, options))?;
  Ok((ast, report))
}

#[test]
fn static_html_reduces_token_elements_without_changing_copy_metadata() {
  let source = r#"import {CodeBlock} from '@farmfe/docs/snippets';export const x=<CodeBlock language="text" showLineNumbers highlightLines="1">{'<script>&λ\n'}</CodeBlock>;"#;
  let (original, _) = compile(source).unwrap();
  let mut options = Options::default();
  options.highlight.representation = farmfe_syntax_highlight::HighlightRepresentation::StaticHtml;
  let (compact, _) = compile_options(source, &options).unwrap();
  let mut before = Generated::default();
  let mut after = Generated::default();
  original.visit_with(&mut before);
  compact.visit_with(&mut after);
  assert_eq!(before.codes, after.codes);
  assert_eq!(before.numbered, after.numbered);
  assert!(before.tokens > 0);
  assert_eq!(after.tokens, 0);
  #[derive(Default)]
  struct Html(Vec<String>);
  impl Visit for Html {
    fn visit_key_value_prop(&mut self, property: &farmfe_core::swc_ecma_ast::KeyValueProp) {
      if matches!(&property.key, farmfe_core::swc_ecma_ast::PropName::Str(key) if key.value == "__html")
      {
        if let Expr::Lit(Lit::Str(value)) = property.value.as_ref() {
          self.0.push(value.value.to_string_lossy().into_owned());
        }
      }
      property.visit_children_with(self);
    }
  }
  let mut html = Html::default();
  compact.visit_with(&mut html);
  assert_eq!(html.0.len(), 1);
  assert!(html.0[0].contains("&lt;script&gt;&amp;λ"));
  assert!(html.0[0].contains("data-highlighted=\"true\""));
  assert!(!html.0[0].contains("<script>"));
}

#[test]
fn static_html_commands_keep_every_panel_and_cr_uses_numeric_reference() {
  let mut options = Options::default();
  options.highlight.representation = farmfe_syntax_highlight::HighlightRepresentation::StaticHtml;
  let (tabs, _) = compile_options(
    "import {CommandTabs} from '@farmfe/docs/snippets';export const x=<CommandTabs commands={{npm:'npm run dev',pnpm:'pnpm dev',yarn:'yarn dev',bun:'bun run dev'}}/>;",
    &options,
  ).unwrap();
  let mut generated = Generated::default();
  tabs.visit_with(&mut generated);
  assert_eq!(generated.codes.len(), 4);
  assert_eq!(generated.tokens, 0);
  let (carriage_return, _) = compile_options(
    r#"import {CodeBlock} from '@farmfe/docs/snippets';export const x=<CodeBlock>{'a\rb'}</CodeBlock>;"#,
    &options,
  ).unwrap();
  let mut generated = Generated::default();
  carriage_return.visit_with(&mut generated);
  assert_eq!(generated.codes, ["a\rb"]);
  assert_eq!(generated.tokens, 0);
  assert_eq!(generated.html.len(), 1);
  assert!(generated.html[0].contains("a&#13;b"));
  assert!(!generated.html[0].contains('\r'));
  let (fallback, _) = compile_options(
    r#"import {CodeBlock} from '@farmfe/docs/snippets';export const x=<CodeBlock>{'a\u0000b'}</CodeBlock>;"#,
    &options,
  ).unwrap();
  let mut generated = Generated::default();
  fallback.visit_with(&mut generated);
  assert_eq!(generated.codes, ["a\0b"]);
  assert!(generated.tokens > 0);
  assert!(generated.html.is_empty());
}

#[derive(Default)]
struct Generated {
  html: Vec<String>,
  codes: Vec<String>,
  tokens: usize,
  numbered: usize,
  highlighted: usize,
}
impl Visit for Generated {
  fn visit_key_value_prop(&mut self, property: &farmfe_core::swc_ecma_ast::KeyValueProp) {
    if matches!(&property.key, farmfe_core::swc_ecma_ast::PropName::Str(key) if key.value == "__html")
      && let Expr::Lit(Lit::Str(value)) = property.value.as_ref()
    {
      self.html.push(value.value.to_string_lossy().into_owned());
    }
    property.visit_children_with(self);
  }

  fn visit_jsx_element(&mut self, element: &JSXElement) {
    for attribute in &element.opening.attrs {
      let JSXAttrOrSpread::JSXAttr(attribute) = attribute else {
        continue;
      };
      let JSXAttrName::Ident(name) = &attribute.name else {
        continue;
      };
      let Some(JSXAttrValue::JSXExprContainer(container)) = &attribute.value else {
        continue;
      };
      let JSXExpr::Expr(value) = &container.expr else {
        continue;
      };
      if let Expr::Lit(Lit::Str(value)) = value.as_ref() {
        if name.sym == "data-code" {
          self.codes.push(value.value.to_string_lossy().into_owned());
        }
        if name.sym == "className" && value.value == "farm-code-token" {
          self.tokens += 1;
        }
        if name.sym == "data-line-numbers" && value.value == "true" {
          self.numbered += 1;
        }
        if name.sym == "data-highlighted" && value.value == "true" {
          self.highlighted += 1;
        }
      }
    }
    element.visit_children_with(self);
  }
}

#[test]
fn codeblock_numbers_and_highlights_lines_without_changing_copy_text() {
  let (ast, _) = compile(r#"import {CodeBlock} from '@farmfe/docs/snippets';export const x=<CodeBlock showLineNumbers highlightLines="1,3-4,3" title="A &amp; B">{'a\nb\nc\nd\n'}</CodeBlock>;"#).unwrap();
  let mut result = Generated::default();
  ast.visit_with(&mut result);
  assert_eq!(result.numbered, 1);
  assert_eq!(result.highlighted, 3);
  assert_eq!(result.codes, ["a\nb\nc\nd\n"]);
}

#[test]
fn codeblock_rejects_invalid_ranges_and_dynamic_line_numbers() {
  for props in [
    "highlightLines='0'",
    "highlightLines='3'",
    "highlightLines='2-1'",
    "highlightLines='1-'",
    "highlightLines='1,,2'",
    "showLineNumbers={dynamic}",
    "showLineNumbers='true'",
  ] {
    assert!(compile(&format!("import {{CodeBlock}} from '@farmfe/docs/snippets';export const x=<CodeBlock {props}>{{'a\\nb'}}</CodeBlock>;")).is_err(), "{props}");
  }
  let (ast, _) = compile("import {CodeBlock} from '@farmfe/docs/snippets';export const x=<CodeBlock showLineNumbers={false}>text</CodeBlock>;").unwrap();
  let mut result = Generated::default();
  ast.visit_with(&mut result);
  assert_eq!(result.numbered, 0);
}

#[test]
fn lowers_alias_with_unicode_code_and_preserves_shadowed_component() {
  let (ast, report) = compile(
    r#"import {CodeBlock as Block} from '@farmfe/docs/snippets';
    export function Local(Block){return <Block>{dynamic}</Block>}
    export const example=<Block language="ts" title="例.ts">{"const 中文 = '<&';\n"}</Block>;"#,
  )
  .unwrap();
  assert_eq!(report.transformed, 1);
  let mut result = Generated::default();
  ast.visit_with(&mut result);
  assert_eq!(result.codes, ["const 中文 = '<&';\n"]);
  assert!(result.tokens > 0);
  assert!(ast.body.iter().any(|item| matches!(item, farmfe_core::swc_ecma_ast::ModuleItem::ModuleDecl(farmfe_core::swc_ecma_ast::ModuleDecl::Import(import)) if import.src.value == "@farmfe/docs/snippets-runtime")));
}

#[test]
fn ordinary_same_named_components_are_unchanged() {
  let (ast, report) = compile("import {CodeBlock} from 'other-library';export const example=<CodeBlock>{dynamic}</CodeBlock>;").unwrap();
  assert_eq!(report.transformed, 0);
  let mut result = Generated::default();
  ast.visit_with(&mut result);
  assert!(result.codes.is_empty());
}

#[test]
fn rejects_dynamic_code_and_spread_instead_of_runtime_fallback() {
  for source in [
    "<CodeBlock>{value}</CodeBlock>",
    "<CodeBlock {...options}>text</CodeBlock>",
    "<CodeBlock language={language}>text</CodeBlock>",
  ] {
    let error = compile(&format!(
      "import {{CodeBlock}} from '@farmfe/docs/snippets';export const x={source};"
    ))
    .unwrap_err();
    assert!(error.contains("DOCS_SNIPPET_STATIC"), "{error}");
  }
}

#[test]
fn rejects_marker_value_escapes_and_namespace_imports() {
  for source in [
    "import {CodeBlock} from '@farmfe/docs/snippets';export const value=CodeBlock;",
    "import * as S from '@farmfe/docs/snippets';export const x=<S.CodeBlock>text</S.CodeBlock>;",
  ] {
    assert!(compile(source).is_err());
  }
}

#[test]
fn marker_imports_cannot_bypass_compilation_with_side_effect_or_reexport() {
  for source in [
    "import '@farmfe/docs/snippets';",
    "export {CodeBlock} from '@farmfe/docs/snippets';",
    "export * from '@farmfe/docs/snippets';",
  ] {
    let error = compile(source).unwrap_err();
    assert!(error.contains("DOCS_SNIPPET_IMPORT"), "{error}");
  }
}

#[test]
fn failed_second_snippet_leaves_entire_resolved_ast_unchanged() {
  let mut ast = parse_module(&ModuleId::from("fixture.jsx"), Arc::new("import {CodeBlock} from '@farmfe/docs/snippets';export const x=<><CodeBlock>first</CodeBlock><CodeBlock>{dynamic}</CodeBlock></>;".into()), Syntax::Es(EsSyntax {jsx:true,..Default::default()}), EsVersion::latest()).unwrap().ast;
  let globals = Globals::new();
  resolve_module_mark(&mut ast, false, &globals);
  let original = ast.clone();
  assert!(
    GLOBALS
      .set(&globals, || lower(&mut ast, &Options::default()))
      .is_err()
  );
  assert_eq!(ast, original);
}

#[test]
fn unknown_language_keeps_original_code_with_diagnostic() {
  let (ast, report) = compile("import {CodeBlock} from '@farmfe/docs/snippets';export const x=<CodeBlock language='not-a-language'>{'literal'}</CodeBlock>;").unwrap();
  assert_eq!(report.transformed, 1);
  assert!(
    report
      .diagnostics
      .iter()
      .any(|diagnostic| diagnostic.message.contains("DOCS_UNKNOWN_LANGUAGE"))
  );
  let mut result = Generated::default();
  ast.visit_with(&mut result);
  assert_eq!(result.codes, ["literal"]);
}

#[test]
fn command_tabs_compile_explicit_commands_without_shell_rewriting() {
  let (ast, report) = compile(r#"import {CommandTabs as Commands} from '@farmfe/docs/snippets';
    export const x=<Commands commands={{npm: 'npm run dev -- "pnpm"', pnpm: `pnpm dev`, yarn: 'yarn dev', bun: 'bun dev'}}/>;"#).unwrap();
  assert_eq!(report.transformed, 1);
  let mut result = Generated::default();
  ast.visit_with(&mut result);
  assert_eq!(
    result.codes,
    ["npm run dev -- \"pnpm\"", "pnpm dev", "yarn dev", "bun dev"]
  );
  assert!(result.tokens > 4);
}

#[test]
fn command_tabs_reject_ambiguous_or_dynamic_inputs() {
  for props in [
    "commands={commands}",
    "commands={{...commands}}",
    "commands={{npm: command}}",
    "commands={{npm: 'x', npm: 'y'}}",
    "commands={{['npm']: 'x'}}",
    "commands={{deno: 'deno run'}}",
    "commands={{}}",
    "command='npm run dev'",
  ] {
    let error = compile(&format!(
      "import {{CommandTabs}} from '@farmfe/docs/snippets';export const x=<CommandTabs {props}/>;"
    ))
    .unwrap_err();
    assert!(error.contains("DOCS_SNIPPET_STATIC"), "{props}: {error}");
  }
}
