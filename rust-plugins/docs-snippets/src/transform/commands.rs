use super::{Options, Report, ast, codeblock, values};
use farmfe_core::swc_common::Spanned;
use farmfe_core::swc_ecma_ast::*;
use std::collections::BTreeMap;

// The compiler supplies precompiled intrinsic pre elements. The renderer only
// presents these panels and attaches copy/tab interactions; it never parses code.
pub fn lower(
  element: &JSXElement,
  options: &Options,
  report: &mut Report,
) -> Result<JSXElement, String> {
  if !values::children(&element.children)?.trim().is_empty() {
    return Err("CommandTabs cannot contain code children".into());
  }
  if element.opening.attrs.len() != 1 {
    return Err("CommandTabs requires exactly one commands attribute".into());
  }
  let JSXAttrOrSpread::JSXAttr(attribute) = &element.opening.attrs[0] else {
    return Err("CommandTabs spread attributes are not static".into());
  };
  if !matches!(&attribute.name, JSXAttrName::Ident(name) if name.sym == "commands") {
    return Err("CommandTabs requires an explicit commands mapping".into());
  }
  let Some(JSXAttrValue::JSXExprContainer(container)) = &attribute.value else {
    return Err("commands must be a static object literal".into());
  };
  let JSXExpr::Expr(expression) = &container.expr else {
    return Err("commands must be a static object literal".into());
  };
  let Expr::Object(object) = expression.as_ref() else {
    return Err("commands must be a static object literal".into());
  };
  let mut commands = BTreeMap::new();
  for property in &object.props {
    let PropOrSpread::Prop(property) = property else {
      return Err("commands cannot contain spreads".into());
    };
    let Prop::KeyValue(property) = property.as_ref() else {
      return Err("commands require literal key/value properties".into());
    };
    let tool = match &property.key {
      PropName::Ident(name) => name.sym.to_string(),
      PropName::Str(name) => name.value.as_str().ok_or("tool must be UTF-8")?.to_owned(),
      _ => return Err("command tool keys cannot be computed".into()),
    };
    if !["npm", "pnpm", "yarn", "bun"].contains(&tool.as_str()) {
      return Err(format!("unsupported command tool {tool}"));
    }
    let command = values::string(&property.value).ok_or("commands must be static UTF-8 strings")?;
    if command.trim().is_empty() {
      return Err(format!("command for {tool} cannot be empty"));
    }
    if commands.insert(tool.clone(), command).is_some() {
      return Err(format!("duplicate command tool {tool}"));
    }
  }
  if commands.is_empty() {
    return Err("commands must contain at least one tool".into());
  }
  let span = element.span;
  let mut panels = Vec::new();
  for tool in ["npm", "pnpm", "yarn", "bun"] {
    let Some(command) = commands.remove(tool) else {
      continue;
    };
    let input = ast::element(
      ast::name("pre", element.opening.name.span()),
      vec![ast::text_attribute("language", "sh", span)],
      vec![ast::child(&command, span)],
      span,
    );
    let mut panel = codeblock::lower(&input, options, report)?;
    panel
      .opening
      .attrs
      .push(ast::text_attribute("data-command-tool", tool, span));
    panels.push(JSXElementChild::JSXElement(Box::new(panel)));
  }
  Ok(ast::element(
    element.opening.name.clone(),
    Vec::new(),
    panels,
    span,
  ))
}
