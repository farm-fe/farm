use std::collections::BTreeMap;
use swc_core::{
  common::Span,
  ecma::{
    ast::{
      Expr, IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXElement,
      JSXElementChild, JSXExpr, JSXExprContainer, KeyValueProp, Lit, Module, ObjectLit, Prop,
      PropName, PropOrSpread, Str,
    },
    visit::{VisitMut, VisitMutWith},
  },
};

/// Only Rust-generated fenced blocks are entered, by their original source span.
/// Authored MDX cannot request arbitrary raw markup through an HTML attribute.
pub(crate) type Blocks = BTreeMap<(usize, usize), String>;

fn raw_html_attribute(html: String, span: Span) -> JSXAttrOrSpread {
  JSXAttrOrSpread::JSXAttr(JSXAttr {
    span,
    name: JSXAttrName::Ident(IdentName::new("dangerouslySetInnerHTML".into(), span)),
    value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
      span,
      expr: JSXExpr::Expr(Box::new(Expr::Object(ObjectLit {
        span,
        props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
          key: PropName::Str(Str {
            span,
            value: "__html".into(),
            raw: None,
          }),
          value: Box::new(Expr::Lit(Lit::Str(Str {
            span,
            value: html.into(),
            raw: None,
          }))),
        })))],
      }))),
    })),
  })
}

/// Attach escaped generated markup in the AST before code generation, preserving
/// normal MDX component resolution, source-map chaining and all outer metadata.
/// Opt-in static representation changes the code component's children contract;
/// custom components must forward these props instead of replacing children.
pub(crate) fn apply(module: &mut Module, blocks: Blocks) -> Result<(), String> {
  struct Attach {
    blocks: Blocks,
    error: Option<String>,
  }
  impl VisitMut for Attach {
    fn visit_mut_jsx_element(&mut self, element: &mut JSXElement) {
      if self.error.is_some() {
        return;
      }
      let key = (element.span.lo.0 as usize, element.span.hi.0 as usize);
      if let Some(html) = self.blocks.remove(&key) {
        let [JSXElementChild::JSXElement(code)] = element.children.as_mut_slice() else {
          self.error = Some("[DOCS_HIGHLIGHT_HTML] generated code structure changed".into());
          return;
        };
        if !code.children.is_empty()
          || code.opening.attrs.iter().any(|attribute| {
            matches!(attribute, JSXAttrOrSpread::JSXAttr(attribute)
              if matches!(&attribute.name, JSXAttrName::Ident(name)
                if name.sym == "dangerouslySetInnerHTML" || name.sym == "children"))
          })
        {
          self.error = Some("[DOCS_HIGHLIGHT_HTML] generated code properties conflict".into());
          return;
        }
        code
          .opening
          .attrs
          .push(raw_html_attribute(html, element.span.shrink_to_lo()));
      }
      element.visit_mut_children_with(self);
    }
  }
  let mut visitor = Attach {
    blocks,
    error: None,
  };
  module.visit_mut_with(&mut visitor);
  if let Some(error) = visitor.error {
    return Err(error);
  }
  if !visitor.blocks.is_empty() {
    return Err("[DOCS_HIGHLIGHT_HTML] generated code mapping was not retained".into());
  }
  Ok(())
}
