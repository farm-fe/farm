use swc_common::{SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_visit::VisitMut;

use super::core;
use crate::core::config::SvgProp;

pub enum AttributePosition {
  Start,
  End,
}

#[derive(Default)]
pub struct Attribute {
  pub name: String,
  pub value: Option<String>,
  pub spread: bool,
  pub literal: bool,
  pub position: Option<AttributePosition>,
}

pub struct Visitor {
  elements: Vec<String>,
  attributes: Vec<Attribute>,
}

impl Visitor {
  pub fn new(config: &core::config::Config) -> Self {
    let mut attributes = Vec::new();

    for SvgProp { key, value } in &config.svg_props {
      let attr = svg_prop_to_attr(key, value);
      attributes.push(attr);
    }

    if config.r#ref {
      attributes.push(Attribute {
        name: "ref".to_string(),
        value: Some("ref".to_string()),
        literal: true,
        ..Default::default()
      });
    }

    if config.title_prop {
      attributes.push(Attribute {
        name: "aria-labelledby".to_string(),
        value: Some("titleId".to_string()),
        literal: true,
        ..Default::default()
      });
    }

    if config.desc_prop {
      attributes.push(Attribute {
        name: "aria-describedby".to_string(),
        value: Some("descId".to_string()),
        literal: true,
        ..Default::default()
      });
    }

    let expand_props = !matches!(config.expand_props, core::config::ExpandProps::None);
    if expand_props {
      let position = match config.expand_props {
        core::config::ExpandProps::Start => Some(AttributePosition::Start),
        core::config::ExpandProps::End => Some(AttributePosition::End),
        core::config::ExpandProps::None => None,
      };
      attributes.push(Attribute {
        name: "props".to_string(),
        spread: true,
        position,
        ..Default::default()
      });
    }

    Self {
      elements: vec!["svg".to_string(), "Svg".to_string()],
      attributes,
    }
  }
}

impl VisitMut for Visitor {
  fn visit_mut_jsx_opening_element(&mut self, n: &mut JSXOpeningElement) {
    if let JSXElementName::Ident(ident) = &n.name {
      if !self.elements.contains(&ident.sym.to_string()) {
        return;
      }
    } else {
      return;
    }

    for attribute in self.attributes.iter() {
      let Attribute {
        name,
        value,
        spread,
        literal,
        position,
      } = attribute;

      let position = match position {
        Some(position) => position,
        None => &AttributePosition::End,
      };

      let new_attr = get_attr(*spread, name, value.as_ref(), *literal);

      let is_equal_attr = |attr: &JSXAttrOrSpread| -> bool {
        if *spread {
          if let JSXAttrOrSpread::SpreadElement(spread) = attr {
            if let Expr::Ident(ident) = spread.expr.as_ref() {
              return ident.sym == *name;
            }
          }
          false
        } else {
          if let JSXAttrOrSpread::JSXAttr(attr) = attr {
            if let JSXAttrName::Ident(ident) = &attr.name {
              return ident.sym == *name;
            }
          }
          false
        }
      };

      let replaced = n.attrs.clone().iter().enumerate().any(|(index, attr)| {
        if !is_equal_attr(attr) {
          return false;
        }
        n.attrs[index] = new_attr.clone();
        true
      });

      if !replaced {
        match position {
          AttributePosition::Start => {
            n.attrs.insert(0, new_attr);
          }
          AttributePosition::End => {
            n.attrs.push(new_attr);
          }
        }
      }
    }
  }
}

fn get_attr(spread: bool, name: &str, value: Option<&String>, literal: bool) -> JSXAttrOrSpread {
  if spread {
    JSXAttrOrSpread::SpreadElement(SpreadElement {
      dot3_token: DUMMY_SP,
      expr: Box::new(Expr::Ident(Ident {
        sym: name.to_string().into(),
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        optional: false,
      })),
    })
  } else {
    JSXAttrOrSpread::JSXAttr(JSXAttr {
      span: DUMMY_SP,
      name: JSXAttrName::Ident(IdentName {
        sym: name.to_string().into(),
        span: DUMMY_SP,
      }),
      value: get_attr_value(literal, value),
    })
  }
}

