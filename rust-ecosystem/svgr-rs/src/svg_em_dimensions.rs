use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_visit::VisitMut;

use super::core;

const ELEMENTS: [&str; 2] = ["svg", "Svg"];

enum Size {
  Str(String),
  Num(f64),
}

pub struct Visitor {
  height: Option<Size>,
  width: Option<Size>,
}

impl Visitor {
  pub fn new(config: &core::config::Config) -> Self {
    let height: Option<Size>;
    let width: Option<Size>;

    let icon = config
      .icon
      .clone()
      .unwrap_or(core::config::Icon::Bool(false));
    match icon {
      core::config::Icon::Str(s) => {
        height = Some(Size::Str(s.clone()));
        width = Some(Size::Str(s));
      }
      core::config::Icon::Num(n) => {
        height = Some(Size::Num(n));
        width = Some(Size::Num(n));
      }
      core::config::Icon::Bool(_) => {
        if config.native {
          height = Some(Size::Num(24.0));
          width = Some(Size::Num(24.0));
        } else {
          height = None;
          width = None;
        }
      }
    }

    Self { height, width }
  }
}

impl VisitMut for Visitor {
  fn visit_mut_jsx_opening_element(&mut self, n: &mut JSXOpeningElement) {
    let is_svg = ELEMENTS.iter().any(|element| {
      if let JSXElementName::Ident(ident) = n.name.clone() {
        return ident.sym.as_str() == *element;
      }
      false
    });

    if !is_svg {
      return;
    }

    let mut required_attrs = vec!["width", "height"];

    n.attrs.iter_mut().for_each(|attr| {
      if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr
        && let JSXAttrName::Ident(ident) = &jsx_attr.name
      {
        required_attrs
          .clone()
          .iter()
          .enumerate()
          .for_each(|(index, attr)| {
            if ident.sym.as_str() == *attr {
              match *attr {
                "height" => {
                  jsx_attr.value.replace(get_value(self.height.as_ref()));
                }
                "width" => {
                  jsx_attr.value.replace(get_value(self.width.as_ref()));
                }
                _ => {}
              }
              required_attrs.remove(index);
            }
          });
      }
    });

    required_attrs.iter().for_each(|attr| {
      n.attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(IdentName::new((*attr).into(), DUMMY_SP)),
        value: Some(get_value(match *attr {
          "height" => self.height.as_ref(),
          "width" => self.width.as_ref(),
          _ => None,
        })),
      }));
    });
  }
}

fn get_value(raw: Option<&Size>) -> JSXAttrValue {
  match raw {
    None => JSXAttrValue::Str(Str {
      span: DUMMY_SP,
      value: "1em".into(),
      raw: None,
    }),
    Some(str_or_num) => match str_or_num {
      Size::Str(str) => JSXAttrValue::Str(Str {
        span: DUMMY_SP,
        value: str.clone().into(),
        raw: None,
      }),
      Size::Num(num) => JSXAttrValue::JSXExprContainer(JSXExprContainer {
        expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num(Number {
          span: DUMMY_SP,
          value: *num,
          raw: None,
        })))),
        span: DUMMY_SP,
      }),
    },
  }
}

#[cfg(test)]
mod tests {
  use std::default::Default;

  use swc_common::{sync::Lrc, FileName, SourceMap};
  use swc_ecma_ast::*;
  use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
  use swc_ecma_parser::{lexer::Lexer, EsSyntax, Parser, StringInput, Syntax};
  use swc_ecma_visit::VisitMutWith;

  use super::*;

  struct Options {
    height: Option<Size>,
    width: Option<Size>,
  }

  fn code_test(input: &str, opts: Options, expected: &str) {
    let cm = Lrc::new(SourceMap::default());
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
      height: opts.height,
      width: opts.width,
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
  fn replaces_width_or_height_attributes() {
    code_test(
      r#"<svg foo="bar" width={100} height={200}/>;"#,
      Options {
        height: None,
        width: None,
      },
      r#"<svg foo="bar" width="1em" height="1em"/>;"#,
    );
  }

  #[test]
  fn adds_em_if_they_are_not_present() {
    code_test(
      r#"<svg foo="bar"/>;"#,
      Options {
        height: None,
        width: None,
      },
      r#"<svg foo="bar" width="1em" height="1em"/>;"#,
    );
  }

  #[test]
  fn accepts_numeric_values() {
    code_test(
      r#"<svg foo="bar"/>;"#,
      Options {
        height: Some(Size::Num(24.0)),
        width: Some(Size::Num(24.0)),
      },
      r#"<svg foo="bar" width={24} height={24}/>;"#,
    );
  }

  #[test]
  fn accepts_string_values() {
    code_test(
      r#"<svg foo="bar"/>;"#,
      Options {
        height: Some(Size::Str("2em".to_string())),
        width: Some(Size::Str("2em".to_string())),
      },
      r#"<svg foo="bar" width="2em" height="2em"/>;"#,
    );
  }
}
