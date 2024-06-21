use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  farm_profile_function,
  module::Module,
  swc_common::{comments::SingleThreadedComments, Mark},
  swc_ecma_ast::{Module as EcmaAstModule, ModuleItem},
  swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax},
};
use farmfe_toolkit::{
  common::{create_swc_source_map, Source},
  script::swc_try_with::resolve_module_mark,
};

pub fn get_module_mark(
  module: &Module,
  cloned_module: &mut EcmaAstModule,
  context: &Arc<CompilationContext>,
) -> (Mark, Mark) {
  farm_profile_function!();
  if module.meta.as_script().unresolved_mark == 0 && module.meta.as_script().top_level_mark == 0 {
    resolve_module_mark(cloned_module, module.module_type.is_typescript(), context)
  } else {
    let unresolved_mark = Mark::from_u32(module.meta.as_script().unresolved_mark);
    let top_level_mark = Mark::from_u32(module.meta.as_script().top_level_mark);
    (unresolved_mark, top_level_mark)
  }
}

pub fn parse_module_item(string: &str) -> Result<ModuleItem> {
  let (_, source_file) = create_swc_source_map(Source {
    path: PathBuf::from("unknown"),
    content: Arc::new(string.to_string()),
  });

  let input = StringInput::from(&*source_file);
  let comments = SingleThreadedComments::default();
  let lexer = Lexer::new(
    Syntax::Es(EsConfig::default()),
    farmfe_core::swc_ecma_ast::EsVersion::Es5,
    input,
    Some(&comments),
  );

  let mut parser = Parser::new_from(lexer);

  parser
    .parse_module_item()
    .map_err(|msg| CompilationError::ParseError {
      resolved_path: "unknown temp parser".to_string(),
      msg: format!("failed parse content, cause: {:#?}", msg),
    })
}

pub trait OptionToResult<T> {
  fn to_result<S: ToString>(self, error: S) -> Result<T>;
  fn to_result_with_error(self, error: CompilationError) -> Result<T>;
}

impl<T> OptionToResult<T> for std::option::Option<T> {
  fn to_result<S: ToString>(self, error: S) -> Result<T> {
    match self {
      Some(v) => Ok(v),
      None => Err(CompilationError::GenericError(error.to_string())),
    }
  }

  fn to_result_with_error(self, error: CompilationError) -> Result<T> {
    match self {
      Some(v) => Ok(v),
      None => Err(error),
    }
  }
}
