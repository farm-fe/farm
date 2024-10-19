use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  error::CompilationError,
  regex::Regex,
  swc_common::{comments::SingleThreadedComments, input::SourceFileInput},
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

use crate::common::{build_source_map, create_swc_source_map, Source};

pub struct ParseCssModuleResult {
  pub ast: Stylesheet,
  pub comments: SingleThreadedComments,
}

/// parse the input css file content to [Stylesheet]
pub fn parse_css_stylesheet(
  id: &str,
  orig_content: Arc<String>,
) -> farmfe_core::error::Result<ParseCssModuleResult> {
  // swc_css_parser does not support parsing invalid css, so we need to replace known invalid css here
  // 1. replace --: '' to --farm-empty: ''
  let mut content = orig_content.replace("--:", "--farm-empty:");
  // 2. replace filter: xxx.Microsoft.xxx to filter: "xxx.Microsoft.xxx" using regex. fix #1557
  let regex = Regex::new(r#"filter:\s*([^'"]*?)\.Microsoft\.(.*?)(;|\})"#).unwrap();
  content = regex
    .replace_all(&content, "filter:\"$1.Microsoft.$2\"$3")
    .to_string();
  // // 3. replace invalid operator, eg: top: -8px/2 + 1 to top: "-8px/2 + 1" using regex. fix #1748
  // let regex = Regex::new(r#":\s*([^;{}]*?\d\s+\s\d[^;{}]*?)\s*(;|\})"#).unwrap();
  // content = regex.replace_all(&content, ":\"$1\"$2").to_string();

  let (cm, source_file) = create_swc_source_map(Source {
    path: PathBuf::from(id),
    content: Arc::new(content),
  });

  let config = ParserConfig {
    allow_wrong_line_comments: true,
    css_modules: true,
    legacy_nesting: true,
    legacy_ie: true,
  };

  let comments = SingleThreadedComments::default();
  let lexer = Lexer::new(
    SourceFileInput::from(&*source_file),
    Some(&comments),
    config,
  );
  let mut parser = Parser::new(lexer, config);

  let parse_result = parser.parse_all();
  let mut recovered_errors = parser.take_errors();

  if recovered_errors.len() == 0 {
    match parse_result {
      Err(err) => {
        recovered_errors.push(err);
      }
      Ok(m) => {
        return Ok(ParseCssModuleResult { ast: m, comments });
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
  source: Option<Source>,
  minify: bool,
) -> (String, Option<String>) {
  let mut css_code = String::new();
  let mut mappings = Vec::new();
  let css_writer = BasicCssWriter::new(
    &mut css_code,
    if source.is_some() {
      Some(&mut mappings)
    } else {
      None
    },
    BasicCssWriterConfig::default(),
  );
  let mut gen = CodeGenerator::new(css_writer, CodegenConfig { minify });

  gen.emit(stylesheet).unwrap();

  if let Some(source) = source {
    let (cm, _) = create_swc_source_map(source);
    let map = build_source_map(cm, &mappings);
    let mut src_map = vec![];
    map.to_writer(&mut src_map).unwrap();

    (css_code, Some(String::from_utf8(src_map).unwrap()))
  } else {
    (css_code, None)
  }
}
