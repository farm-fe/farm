use farmfe_core::{HashMap, swc_ecma_ast::*};
use farmfe_toolkit::swc_ecma_visit::{Visit, VisitWith};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Default, Serialize)]
pub struct Inventory {
  pub imports: Vec<Value>,
  pub uses: Vec<Value>,
}

/// The caller supplies an AST with resolver marks, so aliases and shadows differ.
pub fn inspect(module: &Module) -> Inventory {
  inspect_with_resolver(module, |_| None)
}

/// Resolve additional import identities without changing authored import strings.
/// The callback must verify module identity; matching a component basename is insufficient.
pub fn inspect_with_resolver(
  module: &Module,
  resolve: impl Fn(&str) -> Option<String>,
) -> Inventory {
  let mut visitor = Inspector {
    result: Inventory::default(),
    bindings: HashMap::default(),
  };
  for item in &module.body {
    let ModuleItem::ModuleDecl(ModuleDecl::Import(declaration)) = item else {
      continue;
    };
    let source = declaration.src.value.to_string_lossy().into_owned();
    let resolved = resolve(&source).unwrap_or_else(|| source.clone());
    if !resolved.starts_with("@theme/")
      && !resolved.starts_with("@site/")
      && resolved != "@farmfe/docs/snippets"
      && resolved != "@farmfe/docs/mdx"
    {
      continue;
    }
    for specifier in &declaration.specifiers {
      let (local, imported, type_only) = match specifier {
        ImportSpecifier::Default(value) => {
          (&value.local, "default".to_owned(), declaration.type_only)
        }
        ImportSpecifier::Namespace(value) => (&value.local, "*".to_owned(), declaration.type_only),
        ImportSpecifier::Named(value) => (
          &value.local,
          value
            .imported
            .as_ref()
            .map(|name| match name {
              ModuleExportName::Ident(id) => id.sym.to_string(),
              ModuleExportName::Str(value) => value.value.to_string_lossy().into_owned(),
            })
            .unwrap_or_else(|| value.local.sym.to_string()),
          declaration.type_only || value.is_type_only,
        ),
      };
      visitor.result.imports.push(serde_json::json!({"source":source,"resolvedSource":resolved,"local":local.sym.to_string(),"imported":imported,"typeOnly":type_only}));
      if !type_only {
        let component = if imported == "default" {
          resolved.rsplit('/').next().unwrap_or("default").to_owned()
        } else {
          imported
        };
        visitor
          .bindings
          .insert(local.to_id(), (source.clone(), component));
      }
    }
  }
  module.visit_with(&mut visitor);
  visitor.result
}

fn literal(expression: &Expr) -> Option<Value> {
  match expression {
    Expr::Lit(Lit::Str(value)) => Some(Value::String(value.value.to_string_lossy().into_owned())),
    Expr::Lit(Lit::Bool(value)) => Some(Value::Bool(value.value)),
    Expr::Lit(Lit::Num(value)) => serde_json::Number::from_f64(value.value).map(Value::Number),
    Expr::Lit(Lit::Null(_)) => Some(Value::Null),
    Expr::Paren(value) => literal(&value.expr),
    Expr::Tpl(value) if value.exprs.is_empty() => Some(Value::String(
      value
        .quasis
        .iter()
        .map(|quasi| {
          quasi
            .cooked
            .as_ref()
            .map(|text| text.to_string_lossy().into_owned())
        })
        .collect::<Option<Vec<_>>>()?
        .concat(),
    )),
    Expr::Object(value) => {
      let mut result = serde_json::Map::new();
      for property in &value.props {
        let PropOrSpread::Prop(property) = property else {
          return None;
        };
        let Prop::KeyValue(property) = property.as_ref() else {
          return None;
        };
        let key = match &property.key {
          PropName::Ident(id) => id.sym.to_string(),
          PropName::Str(value) => value.value.to_string_lossy().into_owned(),
          _ => return None,
        };
        if result.insert(key, literal(&property.value)?).is_some() {
          return None;
        }
      }
      Some(Value::Object(result))
    }
    Expr::Array(value) => Some(Value::Array(
      value
        .elems
        .iter()
        .map(|item| {
          let item = item.as_ref()?;
          if item.spread.is_some() {
            None
          } else {
            literal(&item.expr)
          }
        })
        .collect::<Option<Vec<_>>>()?,
    )),
    _ => None,
  }
}

struct Inspector {
  result: Inventory,
  bindings: HashMap<Id, (String, String)>,
}

