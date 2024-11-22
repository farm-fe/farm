use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  config::TargetEnv,
  error::CompilationError,
  swc_common::{input::SourceFileInput, DUMMY_SP},
  swc_html_ast::{Attribute, Child, Document, Element, Namespace, Text},
};
use swc_error_reporters::handler::try_with_handler;
use swc_html_codegen::{
  writer::basic::{BasicHtmlWriter, BasicHtmlWriterConfig},
  CodeGenerator, CodegenConfig, Emit,
};
use swc_html_parser::{
  lexer::Lexer,
  parser::{Parser, ParserConfig},
};

use crate::common::{create_swc_source_map, Source};

/// Parse html content to swc_html_ast's [Document], present the ast of HTML.
pub fn parse_html_document(id: &str, content: Arc<String>) -> farmfe_core::error::Result<Document> {
  let (cm, source_file) = create_swc_source_map(Source {
    path: PathBuf::from(id),
    content,
  });
  let html_lexer = Lexer::new(SourceFileInput::from(&*source_file));
  let mut parser = Parser::new(
    html_lexer,
    ParserConfig {
      allow_self_closing: true,
      ..Default::default()
    },
  );

  let parse_result = parser.parse_document();
  let mut recovered_errors = parser.take_errors();

  if recovered_errors.is_empty() {
    match parse_result {
      Err(err) => {
        recovered_errors.push(err);
      }
      Ok(m) => {
        return Ok(m);
      }
    }
  }

  try_with_handler(cm, Default::default(), |handler| {
    for err in recovered_errors {
      err.to_diagnostics(handler).emit();
    }

    Err(anyhow::Error::msg("SyntaxError"))
  })
  .map_err(|e| CompilationError::ParseError {
    resolved_path: id.to_string(),
    msg: if let Some(s) = e.downcast_ref::<String>() {
      s.to_string()
    } else if let Some(s) = e.downcast_ref::<&str>() {
      s.to_string()
    } else {
      "failed to handle with unknown panic message".to_string()
    },
  })
}

/// Generate code from a html [Document] and output the code [String]
pub fn codegen_html_document(document: &Document, minify: bool) -> String {
  let mut html_code = String::new();
  let html_writer = BasicHtmlWriter::new(&mut html_code, None, BasicHtmlWriterConfig::default());
  let mut html_gen = CodeGenerator::new(
    html_writer,
    CodegenConfig {
      minify,
      ..Default::default()
    },
  );

  html_gen.emit(document).unwrap();

  html_code
}

/// Create a <script> [Element]'s ast.
/// Parameter [Vec(&str, &str)] means a set of (name, value) pairs.
pub fn create_element(name: &str, text: Option<&str>, attrs: Vec<(&str, &str)>) -> Element {
  Element {
    span: DUMMY_SP,
    tag_name: name.into(),
    namespace: Namespace::HTML,
    attributes: attrs
      .into_iter()
      .map(|(name, value)| create_attribute(name, Some(value)))
      .collect(),
    children: if text.is_some() {
      vec![Child::Text(Text {
        span: DUMMY_SP,
        data: text.unwrap().into(),
        raw: None,
      })]
    } else {
      vec![]
    },
    content: None,
    is_self_closing: false,
  }
}

pub fn create_attribute(name: &str, value: Option<&str>) -> Attribute {
  Attribute {
    span: DUMMY_SP,
    namespace: None,
    prefix: None,
    name: name.into(),
    raw_name: None,
    value: value.map(|v| v.into()),
    raw_value: None,
  }
}

pub fn get_farm_global_this(namespace: &str, target_env: &TargetEnv) -> String {
  if target_env.is_node() {
    format!("global['{namespace}']")
  } else {
    format!("window['{namespace}']")
  }
}
