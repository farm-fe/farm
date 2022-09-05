use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  error::CompilationError,
  swc_common::{input::SourceFileInput, FileName, SourceMap, DUMMY_SP},
  swc_html_ast::{Attribute, Child, Document, Element, Namespace, Text},
};
use swc_html_codegen::{
  writer::basic::{BasicHtmlWriter, BasicHtmlWriterConfig},
  CodeGenerator, CodegenConfig, Emit,
};
use swc_html_parser::{
  lexer::Lexer,
  parser::{Parser, ParserConfig},
};

/// Parse html content to swc_html_ast's [Document], present the ast of HTML.
pub fn parse_html_document(
  id: &str,
  content: &str,
  cm: Arc<SourceMap>,
) -> farmfe_core::error::Result<Document> {
  let source_file = cm.new_source_file(FileName::Real(PathBuf::from(id)), content.to_string());
  let html_lexer = Lexer::new(SourceFileInput::from(&*source_file));
  let mut parser = Parser::new(html_lexer, ParserConfig::default());

  parser
    .parse_document()
    .map_err(|e| CompilationError::ParseError {
      resolved_path: id.to_string(),
      source: Some(Box::new(CompilationError::GenericError(format!("{:?}", e)))),
    })
}

/// Generate code from a html [Document] and output the code [String]
pub fn codegen_html_document(document: &Document) -> String {
  let mut html_code = String::new();
  let html_writer = BasicHtmlWriter::new(&mut html_code, None, BasicHtmlWriterConfig::default());
  let mut html_gen = CodeGenerator::new(
    html_writer,
    CodegenConfig {
      minify: false,
      ..Default::default()
    },
  );

  html_gen.emit(document).unwrap();

  html_code
}

/// Create a <script> [Element]'s ast.
/// Parameter [Vec(&str, &str)] means a set of (name, value) pairs.
pub fn create_element_with_attrs(name: &str, attrs: Vec<(&str, &str)>) -> Element {
  Element {
    span: DUMMY_SP,
    tag_name: name.into(),
    namespace: Namespace::HTML,
    attributes: attrs
      .into_iter()
      .map(|(name, value)| Attribute {
        span: DUMMY_SP,
        namespace: None,
        prefix: None,
        name: name.into(),
        raw_name: None,
        value: Some(value.into()),
        raw_value: None,
      })
      .collect(),
    children: vec![],
    content: None,
    is_self_closing: false,
  }
}

pub fn create_element_with_text(name: &str, text: &str) -> Element {
  Element {
    span: DUMMY_SP,
    tag_name: name.into(),
    namespace: Namespace::HTML,
    attributes: vec![],
    children: vec![Child::Text(Text {
      span: DUMMY_SP,
      data: text.into(),
      raw: None,
    })],
    content: None,
    is_self_closing: false,
  }
}
