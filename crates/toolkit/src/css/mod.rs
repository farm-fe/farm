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
use swc_error_reporters::handler::try_with_handler;

use crate::sourcemap::swc_gen::{build_source_map, AstModule};

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

  let parse_result = parser.parse_all();
  let mut recovered_errors = parser.take_errors();

  if recovered_errors.len() == 0 {
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

/// generate css code from [Stylesheet], return css code and source map
pub fn codegen_css_stylesheet(
  stylesheet: &Stylesheet,
  cm: Option<Arc<SourceMap>>,
  minify: bool,
) -> (String, Option<Vec<u8>>) {
  let mut css_code = String::new();
  let mut source_map = Vec::new();
  let css_writer = BasicCssWriter::new(
    &mut css_code,
    if cm.is_some() {
      Some(&mut source_map)
    } else {
      None
    },
    BasicCssWriterConfig::default(),
  );
  let mut gen = CodeGenerator::new(css_writer, CodegenConfig { minify });

  gen.emit(stylesheet).unwrap();

  if let Some(cm) = cm {
    let src_map = build_source_map(&source_map, cm, AstModule::Css(stylesheet));
    (css_code, Some(src_map))
  } else {
    (css_code, None)
  }
}
