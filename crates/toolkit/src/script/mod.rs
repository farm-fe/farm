use std::sync::Arc;

use swc_ecma_codegen::{
  text_writer::{JsWriter, WriteJs},
  Emitter, Node,
};
use swc_ecma_parser::{
  lexer::{input::SourceFileInput, Lexer},
  Parser, StringInput, Syntax,
};

use farmfe_core::{
  config::{comments::CommentsConfig, custom::get_config_output_ascii_only},
  context::CompilationContext,
  error::{CompilationError, Result},
  module::ModuleId,
  swc_common::{
    comments::{Comments, SingleThreadedComments},
    BytePos, LineCol, SourceMap,
  },
  swc_ecma_ast::{EsVersion, Module as SwcModule, Stmt},
};

use swc_ecma_visit::VisitMutWith;
use swc_error_reporters::handler::try_with_handler;
use swc_try_with::ResetSpanVisitMut;

use crate::{minify::comments::minify_comments, sourcemap::create_swc_source_map};

pub use utils::ParseScriptModuleResult;

pub mod analyze_statement;
pub mod concatenate_modules;
pub mod constant;
pub mod idents_collector;
pub mod merge_swc_globals;
pub mod module2cjs;
pub mod module_system;
pub mod swc_try_with;
pub mod utils;
pub mod wrap_farm_runtime;

pub use module_system::*;
pub use utils::*;

/// parse the content of a module to [SwcModule] ast.
pub fn parse_module(
  module_id: &ModuleId,
  content: Arc<String>,
  syntax: Syntax,
  target: EsVersion,
) -> Result<ParseScriptModuleResult> {
  let (cm, source_file) = create_swc_source_map(module_id, content);

  let input = StringInput::from(&*source_file);
  let comments = SingleThreadedComments::default();
  let lexer = Lexer::new(syntax, target, input, Some(&comments));

  let mut parser = Parser::new_from(lexer);
  let module = parser.parse_module();
  let mut recovered_errors = parser.take_errors();

  match module {
    Err(err) => {
      recovered_errors.push(err);
    }
    Ok(m) => {
      return Ok(ParseScriptModuleResult {
        ast: m,
        comments,
        source_map: cm,
      });
    }
  }
  try_with_handler(cm, Default::default(), |handler| {
    for err in recovered_errors {
      err.into_diagnostic(handler).emit();
    }

    Err(anyhow::Error::msg("SyntaxError"))
  })
  .map_err(|e| CompilationError::ParseError {
    resolved_path: module_id.to_string(),
    msg: if let Some(s) = e.downcast_ref::<String>() {
      eprintln!("recovered_errors: {}", s);
      s.to_string()
    } else if let Some(s) = e.downcast_ref::<&str>() {
      eprintln!("recovered_errors: {}", s);
      s.to_string()
    } else {
      "failed to handle with unknown panic message".to_string()
    },
  })
}

/// parse the content of a module to [SwcModule] ast.
pub fn parse_stmt(id: &str, content: &str, top_level: bool) -> Result<Stmt> {
  let (_, source_file) = create_swc_source_map(&id.into(), Arc::new(content.to_string()));
  let input = SourceFileInput::from(&*source_file);
  let mut parser = Parser::new(Syntax::Es(Default::default()), input, None);
  let mut stmt = parser
    .parse_stmt(top_level)
    .map_err(|e| CompilationError::ParseError {
      resolved_path: id.to_string(),
      msg: format!("{e:?}"),
    })?;

  stmt.visit_mut_with(&mut ResetSpanVisitMut);
  Ok(stmt)
}

pub struct CodeGenCommentsConfig<'a> {
  pub comments: &'a SingleThreadedComments,
  pub config: &'a CommentsConfig,
}

pub fn create_codegen_config(context: &CompilationContext) -> swc_ecma_codegen::Config {
  let minify = context.config.minify.enabled();
  let target = context.config.script.target.clone();
  let ascii_only = get_config_output_ascii_only(&context.config);

  swc_ecma_codegen::Config::default()
    .with_minify(minify)
    .with_target(target)
    .with_ascii_only(ascii_only)
    .with_omit_last_semi(true)
}

/// ast codegen, return generated utf8 bytes. using [String::from_utf8] if you want to transform the bytes to string.
/// Example:
/// ```ignore
/// let bytes = codegen(swc_ast, cm);
/// let code = String::from_utf8(bytes).unwrap();
/// ```
pub fn codegen_module(
  ast: &SwcModule,
  cm: Arc<SourceMap>,
  src_map: Option<&mut Vec<(BytePos, LineCol)>>,
  codegen_config: swc_ecma_codegen::Config,
  comments_cfg: Option<CodeGenCommentsConfig>,
) -> std::result::Result<Vec<u8>, std::io::Error> {
  let mut buf = vec![];

  {
    let wr = Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, src_map)) as Box<dyn WriteJs>;
    // let cfg = swc_ecma_codegen::Config::default()
    //   .with_minify(minify)
    //   .with_target(target)
    //   .with_omit_last_semi(true)
    //   .with_ascii_only(false);

    if let Some(comments_cfg) = &comments_cfg {
      minify_comments(comments_cfg.comments, comments_cfg.config);
    }

    let comments = comments_cfg.map(|c| c.comments as &dyn Comments);

    let mut emitter = Emitter {
      cfg: codegen_config,
      comments,
      cm,
      wr,
    };

    ast.emit_with(&mut emitter)?;
  }

  Ok(buf)
}
