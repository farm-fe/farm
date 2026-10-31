use crate::{ContentLanguage, PageMetadata};
use serde_json::{Map, Value};
use std::{
  collections::{BTreeSet, HashMap, HashSet},
  sync::Arc,
};
use swc_common::{FileName, SourceMap, Span, Spanned};
use swc_ecma_ast::*;
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_visit::{Visit, VisitWith};

pub struct ScriptPage {
  pub metadata: Value,
  pub language: ContentLanguage,
  pub anchors: Vec<String>,
}
fn unwrap_expression(expr: &Expr) -> &Expr {
  match expr {
    Expr::Paren(value) => unwrap_expression(&value.expr),
    Expr::TsAs(value) => unwrap_expression(&value.expr),
    Expr::TsTypeAssertion(value) => unwrap_expression(&value.expr),
    Expr::TsConstAssertion(value) => unwrap_expression(&value.expr),
    Expr::TsSatisfies(value) => unwrap_expression(&value.expr),
    Expr::TsNonNull(value) => unwrap_expression(&value.expr),
    _ => expr,
  }
}
fn diagnostic(
  code: &str,
  file: &str,
  source: &str,
  base: u32,
  span: Span,
  message: impl std::fmt::Display,
) -> String {
  let offset = span.lo.0.saturating_sub(base) as usize;
  let prefix = source.get(..offset).unwrap_or("");
  let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
  let column = prefix
    .rsplit('\n')
    .next()
    .unwrap_or("")
    .encode_utf16()
    .count()
    + 1;
  let snippet = source.lines().nth(line - 1).unwrap_or("");
  format!("[{code}] {file}:{line}:{column}: {message}\n{snippet}")
}
fn export_name(name: &ModuleExportName) -> String {
  match name {
    ModuleExportName::Ident(value) => value.sym.to_string(),
    ModuleExportName::Str(value) => value.value.to_string_lossy().into_owned(),
  }
}
fn literal(expr: &Expr) -> Option<String> {
  match unwrap_expression(expr) {
    Expr::Lit(Lit::Str(value)) => Some(value.value.to_string_lossy().into_owned()),
    Expr::Tpl(value) if value.exprs.is_empty() => value
      .quasis
      .first()
      .and_then(|quasi| quasi.cooked.as_ref())
      .map(|value| value.to_string_lossy().into_owned()),
    _ => None,
  }
}
fn component(expr: &Expr, bindings: &HashMap<String, &Expr>, seen: &mut HashSet<String>) -> bool {
  match unwrap_expression(expr) {
    Expr::Fn(_) | Expr::Arrow(_) | Expr::Class(_) | Expr::Call(_) => true,
    Expr::Ident(name) => {
      let name = name.sym.to_string();
      if !seen.insert(name.clone()) {
        return false;
      }
      bindings
        .get(&name)
        .map(|expr| component(expr, bindings, seen))
        .unwrap_or(true)
    }
    _ => false,
  }
}
#[derive(Default)]
struct Anchors(BTreeSet<String>);
impl Visit for Anchors {
  fn visit_jsx_opening_element(&mut self, node: &JSXOpeningElement) {
    // Runtime conditions/custom components cannot be proven here; index only
    // literal intrinsic IDs. Full DOM correctness remains browser acceptance.
    let intrinsic = matches!(&node.name,JSXElementName::Ident(name) if name.sym.chars().next().is_some_and(char::is_lowercase));
    if intrinsic {
      for attr in &node.attrs {
        if let JSXAttrOrSpread::JSXAttr(attr) = attr {
          if matches!(&attr.name,JSXAttrName::Ident(name) if name.sym=="id") {
            let value = match &attr.value {
              Some(JSXAttrValue::Str(value)) => Some(value.value.to_string_lossy().into_owned()),
              Some(JSXAttrValue::JSXExprContainer(value)) => match &value.expr {
                JSXExpr::Expr(expr) => literal(expr),
                _ => None,
              },
              _ => None,
            };
            if let Some(value) = value {
              self.0.insert(value);
            }
          }
        }
      }
    }
    node.visit_children_with(self);
  }
}
fn locale_titles(
  expr: &Expr,
  file: &str,
  source: &str,
  base: u32,
  locales: &[String],
) -> Result<Value, String> {
  let fail =
    |span: Span, message: &str| diagnostic("DOCS_PAGE_METADATA", file, source, base, span, message);
  let Expr::Object(object) = unwrap_expression(expr) else {
    return Err(fail(expr.span(), "titleByLocale requires a static object"));
  };
  let mut result = Map::new();
  for property in &object.props {
    let PropOrSpread::Prop(property) = property else {
      return Err(fail(
        property.span(),
        "titleByLocale does not allow spreads",
      ));
    };
    let Prop::KeyValue(property) = &**property else {
      return Err(fail(
        property.span(),
        "titleByLocale requires literal keys and strings",
      ));
    };
    let key = match &property.key {
      PropName::Ident(value) => value.sym.to_string(),
      PropName::Str(value) => value.value.to_string_lossy().into_owned(),
      _ => {
        return Err(fail(
          property.key.span(),
          "titleByLocale keys must be literal",
        ));
      }
    };
    if !locales.contains(&key) || result.contains_key(&key) {
      return Err(fail(
        property.key.span(),
        "titleByLocale has an undeclared or duplicate locale",
      ));
    }
    let value = literal(&property.value).ok_or_else(|| {
      fail(
        property.value.span(),
        "localized titles require static strings",
      )
    })?;
    if value.trim().is_empty() || value.chars().any(char::is_control) {
      return Err(fail(
        property.value.span(),
        "localized titles require nonempty text without controls",
      ));
    }
    result.insert(key, Value::String(value));
  }
  Ok(Value::Object(result))
}

