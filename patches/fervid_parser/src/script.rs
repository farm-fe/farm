use fervid_core::{SfcScriptBlock, SfcScriptLang};
use swc_core::{
    common::Span,
    ecma::ast::{Expr, Module, Pat},
};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};
use swc_html_ast::{Child, Element};

use crate::{ParseError, SfcParser, error::ParseErrorKind};

impl SfcParser<'_, '_, '_> {
    /// Parses the `<script>` and `<script setup>`, both in EcmaScript and TypeScript
    pub fn parse_sfc_script_element(
        &mut self,
        element: Element,
    ) -> Result<Option<SfcScriptBlock>, ParseError> {
        // Find `setup` and `lang`
        let mut is_setup = false;
        let mut is_setup_seen = false;
        let mut is_lang_seen = false;
        let mut lang = SfcScriptLang::Es;
        for attr in element.attributes.iter() {
            match attr.name.as_str() {
                "setup" => {
                    if is_setup_seen {
                        self.report_error(ParseError {
                            kind: ParseErrorKind::DuplicateAttribute,
                            span: attr.span,
                        });
                    }

                    is_setup = true;
                    is_setup_seen = true;
                }
                "lang" if is_lang_seen => self.errors.push(ParseError {
                    kind: ParseErrorKind::DuplicateAttribute,
                    span: attr.span,
                }),
                "lang" => {
                    is_lang_seen = true;

                    lang = match attr.value.as_ref().map(|v| v.as_str()) {
                        Some("ts" | "typescript") => SfcScriptLang::Typescript,
                        None | Some("js" | "javascript") => SfcScriptLang::Es,
                        Some(_) => {
                            return Err(ParseError {
                                kind: ParseErrorKind::UnsupportedLang,
                                span: attr.span,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        // `<script>` should always have a single `Text` child
        let script_content = match element.children.first() {
            Some(Child::Text(t)) => t,
            Some(_) => {
                return Err(ParseError {
                    kind: ParseErrorKind::UnexpectedNonRawTextContent,
                    span: element.span,
                });
            }
            None if self.ignore_empty => {
                return Ok(None);
            }
            None => {
                // Allow empty
                return Ok(Some(SfcScriptBlock {
                    content: Box::new(Module {
                        span: element.span,
                        body: Vec::new(),
                        shebang: None,
                    }),
                    lang,
                    is_setup,
                    span: element.span,
                }));
            }
        };

        // Ignore empty unless allowed
        if self.ignore_empty && script_content.data.trim().is_empty() {
            return Ok(None);
        }

        let module_content = self.parse_module(
            &script_content.data,
            if matches!(lang, SfcScriptLang::Typescript) {
                Syntax::Typescript(TsSyntax::default())
            } else {
                Syntax::Es(EsSyntax::default())
            },
            script_content.span,
        )?;

        Ok(Some(SfcScriptBlock {
            content: Box::new(module_content),
            lang,
            is_setup,
            span: element.span,
        }))
    }

    #[inline]
    pub fn parse_module(
        &mut self,
        raw: &str,
        syntax: Syntax,
        span: Span,
    ) -> Result<Module, ParseError> {
        let lexer = Lexer::new(
            syntax,
            // EsVersion defaults to es5
            Default::default(),
            StringInput::new(raw, span.lo, span.hi),
            Some(&self.comments),
        );

        let mut parser = Parser::new_from(lexer);
        let parse_result = parser.parse_module();

        // Map errors to EcmaSyntaxError
        self.errors
            .extend(parser.take_errors().into_iter().map(From::from));

        parse_result.map_err(From::from)
    }

    pub fn parse_expr(
        &mut self,
        raw: &str,
        syntax: Syntax,
        span: Span,
    ) -> Result<Box<Expr>, ParseError> {
        let lexer = Lexer::new(
            syntax,
            // EsVersion defaults to es5
            Default::default(),
            StringInput::new(raw, span.lo, span.hi),
            Some(&self.comments),
        );

        let mut parser = Parser::new_from(lexer);
        let parse_result = parser.parse_expr();

        // Map errors to EcmaSyntaxError
        self.errors
            .extend(parser.take_errors().into_iter().map(From::from));

        parse_result.map_err(From::from)
    }

    pub fn parse_pat(&mut self, raw: &str, syntax: Syntax, span: Span) -> Result<Pat, ParseError> {
        let lexer = Lexer::new(
            syntax,
            // EsVersion defaults to es5
            Default::default(),
            StringInput::new(raw, span.lo, span.hi),
            Some(&self.comments),
        );

        let mut parser = Parser::new_from(lexer);
        let parse_result = parser.parse_pat();

        // Map errors to EcmaSyntaxError
        self.errors
            .extend(parser.take_errors().into_iter().map(From::from));

        parse_result.map_err(From::from)
    }
}
