use std::collections::HashMap;

use swc_common::{SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_visit::VisitMut;

use super::core;

pub struct Visitor {
  values: HashMap<String, String>,
}

impl Visitor {
  pub fn new(config: &core::config::Config) -> Self {
    let replace_attr_values = config.replace_attr_values.as_ref().unwrap();

    Self {
      values: replace_attr_values.clone(),
    }
  }
}

impl VisitMut for Visitor {
  fn visit_mut_jsx_opening_element(&mut self, n: &mut JSXOpeningElement) {
    n.attrs.iter_mut().for_each(|attr| {
      if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
        if let Some(JSXAttrValue::Str(str)) = &jsx_attr.value {
          let old_value = str.value.to_string_lossy().into_owned();

          if self.values.contains_key(&old_value) {
            let attr_value = get_attr_value(self.values.get(&old_value).unwrap());
            jsx_attr.value = Some(attr_value);
          }
        }
      }
    });
  }
}

fn get_attr_value(new: &str) -> JSXAttrValue {
  let literal = new.starts_with('{') && new.ends_with('}');
  let s = if literal { &new[1..new.len() - 1] } else { new };

  if literal {
    JSXAttrValue::JSXExprContainer(JSXExprContainer {
      span: DUMMY_SP,
      expr: JSXExpr::Expr(Box::new(Expr::Ident(Ident {
        sym: s.to_string().into(),
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        optional: false,
      }))),
    })
  } else {
    JSXAttrValue::Str(Str {
      span: DUMMY_SP,
      value: s.to_string().into(),
      raw: None,
    })
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

  fn code_test(input: &str, replace_attr_values: HashMap<String, String>, expected: &str) {
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

    module.visit_mut_with(&mut Visitor::new(&core::config::Config {
      replace_attr_values: Some(replace_attr_values),
      ..Default::default()
    }));

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
  fn should_replace_attribute_values_1() {
    let mut replace_attr_values = HashMap::new();
    replace_attr_values.insert("cool".to_string(), "not cool".to_string());
    code_test(
      r#"<div something="cool"/>;"#,
      replace_attr_values,
      r#"<div something="not cool"/>;"#,
    );
  }

  #[test]
  fn should_replace_attribute_values_2() {
    let mut replace_attr_values = HashMap::new();
    replace_attr_values.insert("cool".to_string(), "{props.color}".to_string());
    code_test(
      r#"<div something="cool"/>;"#,
      replace_attr_values,
      r#"<div something={props.color}/>;"#,
    );
  }
}
