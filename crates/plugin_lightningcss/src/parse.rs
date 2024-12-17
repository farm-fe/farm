use farmfe_core::error::CompilationError;
use lightningcss::stylesheet::{ParserFlags, ParserOptions, StyleSheet};

struct ParseCssModuleResult {
  ast: String,
  comments: Vec<String>,
}

pub fn parse_css_stylesheet(
  id: &str,
  orig_content: String,
) -> farmfe_core::error::Result<ParseCssModuleResult> {
  let options = ParserOptions {
    filename: id.clone().to_string(),
    css_modules: None,
    source_index: 0,
    warnings: None,
    error_recovery: false,
    flags: ParserFlags::default(),
  };

  match StyleSheet::parse(&orig_content, options) {
    Ok(stylesheet) => Ok(ParseCssModuleResult {
      ast: "".to_string(),
      comments: Vec::new(),
    }),
    Err(err) => Err(CompilationError::ParseError {
      resolved_path: id.to_string(),
      msg: format!("CSS parse error: {:?}", err),
    }),
  }
}
