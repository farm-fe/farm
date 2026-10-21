use farmfe_core::{
  swc_common::{Span, SyntaxContext},
  swc_ecma_ast::*,
};

pub fn string(value: &str, span: Span) -> Expr {
  Expr::Lit(Lit::Str(Str {
    span,
    value: value.into(),
    raw: None,
  }))
}

pub fn child(value: &str, span: Span) -> JSXElementChild {
  JSXElementChild::JSXExprContainer(JSXExprContainer {
    span,
    expr: JSXExpr::Expr(Box::new(string(value, span))),
  })
}

pub fn attribute(name: &str, value: Expr, span: Span) -> JSXAttrOrSpread {
  JSXAttrOrSpread::JSXAttr(JSXAttr {
    span,
    name: JSXAttrName::Ident(IdentName::new(name.into(), span)),
    value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
      span,
      expr: JSXExpr::Expr(Box::new(value)),
    })),
  })
}

pub fn text_attribute(name: &str, value: &str, span: Span) -> JSXAttrOrSpread {
  attribute(name, string(value, span), span)
}

pub fn name(value: &str, span: Span) -> JSXElementName {
  JSXElementName::Ident(Ident::new(value.into(), span, SyntaxContext::empty()))
}

pub fn element(
  name: JSXElementName,
  attrs: Vec<JSXAttrOrSpread>,
  children: Vec<JSXElementChild>,
  span: Span,
) -> JSXElement {
  JSXElement {
    span,
    opening: JSXOpeningElement {
      name: name.clone(),
      span,
      attrs,
      self_closing: false,
      type_args: None,
    },
    children,
    closing: Some(JSXClosingElement { span, name }),
  }
}

pub fn style(light: &str, dark: &str, span: Span) -> Expr {
  Expr::Object(ObjectLit {
    span,
    props: [("--code-light", light), ("--code-dark", dark)]
      .into_iter()
      .map(|(key, value)| {
        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
          key: PropName::Str(Str {
            span,
            value: key.into(),
            raw: None,
          }),
          value: Box::new(string(value, span)),
        })))
      })
      .collect(),
  })
}
