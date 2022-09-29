use std::{path::PathBuf, sync::Arc};

use swc_ecma_codegen::{
  text_writer::{JsWriter, WriteJs},
  Emitter, Node,
};
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax, TsConfig};

use farmfe_core::{
  config::ScriptParserConfig,
  error::{CompilationError, Result},
  module::ModuleType,
  swc_common::{FileName, SourceMap, Mark},
  swc_ecma_ast::{EsVersion, Module as SwcModule, Stmt, CallExpr, Callee, Expr, Ident},
};

/// parse the content of a module to [SwcModule] ast.
pub fn parse_module(
  id: &str,
  content: &str,
  syntax: Syntax,
  target: EsVersion,
  cm: Arc<SourceMap>,
) -> Result<SwcModule> {
  let source_file = cm.new_source_file(FileName::Real(PathBuf::from(id)), content.to_string());
  let input = StringInput::from(&*source_file);
  // TODO support parsing comments
  let lexer = Lexer::new(syntax, target, input, None);
  let mut parser = Parser::new_from(lexer);
  parser
    .parse_module()
    .map_err(|e| CompilationError::ParseError {
      resolved_path: id.to_string(),
      source: Some(Box::new(CompilationError::GenericError(format!("{:?}", e))) as _),
    })
}

/// parse the content of a module to [SwcModule] ast.
pub fn parse_stmt(
  id: &str,
  content: &str,
  syntax: Syntax,
  cm: Arc<SourceMap>,
  top_level: bool,
) -> Result<Stmt> {
  let source_file = cm.new_source_file(FileName::Real(PathBuf::from(id)), content.to_string());
  let input = StringInput::from(&*source_file);
  // TODO support parsing comments
  let mut parser = Parser::new(syntax, input, None);
  parser
    .parse_stmt(top_level)
    .map_err(|e| CompilationError::ParseError {
      resolved_path: id.to_string(),
      source: Some(Box::new(CompilationError::GenericError(format!("{:?}", e))) as _),
    })
}

/// ast codegen, return generated utf8 bytes. using [String::from_utf8] if you want to transform the bytes to string.
/// Example:
/// ```ignore
/// let bytes = codegen(swc_ast, cm);
/// let code = String::from_utf8(bytes).unwrap();
/// ```
pub fn codegen_module(
  ast: &SwcModule,
  target: EsVersion,
  cm: Arc<SourceMap>,
) -> std::result::Result<Vec<u8>, std::io::Error> {
  let mut buf = vec![];

  {
    // TODO support source map
    let wr = Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, None)) as Box<dyn WriteJs>;

    let mut emitter = Emitter {
      cfg: swc_ecma_codegen::Config {
        target,
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

/// Get [ModuleType] from the resolved id's extension, return [ModuleType::Custom(ext)] if the extension is not internally supported.
/// Panic if the id do not has a extension.
pub fn module_type_from_id(id: &str) -> ModuleType {
  let path = PathBuf::from(id);

  match path
    .extension()
    .unwrap_or_else(|| panic!("extension of {} is None", id))
    .to_str()
    .unwrap()
  {
    "ts" => ModuleType::Ts,
    "tsx" => ModuleType::Tsx,
    "js" | "mjs" | "cjs" => ModuleType::Js,
    "jsx" => ModuleType::Jsx,
    "css" => ModuleType::Css,
    "html" => ModuleType::Html,
    ext => ModuleType::Custom(ext.to_string()),
  }
}

/// return [None] if module type is not script
pub fn syntax_from_module_type(
  module_type: &ModuleType,
  config: ScriptParserConfig,
) -> Option<Syntax> {
  match module_type {
    ModuleType::Js => Some(Syntax::Es(EsConfig {
      jsx: false,
      ..config.es_config
    })),
    ModuleType::Jsx => Some(Syntax::Es(EsConfig {
      jsx: true,
      ..config.es_config
    })),
    ModuleType::Ts => Some(Syntax::Typescript(TsConfig {
      tsx: false,
      ..config.ts_config
    })),
    ModuleType::Tsx => Some(Syntax::Typescript(TsConfig {
      tsx: true,
      ..config.ts_config
    })),
    _ => None,
  }
}

/// Whether the call expr is commonjs require.
/// A call expr is commonjs require if:
/// * callee is an identifier named `require`
/// * arguments is a single string literal
/// * require is a global variable
pub fn is_commonjs_require(unresolved_mark: Mark, call_expr: &CallExpr) -> bool {
  if let Callee::Expr(box Expr::Ident(Ident { span, sym, .. })) = &call_expr.callee {
    sym == "require" && span.ctxt.outer() == unresolved_mark
  } else {
    false
  }
}