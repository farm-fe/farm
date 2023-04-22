use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  error::CompilationError,
  swc_common::{input::SourceFileInput, FileName, SourceMap},
  swc_css_ast::Stylesheet,
};
use swc_css_codegen::{
  writer::basic::{BasicCssWriter, BasicCssWriterConfig},
  CodeGenerator, CodegenConfig, Emit,
};
use swc_css_parser::{
  lexer::Lexer,
  parser::{Parser, ParserConfig},
};

/// parse the input css file content to [Stylesheet]
pub fn parse_css_stylesheet(
  id: &str,
  content: &str,
  cm: Arc<SourceMap>,
) -> farmfe_core::error::Result<Stylesheet> {
  // id must be relative path
  let source_file = cm.new_source_file(FileName::Real(PathBuf::from(id)), content.to_string());
  let config = ParserConfig {
    allow_wrong_line_comments: true,
    // TODO support css modules
    ..Default::default()
  };

  let lexer = Lexer::new(SourceFileInput::from(&*source_file), config);
  let mut parser = Parser::new(lexer, config);

  // TODO may need to show error with parse.take_error()
  parser
    .parse_all()
    .map_err(|e| CompilationError::ParseError {
      resolved_path: id.to_string(),
      source: Some(Box::new(CompilationError::GenericError(format!("{:?}", e)))),
    })
}

pub fn codegen_css_stylesheet(stylesheet: &Stylesheet, minify: bool) -> String {
  let mut css_code = String::new();
  let css_writer = BasicCssWriter::new(&mut css_code, None, BasicCssWriterConfig::default());
  let mut gen = CodeGenerator::new(css_writer, CodegenConfig { minify });

  gen.emit(stylesheet).unwrap();

  css_code
}
