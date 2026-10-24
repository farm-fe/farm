use std::{collections::BTreeMap, path::Path, sync::Arc};

use codemap::{Span, Spanned};

use crate::{
    ast::*, builtin::DISALLOWED_PLAIN_CSS_FUNCTION_NAMES, common::QuoteKind, error::SassResult,
    lexer::Lexer, ContextFlags, Options,
};

use super::{value::ValueParser, BaseParser, StylesheetParser};

pub(crate) struct CssParser<'a> {
    pub toks: Lexer,
    pub path: &'a Path,
    pub empty_span: Span,
    pub flags: ContextFlags,
    pub options: &'a Options<'a>,
}

impl<'a> BaseParser for CssParser<'a> {
    fn toks(&self) -> &Lexer {
        &self.toks
    }

    fn toks_mut(&mut self) -> &mut Lexer {
        &mut self.toks
    }

    fn skip_silent_comment(&mut self) -> SassResult<()> {
        Err((
            "Silent comments aren't allowed in plain CSS.",
            self.toks.current_span(),
        )
            .into())
    }
}

impl<'a> StylesheetParser<'a> for CssParser<'a> {
    fn is_plain_css(&self) -> bool {
        true
    }

    fn is_indented(&self) -> bool {
        false
    }

    fn path(&self) -> &'a Path {
        self.path
    }

    fn options(&self) -> &Options {
        self.options
    }

    fn flags(&self) -> &ContextFlags {
        &self.flags
    }

    fn flags_mut(&mut self) -> &mut ContextFlags {
        &mut self.flags
    }

    fn current_indentation(&self) -> usize {
        0
    }

    fn empty_span(&self) -> Span {
        self.empty_span
    }

    const IDENTIFIER_LIKE: Option<fn(&mut Self) -> SassResult<Spanned<AstExpr>>> =
        Some(Self::parse_identifier_like);

    fn parse_at_rule(
        &mut self,
        _child: fn(&mut Self) -> SassResult<AstStmt>,
    ) -> SassResult<AstStmt> {
        let start = self.toks.cursor();

        self.expect_char('@')?;
        let name = self.parse_interpolated_identifier()?;
        self.whitespace()?;

        match name.as_plain() {
            Some("at-root") | Some("content") | Some("debug") | Some("each") | Some("error")
            | Some("extend") | Some("for") | Some("function") | Some("if") | Some("include")
            | Some("mixin") | Some("return") | Some("warn") | Some("while") => {
                self.almost_any_value(false)?;
                Err((
                    "This at-rule isn't allowed in plain CSS.",
                    self.toks.span_from(start),
                )
                    .into())
            }
            Some("import") => self.parse_css_import_rule(start),
            Some("media") => self.parse_media_rule(start),
            Some("-moz-document") => self._parse_moz_document_rule(name),
            Some("supports") => self.parse_supports_rule(),
            _ => self.unknown_at_rule(name, start),
        }
    }
}

impl<'a> CssParser<'a> {
    pub fn new(
        toks: Lexer,
        options: &'a Options<'a>,
        empty_span: Span,
        file_name: &'a Path,
    ) -> Self {
        CssParser {
            toks,
            path: file_name,
            empty_span,
            flags: ContextFlags::empty(),
            options,
        }
    }

    fn parse_css_import_rule(&mut self, _start: usize) -> SassResult<AstStmt> {
        let url_start = self.toks.cursor();

        let url = if self.toks.next_char_is('u') || self.toks.next_char_is('U') {
            self.parse_dynamic_url()?
                .span(self.toks.span_from(url_start))
        } else {
            let string = self.parse_interpolated_string()?;
            AstExpr::String(
                StringExpr(string.node.as_interpolation(true), QuoteKind::None),
                string.span,
            )
            .span(string.span)
        };

        self.whitespace()?;
        let modifiers = self.try_import_modifiers()?;
        self.expect_statement_separator(Some("@import rule"))?;

        Ok(AstStmt::ImportRule(AstImportRule {
            imports: vec![AstImport::Plain(AstPlainCssImport {
                url: Interpolation::new_with_expr(url),
                modifiers,
                span: self.toks.span_from(url_start),
            })],
        }))
    }

    fn parse_identifier_like(&mut self) -> SassResult<Spanned<AstExpr>> {
        let start = self.toks.cursor();
        let identifier = self.parse_interpolated_identifier()?;
        let plain = identifier.as_plain().unwrap();

        let lower = plain.to_ascii_lowercase();

        if let Some(special_fn) = ValueParser::try_parse_special_function(self, &lower, start)? {
            return Ok(special_fn);
        }

        let before_args = self.toks.cursor();

        if !self.scan_char('(') {
            let span = self.toks.span_from(start);
            return Ok(AstExpr::String(StringExpr(identifier, QuoteKind::None), span).span(span));
        }

        let allow_empty_second_arg = lower == "var";

        let mut arguments = Vec::new();

        if !self.scan_char(')') {
            loop {
                self.whitespace()?;

                let arg_start = self.toks.cursor();
                if allow_empty_second_arg && arguments.len() == 1 && self.toks.next_char_is(')') {
                    arguments.push(AstExpr::String(
                        StringExpr(Interpolation::new_plain(String::new()), QuoteKind::None),
                        self.toks.span_from(arg_start),
                    ));
                    break;
                }

                arguments.push(self.parse_expression_until_comma(true)?.node);
                self.whitespace()?;
                if !self.scan_char(',') {
                    break;
                }
            }
            self.expect_char(')')?;
        }

        let span = self.toks.span_from(start);

        if DISALLOWED_PLAIN_CSS_FUNCTION_NAMES.contains(plain) {
            return Err(("This function isn't allowed in plain CSS.", span).into());
        }

        Ok(
            AstExpr::InterpolatedFunction(Arc::new(InterpolatedFunction {
                name: identifier,
                arguments: ArgumentInvocation {
                    positional: arguments,
                    named: BTreeMap::new(),
                    rest: None,
                    keyword_rest: None,
                    span: self.toks.span_from(before_args),
                },
                span,
            }))
            .span(span),
        )
    }
}
