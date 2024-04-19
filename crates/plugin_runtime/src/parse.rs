/// This file is not used and is only for reference purposes.
use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::ModuleType,
  swc_common::BytePos,
  swc_ecma_ast::{EsVersion, Expr, Module, ModuleItem, Script, Stmt},
  swc_ecma_parser::{lexer::Lexer, Parser, StringInput},
};
use farmfe_toolkit::script::syntax_from_module_type;

pub struct ParserUtils<'a>(Parser<Lexer<'a>>);

impl<'a> ParserUtils<'a> {
  pub fn parse<'b, T>(source: T, context: &Arc<CompilationContext>) -> ParserUtils<'b>
  where
    T: Into<&'b str>,
  {
    let syntax =
      syntax_from_module_type(&ModuleType::Js, context.config.script.parser.clone()).unwrap();

    let input = StringInput::new(source.into(), BytePos::DUMMY, BytePos::DUMMY);
    // let comments = SingleThreadedComments::default();
    let lexer = Lexer::new(syntax, EsVersion::EsNext, input, None);

    let mut parser = Parser::new_from(lexer);
    // parser.parse_stmt(top_level)
    ParserUtils(parser)
  }

  pub fn parse_stmt(mut self, top_level: bool) -> Stmt {
    self.0.parse_stmt(top_level).unwrap()
  }
}

macro_rules! impl_parse {
  ($name:ident, $r:ty) => {
    impl<'a> ParserUtils<'a> {
      pub fn $name(mut self) -> $r {
        self.0.$name().unwrap()
      }
    }
  };
}

impl_parse!(parse_module, Module);
impl_parse!(parse_module_item, ModuleItem);
impl_parse!(parse_script, Script);
impl_parse!(parse_expr, Box<Expr>);