fn get_attr_value(literal: bool, attr_value: Option<&String>) -> Option<JSXAttrValue> {
  match attr_value {
    Some(value) => {
      if literal {
        Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
          span: DUMMY_SP,
          expr: JSXExpr::Expr(Box::new(Expr::Ident(Ident {
            sym: value.to_string().into(),
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            optional: false,
          }))),
        }))
      } else {
        Some(JSXAttrValue::Str(Str {
          span: DUMMY_SP,
          value: value.to_string().into(),
          raw: None,
        }))
      }
    }
    None => None,
  }
}

fn svg_prop_to_attr(key: &str, value: &str) -> Attribute {
  let literal = value.starts_with('{') && value.ends_with('}');
  let str = if literal {
    &value[1..value.len() - 1]
  } else {
    value
  };
  Attribute {
    name: key.to_string(),
    value: Some(str.to_string()),
    literal,
    ..Default::default()
  }
}

#[cfg(test)]
mod tests {
  use std::rc::Rc;

  use swc_common::{FileName, SourceMap};
  use swc_ecma_ast::*;
  use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
  use swc_ecma_parser::{lexer::Lexer, EsSyntax, Parser, StringInput, Syntax};
  use swc_ecma_visit::VisitMutWith;

  use super::*;

  pub struct Options {
    elements: Vec<String>,
    attributes: Vec<Attribute>,
  }

  fn code_test(input: &str, opts: Options, expected: &str) {
    let cm = Rc::<SourceMap>::default();
    let fm = cm.new_source_file(FileName::Anon.into(), input.to_string());

    let lexer = Lexer::new(
      Syntax::Es(EsSyntax {
        decorators: true,
        jsx: true,
        ..Default::default()
      }),
      EsVersion::EsNext,
      StringInput::from(&*fm),
      None,
    );

    let mut parser = Parser::new_from(lexer);
    let mut module = parser.parse_module().unwrap();

    module.visit_mut_with(&mut Visitor {
      elements: opts.elements,
      attributes: opts.attributes,
    });

    let mut buf = vec![];
    let mut emitter = Emitter {
      cfg: Default::default(),
      cm: cm.clone(),
      comments: None,
      wr: JsWriter::new(cm, "", &mut buf, None),
    };
    emitter.emit_module(&module).unwrap();
    let result = String::from_utf8_lossy(&buf).to_string();

    assert_eq!(result, expected);
  }

  #[test]
  fn should_add_simple_attribute() {
    code_test(
      r#"<div/>;"#,
      Options {
        elements: vec!["div".to_string()],
        attributes: vec![Attribute {
          name: "disabled".to_string(),
          ..Default::default()
        }],
      },
      r#"<div disabled/>;"#,
    );
  }

  #[test]
  fn should_add_attribute_with_value() {
    code_test(
      r#"<div/>;"#,
      Options {
        elements: vec!["div".to_string()],
        attributes: vec![Attribute {
          name: "disabled".to_string(),
          value: Some("true".to_string()),
          ..Default::default()
        }],
      },
      r#"<div disabled="true"/>;"#,
    );
  }

  #[test]
  fn should_add_literal_attribute() {
    code_test(
      r#"<div/>;"#,
      Options {
        elements: vec!["div".to_string()],
        attributes: vec![Attribute {
          name: "ref".to_string(),
          value: Some("ref".to_string()),
          literal: true,
          ..Default::default()
        }],
      },
      r#"<div ref={ref}/>;"#,
    );

    code_test(
      r#"<div/>;"#,
      Options {
        elements: vec!["div".to_string()],
        attributes: vec![Attribute {
          name: "ref".to_string(),
          value: Some("ref ? ref : null".to_string()),
          literal: true,
          ..Default::default()
        }],
      },
      r#"<div ref={ref ? ref : null}/>;"#,
    );
  }

  #[test]
  fn should_add_spread_attribute() {
    code_test(
      r#"<div foo><span/></div>;"#,
      Options {
        elements: vec!["div".to_string()],
        attributes: vec![Attribute {
          name: "props".to_string(),
          position: Some(AttributePosition::Start),
          spread: true,
          ..Default::default()
        }],
      },
      r#"<div {...props} foo><span/></div>;"#,
    );

    code_test(
      r#"<div><span foo="bar"/></div>;"#,
      Options {
        elements: vec!["span".to_string()],
        attributes: vec![Attribute {
          name: "props".to_string(),
          position: Some(AttributePosition::End),
          spread: true,
          ..Default::default()
        }],
      },
      r#"<div><span foo="bar" {...props}/></div>;"#,
    );
  }
}