pub fn parse(
  source: &str,
  file: &str,
  extension: &str,
  locales: &[String],
) -> Result<Option<ScriptPage>, String> {
  let language = match extension {
    "ts" => ContentLanguage::Ts,
    "tsx" => ContentLanguage::Tsx,
    _ => ContentLanguage::Jsx,
  };
  let syntax = match language {
    ContentLanguage::Ts | ContentLanguage::Tsx => Syntax::Typescript(TsSyntax {
      tsx: language == ContentLanguage::Tsx,
      ..Default::default()
    }),
    _ => Syntax::Es(EsSyntax {
      jsx: true,
      import_attributes: true,
      ..Default::default()
    }),
  };
  let map = Arc::new(SourceMap::default());
  let source_file = map.new_source_file(FileName::Custom(file.into()).into(), source.to_owned());
  let mut parser = Parser::new_from(Lexer::new(
    syntax,
    EsVersion::EsNext,
    StringInput::from(&*source_file),
    None,
  ));
  let parsed = parser.parse_module();
  let mut errors = parser.take_errors();
  let module = match parsed {
    Ok(module) => module,
    Err(error) => {
      return Err(diagnostic(
        "DOCS_PAGE_PARSE",
        file,
        source,
        source_file.start_pos.0,
        error.span(),
        error.kind().msg(),
      ));
    }
  };
  if let Some(error) = errors.pop() {
    return Err(diagnostic(
      "DOCS_PAGE_PARSE",
      file,
      source,
      source_file.start_pos.0,
      error.span(),
      error.kind().msg(),
    ));
  }
  let mut has_default = false;
  let mut bindings = HashMap::new();
  for item in &module.body {
    let decl = match item {
      ModuleItem::Stmt(Stmt::Decl(decl)) => Some(decl),
      ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(value)) => Some(&value.decl),
      _ => None,
    };
    if let Some(Decl::Var(decl)) = decl {
      for declaration in &decl.decls {
        if let (Pat::Ident(name), Some(expr)) = (&declaration.name, &declaration.init) {
          bindings.insert(name.id.sym.to_string(), expr.as_ref());
        }
      }
    }
  }
  let mut metadata = None;
  for item in &module.body {
    let ModuleItem::ModuleDecl(decl) = item else {
      continue;
    };
    match decl {
      ModuleDecl::ExportDefaultDecl(value) => {
        has_default = !matches!(value.decl, DefaultDecl::TsInterfaceDecl(_))
      }
      ModuleDecl::ExportDefaultExpr(value) => {
        has_default = component(&value.expr, &bindings, &mut HashSet::new());
      }
      ModuleDecl::ExportNamed(export) if !export.type_only => {
        for specifier in &export.specifiers {
          if let ExportSpecifier::Named(value) = specifier {
            if !value.is_type_only
              && export_name(value.exported.as_ref().unwrap_or(&value.orig)) == "metadata"
            {
              return Err(diagnostic(
                "DOCS_PAGE_METADATA",
                file,
                source,
                source_file.start_pos.0,
                value.span,
                "use a direct export const metadata object; metadata re-exports are not static inputs",
              ));
            }
            if !value.is_type_only
              && export_name(value.exported.as_ref().unwrap_or(&value.orig)) == "default"
            {
              has_default = match &value.orig {
                ModuleExportName::Ident(name) if export.src.is_none() => bindings
                  .get(&name.sym.to_string())
                  .map(|expr| component(expr, &bindings, &mut HashSet::new()))
                  .unwrap_or(true),
                _ => true,
              };
            }
          }
        }
      }
      ModuleDecl::ExportDecl(value) => {
        if let Decl::Var(decl) = &value.decl {
          for declaration in &decl.decls {
            if matches!(&declaration.name,Pat::Ident(name) if name.id.sym=="metadata") {
              metadata = Some(declaration.init.as_deref().ok_or_else(|| {
                diagnostic(
                  "DOCS_PAGE_METADATA",
                  file,
                  source,
                  source_file.start_pos.0,
                  declaration.span,
                  "metadata requires an object literal",
                )
              })?);
            }
          }
        }
      }
      _ => {}
    }
  }
  if !has_default {
    if let Some(expr) = metadata {
      return Err(diagnostic(
        "DOCS_PAGE_EXPORT",
        file,
        source,
        source_file.start_pos.0,
        expr.span(),
        "page metadata requires a default component export",
      ));
    }
    return Ok(None);
  }
  let mut values = Map::new();
  if let Some(expr) = metadata {
    let Expr::Object(object) = unwrap_expression(expr) else {
      return Err(diagnostic(
        "DOCS_PAGE_METADATA",
        file,
        source,
        source_file.start_pos.0,
        expr.span(),
        "metadata must be a static object literal",
      ));
    };
    for property in &object.props {
      let PropOrSpread::Prop(property) = property else {
        return Err(diagnostic(
          "DOCS_PAGE_METADATA",
          file,
          source,
          source_file.start_pos.0,
          property.span(),
          "metadata spreads are not static",
        ));
      };
      let Prop::KeyValue(property) = &**property else {
        return Err(diagnostic(
          "DOCS_PAGE_METADATA",
          file,
          source,
          source_file.start_pos.0,
          property.span(),
          "metadata fields require literal values",
        ));
      };
      let key = match &property.key {
        PropName::Ident(value) => value.sym.to_string(),
        PropName::Str(value) => value.value.to_string_lossy().into_owned(),
        _ => {
          return Err(diagnostic(
            "DOCS_PAGE_METADATA",
            file,
            source,
            source_file.start_pos.0,
            property.key.span(),
            "metadata keys must be literal",
          ));
        }
      };
      if ![
        "id",
        "title",
        "titleByLocale",
        "description",
        "slug",
        "layout",
      ]
      .contains(&key.as_str())
        || values.contains_key(&key)
      {
        return Err(diagnostic(
          "DOCS_PAGE_METADATA",
          file,
          source,
          source_file.start_pos.0,
          property.key.span(),
          format!("unknown or duplicate field {key}"),
        ));
      }
      let value = if key == "titleByLocale" {
        locale_titles(
          &property.value,
          file,
          source,
          source_file.start_pos.0,
          locales,
        )?
      } else {
        match unwrap_expression(&property.value) {
          Expr::Lit(Lit::Null(_)) => Value::Null,
          expr if literal(expr).is_some() => Value::String(literal(expr).unwrap_or_default()),
          _ => {
            return Err(diagnostic(
              "DOCS_PAGE_METADATA",
              file,
              source,
              source_file.start_pos.0,
              property.value.span(),
              "metadata values require static strings or null",
            ));
          }
        }
      };
      if let Some(text) = value.as_str() {
        if text.trim().is_empty()
          || text.chars().any(char::is_control)
          || (key == "layout" && !matches!(text, "default" | "none"))
        {
          return Err(diagnostic(
            "DOCS_PAGE_METADATA",
            file,
            source,
            source_file.start_pos.0,
            property.value.span(),
            "invalid metadata text/layout",
          ));
        }
        if key == "slug" {
          crate::prepare::safe_slug(text).map_err(|cause| {
            diagnostic(
              "DOCS_PAGE_METADATA",
              file,
              source,
              source_file.start_pos.0,
              property.value.span(),
              cause,
            )
          })?;
        }
      }
      values.insert(key, value);
    }
  }
  let metadata = Value::Object(values);
  serde_json::from_value::<PageMetadata>(metadata.clone())
    .map_err(|cause| format!("[DOCS_PAGE_METADATA] {file}: {cause}"))?;
  let mut anchors = Anchors::default();
  module.visit_with(&mut anchors);
  Ok(Some(ScriptPage {
    metadata,
    language,
    anchors: anchors.0.into_iter().collect(),
  }))
}
