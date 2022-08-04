use std::{path::PathBuf, sync::Arc};

use swc_ecma_codegen::{
  text_writer::{JsWriter, WriteJs},
  Emitter, Node,
};
use swc_ecma_parser::{EsConfig, Parser, StringInput, Syntax, TsConfig};

use farmfe_core::{
  error::{CompilationError, Result},
  module::ModuleType,
  swc_common::{FileName, SourceMap},
  swc_ecma_ast::Module as SwcModule,
};

/// parse the content of a module to [SwcModule] ast.
pub fn parse_module(
  id: &str,
  content: &str,
  syntax: Syntax,
  cm: Arc<SourceMap>,
) -> Result<SwcModule> {
  let source_file = cm.new_source_file(FileName::Real(PathBuf::from(id)), content.to_string());
  let input = StringInput::from(&*source_file);
  // TODO support parsing comments
  let mut parser = Parser::new(syntax, input, None);
  parser
    .parse_module()
    .map_err(|e| CompilationError::ParseError {
      id: id.to_string(),
      source: Some(Box::new(CompilationError::GenericError(format!("{:?}", e))) as _),
    })
}

/// ast codegen, return generated utf8 bytes. using [String::from_utf8] if you want to transform the bytes to string.
/// Example:
/// ```rust
/// let bytes = codegen(swc_ast, cm);
/// let code = String::from_utf8(bytes).unwrap();
/// ```
pub fn codegen_module(
  ast: &SwcModule,
  cm: Arc<SourceMap>,
) -> std::result::Result<Vec<u8>, std::io::Error> {
  let mut buf = vec![];

  {
    // TODO support source map
    let wr = Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, None)) as Box<dyn WriteJs>;

    let mut emitter = Emitter {
      cfg: swc_ecma_codegen::Config {
        target: Default::default(),
        ascii_only: false,
        minify: false,
      },
      comments: None,
      cm,
      wr,
    };

    ast.emit_with(&mut emitter)?;
  }

  Ok(buf)
}

/// Get [ModuleType] from the resolved id's extension, return [None] if the extension is not supported
/// TODO support extra extension
pub fn module_type_from_id(id: &str) -> Option<ModuleType> {
  if id.ends_with(".ts") {
    Some(ModuleType::Ts)
  } else if id.ends_with(".tsx") {
    Some(ModuleType::Tsx)
  } else if id.ends_with(".js") || id.ends_with(".mjs") || id.ends_with(".cjs") {
    Some(ModuleType::Js)
  } else if id.ends_with(".jsx") {
    Some(ModuleType::Jsx)
  } else {
    None
  }
}

/// TODO support custom [EsConfig] and [TsConfig]
pub fn syntax_from_module_type(module_type: &ModuleType) -> Option<Syntax> {
  match module_type {
    ModuleType::Js => Some(Syntax::Es(Default::default())),
    ModuleType::Jsx => Some(Syntax::Es(EsConfig {
      jsx: true,
      ..Default::default()
    })),
    ModuleType::Ts => Some(Syntax::Typescript(Default::default())),
    ModuleType::Tsx => Some(Syntax::Typescript(TsConfig {
      tsx: true,
      ..Default::default()
    })),
    _ => None,
  }
}
