use super::{Diagnostic, Options, Report, ast, values};
use farmfe_core::swc_common::Spanned;
use farmfe_core::swc_ecma_ast::*;
use std::collections::BTreeSet;

pub fn lower(
  element: &JSXElement,
  options: &Options,
  report: &mut Report,
) -> Result<JSXElement, String> {
  let mut seen = BTreeSet::new();
  let (mut code, mut language, mut title) = (None, "text".to_owned(), None);
  let (mut numbers, mut ranges) = (false, None);
  for attribute in &element.opening.attrs {
    let JSXAttrOrSpread::JSXAttr(attribute) = attribute else {
      return Err("spread attributes are not static".into());
    };
    let JSXAttrName::Ident(name) = &attribute.name else {
      return Err("namespaced attributes are unsupported".into());
    };
    if !seen.insert(name.sym.to_string()) {
      return Err(format!("duplicate attribute {}", name.sym));
    }
    if name.sym == "showLineNumbers" {
      numbers =
        values::attribute_boolean(attribute).ok_or("showLineNumbers must be a static boolean")?;
      continue;
    }
    let value = values::attribute_string(attribute)
      .ok_or_else(|| format!("{} must be a static UTF-8 string", name.sym))?;
    match name.sym.as_ref() {
      "code" => code = Some(value),
      "language" => language = value,
      "title" => title = Some(value),
      "highlightLines" => ranges = Some(value),
      other => return Err(format!("unsupported CodeBlock attribute {other}")),
    }
  }
  let children = values::children(&element.children)?;
  if code.is_some() && !children.is_empty() {
    return Err("use code or string children, not both".into());
  }
  let code = code.unwrap_or(children);
  let highlighted = farmfe_syntax_highlight::highlight(&code, &language, &options.highlight)?;
  let lines = ranges
    .as_deref()
    .map(|ranges| farmfe_syntax_highlight::parse_line_ranges(ranges, highlighted.lines.len()))
    .transpose()?
    .unwrap_or_default();
  if !highlighted.known_language {
    report.diagnostics.push(Diagnostic::at(
      format!("[DOCS_UNKNOWN_LANGUAGE] {language}; rendered as plain text"),
      element.opening.name.span(),
    ));
  }
  // JSX '<' punctuation may have no upstream mapping. The component name is
  // an explicit mapped anchor in Docs-generated JSX and ordinary TSX alike.
  let span = element.opening.name.span();
  let mut attributes = vec![
    ast::text_attribute("className", "farm-code-block", span),
    ast::text_attribute("data-code", &code, span),
    ast::text_attribute("data-language", &language, span),
  ];
  if let Some(title) = title {
    attributes.push(ast::text_attribute("data-title", &title, span));
  }
  if numbers {
    attributes.push(ast::text_attribute("data-line-numbers", "true", span));
  }
  if !highlighted.known_language {
    attributes.push(ast::text_attribute("data-unknown-language", "true", span));
  }
  let lines = highlighted
    .lines
    .into_iter()
    .enumerate()
    .map(|(index, tokens)| {
      let tokens = tokens
        .into_iter()
        .map(|token| {
          JSXElementChild::JSXElement(Box::new(ast::element(
            ast::name("span", span),
            vec![
              ast::text_attribute("className", "farm-code-token", span),
              ast::attribute("style", ast::style(&token.light, &token.dark, span), span),
            ],
            vec![ast::child(&token.text, span)],
            span,
          )))
        })
        .collect();
      let mut attributes = vec![
        ast::text_attribute("className", "farm-code-line", span),
        ast::text_attribute("data-line", &(index + 1).to_string(), span),
      ];
      if lines.contains(&(index + 1)) {
        attributes.push(ast::text_attribute("data-highlighted", "true", span));
      }
      JSXElementChild::JSXElement(Box::new(ast::element(
        ast::name("span", span),
        attributes,
        tokens,
        span,
      )))
    })
    .collect();
  let content = ast::element(
    ast::name("code", span),
    vec![ast::text_attribute(
      "className",
      &format!("language-{language}"),
      span,
    )],
    lines,
    span,
  );
  Ok(ast::element(
    element.opening.name.clone(),
    attributes,
    vec![JSXElementChild::JSXElement(Box::new(content))],
    element.span,
  ))
}