impl Visit for Inspector {
  fn visit_jsx_element(&mut self, element: &JSXElement) {
    if let JSXElementName::Ident(name) = &element.opening.name {
      if let Some((source, component)) = self.bindings.get(&name.to_id()) {
        let mut attributes = serde_json::Map::new();
        let mut dynamic = false;
        for attribute in &element.opening.attrs {
          let JSXAttrOrSpread::JSXAttr(attribute) = attribute else {
            dynamic = true;
            continue;
          };
          let JSXAttrName::Ident(key) = &attribute.name else {
            dynamic = true;
            continue;
          };
          let value = match &attribute.value {
            None => Some(Value::Bool(true)),
            Some(JSXAttrValue::Str(value)) => {
              Some(Value::String(value.value.to_string_lossy().into_owned()))
            }
            Some(JSXAttrValue::JSXExprContainer(value)) => match &value.expr {
              JSXExpr::Expr(value) => literal(value),
              _ => None,
            },
            _ => None,
          };
          if let Some(value) = value {
            if attributes.insert(key.sym.to_string(), value).is_some() {
              dynamic = true;
            }
          } else {
            dynamic = true;
          }
        }
        let mut code = String::new();
        if component == "CodeBlock" {
          for child in &element.children {
            match child {
              JSXElementChild::JSXText(value) => code.push_str(&value.value),
              JSXElementChild::JSXExprContainer(value) => match &value.expr {
                JSXExpr::JSXEmptyExpr(_) => {}
                JSXExpr::Expr(value) => {
                  if let Some(Value::String(value)) = literal(value) {
                    code.push_str(&value);
                  } else {
                    dynamic = true;
                  }
                }
              },
              _ => dynamic = true,
            }
          }
        }
        self.result.uses.push(serde_json::json!({"source":source,"component":component,"local":name.sym.to_string(),"attributes":attributes,"code":if component == "CodeBlock" { Some(code) } else { None },"dynamic":dynamic}));
      }
    }
    element.visit_children_with(self);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::{
    module::ModuleId,
    swc_common::{GLOBALS, Globals},
    swc_ecma_ast::EsVersion,
    swc_ecma_parser::{EsSyntax, Syntax},
  };
  use farmfe_toolkit::script::{parse_module, swc_try_with::resolve_module_mark};
  use std::sync::Arc;

  fn inspect_source(source: &str) -> Inventory {
    inspect_source_with(source, |_| None)
  }

  fn inspect_source_with(source: &str, resolve: impl Fn(&str) -> Option<String>) -> Inventory {
    let id = ModuleId::from("fixture.jsx");
    let mut ast = parse_module(
      &id,
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
    GLOBALS.set(&globals, || inspect_with_resolver(&ast, resolve))
  }

  #[test]
  fn inventory_resolves_relative_identity_without_matching_unrelated_or_shadowed_tags() {
    let report = inspect_source_with(
      r#"import {PackageManagerTabs as PM} from '../../src/theme/PackageManagerTabs';
      import Other from './PackageManagerTabs';
      export function Shadow(PM){return <PM command='local'/>}
      export const example=<><PM command='pnpm dev'/><Other command='ignored'/></>;"#,
      |source| {
        (source == "../../src/theme/PackageManagerTabs")
          .then(|| "@site/src/theme/PackageManagerTabs".to_owned())
      },
    );
    assert_eq!(report.imports.len(), 1);
    assert_eq!(
      report.imports[0]["source"],
      "../../src/theme/PackageManagerTabs"
    );
    assert_eq!(
      report.imports[0]["resolvedSource"],
      "@site/src/theme/PackageManagerTabs"
    );
    assert_eq!(report.uses.len(), 1);
    assert_eq!(report.uses[0]["component"], "PackageManagerTabs");
    assert_eq!(report.uses[0]["attributes"]["command"], "pnpm dev");
  }

  #[test]
  fn inventory_uses_import_bindings_and_excludes_shadowed_tags() {
    let report = inspect_source(
      r#"import {PackageManagerTabs as PM} from '@site/src/theme/PackageManagerTabs';
      export function Shadow(PM){return <PM command='local'/>}
      export const example=<PM command='pnpm dev'/>;"#,
    );
    assert_eq!(report.imports.len(), 1);
    assert_eq!(report.uses.len(), 1);
    assert_eq!(report.uses[0]["attributes"]["command"], "pnpm dev");
  }

  #[test]
  fn inventory_reports_static_values_and_dynamic_attributes_separately() {
    let report = inspect_source(
      r#"import CodeBlock from '@theme/CodeBlock';
      export const example=<><CodeBlock language='ts'>{`const 中文 = 1;`}</CodeBlock><CodeBlock {...options}>{source}</CodeBlock></>;"#,
    );
    assert_eq!(report.uses.len(), 2);
    assert_eq!(report.uses[0]["code"], "const 中文 = 1;");
    assert_eq!(report.uses[1]["dynamic"], true);
  }

  #[test]
  fn inventory_ignores_fenced_import_and_component_examples() {
    let source = "# Example\n\n```jsx\nimport CodeBlock from '@theme/CodeBlock';\n<CodeBlock>example</CodeBlock>\n```\n";
    let compiled = farmfe_docs_content::compile_content_with_format(
      source,
      &mdxjs::Options {
        jsx: true,
        filepath: Some("fixture.mdx".into()),
        ..Default::default()
      },
      false,
      None,
      farmfe_docs_content::ContentFormat::Mdx,
    )
    .unwrap();
    let report = inspect_source(&compiled.code);
    assert!(report.imports.is_empty());
    assert!(report.uses.is_empty());
  }
}
