use std::{
    cell::Cell,
    collections::{BTreeMap, HashSet},
    ffi::OsString,
    mem,
    path::{Path, PathBuf},
    sync::Arc,
};

use codemap::{Span, Spanned};

use crate::{
    ast::*,
    common::{unvendor, Identifier, QuoteKind},
    error::SassResult,
    lexer::Lexer,
    utils::{is_name, is_name_start, is_plain_css_import, opposite_bracket},
    ContextFlags, Options, Token,
};

use super::{
    value::{Predicate, ValueParser},
    BaseParser, DeclarationOrBuffer, ScssParser, VariableDeclOrInterpolation, RESERVED_IDENTIFIERS,
};

/// Default implementations are oriented towards the SCSS syntax, as both CSS and
/// SCSS share the behavior
pub(crate) trait StylesheetParser<'a>: BaseParser + Sized {
    // todo: make constant?
    fn is_plain_css(&self) -> bool;
    // todo: make constant?
    fn is_indented(&self) -> bool;
    fn options(&self) -> &Options;
    fn path(&self) -> &Path;
    fn empty_span(&self) -> Span;
    fn current_indentation(&self) -> usize;
    fn flags(&self) -> &ContextFlags;
    fn flags_mut(&mut self) -> &mut ContextFlags;

    #[allow(clippy::type_complexity)]
    const IDENTIFIER_LIKE: Option<fn(&mut Self) -> SassResult<Spanned<AstExpr>>> = None;

    fn parse_style_rule_selector(&mut self) -> SassResult<Interpolation> {
        self.almost_any_value(false)
    }

    fn expect_statement_separator(&mut self, _name: Option<&str>) -> SassResult<()> {
        self.whitespace_without_comments();
        match self.toks().peek() {
            Some(Token {
                kind: ';' | '}', ..
            })
            | None => Ok(()),
            _ => {
                self.expect_char(';')?;
                unreachable!();
            }
        }
    }

    fn at_end_of_statement(&self) -> bool {
        matches!(
            self.toks().peek(),
            Some(Token {
                kind: ';' | '}' | '{',
                ..
            }) | None
        )
    }

    fn looking_at_children(&mut self) -> SassResult<bool> {
        Ok(matches!(self.toks().peek(), Some(Token { kind: '{', .. })))
    }

    fn scan_else(&mut self, _if_indentation: usize) -> SassResult<bool> {
        let start = self.toks().cursor();

        self.whitespace()?;

        if self.scan_char('@') {
            if self.scan_identifier("else", true)? {
                return Ok(true);
            }

            if self.scan_identifier("elseif", true)? {
                // todo: deprecation warning here
                let new_cursor = self.toks().cursor() - 2;
                self.toks_mut().set_cursor(new_cursor);
                return Ok(true);
            }
        }

        self.toks_mut().set_cursor(start);

        Ok(false)
    }

    fn parse_children(
        &mut self,
        child: fn(&mut Self) -> SassResult<AstStmt>,
    ) -> SassResult<Vec<AstStmt>> {
        self.expect_char('{')?;
        self.whitespace_without_comments();
        let mut children = Vec::new();

        let mut found_matching_brace = false;

        while let Some(tok) = self.toks().peek() {
            match tok.kind {
                '$' => children.push(AstStmt::VariableDecl(
                    self.parse_variable_declaration_without_namespace(None, None)?,
                )),
                '/' => match self.toks().peek_n(1) {
                    Some(Token { kind: '/', .. }) => {
                        children.push(self.parse_silent_comment()?);
                        self.whitespace_without_comments();
                    }
                    Some(Token { kind: '*', .. }) => {
                        children.push(AstStmt::LoudComment(self.parse_loud_comment()?));
                        self.whitespace_without_comments();
                    }
                    _ => children.push(child(self)?),
                },
                ';' => {
                    self.toks_mut().next();
                    self.whitespace_without_comments();
                }
                '}' => {
                    self.expect_char('}')?;
                    found_matching_brace = true;
                    break;
                }
                _ => children.push(child(self)?),
            }
        }

        if !found_matching_brace {
            return Err(("expected \"}\".", self.toks().current_span()).into());
        }

        Ok(children)
    }

    fn parse_statements(
        &mut self,
        statement: fn(&mut Self) -> SassResult<Option<AstStmt>>,
    ) -> SassResult<Vec<AstStmt>> {
        let mut stmts = Vec::new();
        self.whitespace_without_comments();
        while let Some(tok) = self.toks().peek() {
            match tok.kind {
                '$' => stmts.push(AstStmt::VariableDecl(
                    self.parse_variable_declaration_without_namespace(None, None)?,
                )),
                '/' => match self.toks().peek_n(1) {
                    Some(Token { kind: '/', .. }) => {
                        stmts.push(self.parse_silent_comment()?);
                        self.whitespace_without_comments();
                    }
                    Some(Token { kind: '*', .. }) => {
                        stmts.push(AstStmt::LoudComment(self.parse_loud_comment()?));
                        self.whitespace_without_comments();
                    }
                    _ => {
                        if let Some(stmt) = statement(self)? {
                            stmts.push(stmt);
                        }
                    }
                },
                ';' => {
                    self.toks_mut().next();
                    self.whitespace_without_comments();
                }
                _ => {
                    if let Some(stmt) = statement(self)? {
                        stmts.push(stmt);
                    }
                }
            }
        }

        Ok(stmts)
    }

    // todo: rename
    fn __parse(&mut self) -> SassResult<StyleSheet> {
        let mut style_sheet = StyleSheet::new(
            self.is_plain_css(),
            self.options()
                .fs
                .canonicalize(self.path())
                .unwrap_or_else(|_| self.path().to_path_buf()),
        );

        // Allow a byte-order mark at the beginning of the document.
        self.scan_char('\u{feff}');

        style_sheet.body = self.parse_statements(|parser| {
            if parser.next_matches("@charset") {
                parser.expect_char('@')?;
                parser.expect_identifier("charset", false)?;
                parser.whitespace()?;
                parser.parse_string()?;
                return Ok(None);
            }

            Ok(Some(parser.parse_statement()?))
        })?;

        for (idx, child) in style_sheet.body.iter().enumerate() {
            match child {
                AstStmt::VariableDecl(_) | AstStmt::LoudComment(_) | AstStmt::SilentComment(_) => {
                    continue
                }
                AstStmt::Use(..) => style_sheet.uses.push(idx),
                AstStmt::Forward(..) => style_sheet.forwards.push(idx),
                _ => break,
            }
        }

        Ok(style_sheet)
    }

    fn looking_at_expression(&mut self) -> bool {
        let character = if let Some(c) = self.toks().peek() {
            c
        } else {
            return false;
        };

        match character.kind {
            '.' => !matches!(self.toks().peek_n(1), Some(Token { kind: '.', .. })),
            '!' => match self.toks().peek_n(1) {
                Some(Token {
                    kind: 'i' | 'I', ..
                })
                | None => true,
                Some(Token { kind, .. }) => kind.is_ascii_whitespace(),
            },
            '(' | '/' | '[' | '\'' | '"' | '#' | '+' | '-' | '\\' | '$' | '&' => true,
            c => is_name_start(c) || c.is_ascii_digit(),
        }
    }

    fn parse_argument_declaration(&mut self) -> SassResult<ArgumentDeclaration> {
        self.expect_char('(')?;
        self.whitespace()?;

        let mut arguments = Vec::new();
        let mut named = HashSet::new();

        let mut rest_argument: Option<Identifier> = None;

        while self.toks_mut().next_char_is('$') {
            let name_start = self.toks().cursor();
            let name = Identifier::from(self.parse_variable_name()?);
            let name_span = self.toks_mut().span_from(name_start);
            self.whitespace()?;

            let mut default_value: Option<AstExpr> = None;

            if self.scan_char(':') {
                self.whitespace()?;
                default_value = Some(self.parse_expression_until_comma(false)?.node);
            } else if self.scan_char('.') {
                self.expect_char('.')?;
                self.expect_char('.')?;
                self.whitespace()?;
                rest_argument = Some(name);
                break;
            }

            arguments.push(Argument {
                name,
                default: default_value,
            });

            if !named.insert(name) {
                return Err(("Duplicate argument.", name_span).into());
            }

            if !self.scan_char(',') {
                break;
            }
            self.whitespace()?;
        }
        self.expect_char(')')?;

        Ok(ArgumentDeclaration {
            args: arguments,
            rest: rest_argument,
        })
    }

    fn plain_at_rule_name(&mut self) -> SassResult<String> {
        self.expect_char('@')?;
        let name = self.parse_identifier(false, false)?;
        self.whitespace()?;
        Ok(name)
    }

    fn with_children(
        &mut self,
        child: fn(&mut Self) -> SassResult<AstStmt>,
    ) -> SassResult<Spanned<Vec<AstStmt>>> {
        let start = self.toks().cursor();
        let children = self.parse_children(child)?;
        let span = self.toks_mut().span_from(start);
        self.whitespace_without_comments();
        Ok(Spanned {
            node: children,
            span,
        })
    }

    fn parse_at_root_query(&mut self) -> SassResult<Interpolation> {
        let mut buffer = Interpolation::new();
        self.expect_char('(')?;
        buffer.add_char('(');

        self.whitespace()?;

        buffer.add_expr(self.parse_expression(None, None, None)?);

        if self.scan_char(':') {
            self.whitespace()?;
            buffer.add_char(':');
            buffer.add_char(' ');
            buffer.add_expr(self.parse_expression(None, None, None)?);
        }

        self.expect_char(')')?;
        self.whitespace()?;
        buffer.add_char(')');

        Ok(buffer)
    }

    fn parse_at_root_rule(&mut self, start: usize) -> SassResult<AstStmt> {
        Ok(AstStmt::AtRootRule(if self.toks_mut().next_char_is('(') {
            let query_start = self.toks().cursor();
            let query = self.parse_at_root_query()?;
            let query_span = self.toks_mut().span_from(query_start);
            self.whitespace()?;
            let children = self.with_children(Self::parse_statement)?.node;

            AstAtRootRule {
                query: Some(Spanned {
                    node: query,
                    span: query_span,
                }),
                body: children,
                span: self.toks_mut().span_from(start),
            }
        } else if self.looking_at_children()? {
            let children = self.with_children(Self::parse_statement)?.node;
            AstAtRootRule {
                query: None,
                body: children,
                span: self.toks_mut().span_from(start),
            }
        } else {
            let child = self.parse_style_rule(None, None)?;
            AstAtRootRule {
                query: None,
                body: vec![child],
                span: self.toks_mut().span_from(start),
            }
        }))
    }

    fn parse_content_rule(&mut self, start: usize) -> SassResult<AstStmt> {
        if !self.flags().in_mixin() {
            return Err((
                "@content is only allowed within mixin declarations.",
                self.toks_mut().span_from(start),
            )
                .into());
        }

        self.whitespace()?;

        let args = if self.toks_mut().next_char_is('(') {
            self.parse_argument_invocation(true, false)?
        } else {
            ArgumentInvocation::empty(self.toks().current_span())
        };

        self.expect_statement_separator(Some("@content rule"))?;

        self.flags_mut().set(ContextFlags::FOUND_CONTENT_RULE, true);

        Ok(AstStmt::ContentRule(AstContentRule { args }))
    }

    fn parse_debug_rule(&mut self) -> SassResult<AstStmt> {
        let value = self.parse_expression(None, None, None)?;
        self.expect_statement_separator(Some("@debug rule"))?;

        Ok(AstStmt::Debug(AstDebugRule {
            value: value.node,
            span: value.span,
        }))
    }

    fn parse_each_rule(
        &mut self,
        child: fn(&mut Self) -> SassResult<AstStmt>,
    ) -> SassResult<AstStmt> {
        let was_in_control_directive = self.flags().in_control_flow();
        self.flags_mut().set(ContextFlags::IN_CONTROL_FLOW, true);

        let mut variables = vec![Identifier::from(self.parse_variable_name()?)];
        self.whitespace()?;
        while self.scan_char(',') {
            self.whitespace()?;
            variables.push(Identifier::from(self.parse_variable_name()?));
            self.whitespace()?;
        }

        self.expect_identifier("in", false)?;
        self.whitespace()?;

        let list = self.parse_expression(None, None, None)?.node;

        let body = self.with_children(child)?.node;

        self.flags_mut()
            .set(ContextFlags::IN_CONTROL_FLOW, was_in_control_directive);

        Ok(AstStmt::Each(AstEach {
            variables,
            list,
            body,
        }))
    }

    fn parse_disallowed_at_rule(&mut self, start: usize) -> SassResult<AstStmt> {
        self.almost_any_value(false)?;
        Err((
            "This at-rule is not allowed here.",
            self.toks_mut().span_from(start),
        )
            .into())
    }

    fn parse_error_rule(&mut self) -> SassResult<AstStmt> {
        let value = self.parse_expression(None, None, None)?;
        self.expect_statement_separator(Some("@error rule"))?;
        Ok(AstStmt::ErrorRule(AstErrorRule {
            value: value.node,
            span: value.span,
        }))
    }

    fn parse_extend_rule(&mut self, start: usize) -> SassResult<AstStmt> {
        if !self.flags().in_style_rule()
            && !self.flags().in_mixin()
            && !self.flags().in_content_block()
        {
            return Err((
                "@extend may only be used within style rules.",
                self.toks_mut().span_from(start),
            )
                .into());
        }

        let value = self.almost_any_value(false)?;

        let is_optional = self.scan_char('!');

        if is_optional {
            self.expect_identifier("optional", false)?;
        }

        self.expect_statement_separator(Some("@extend rule"))?;

        Ok(AstStmt::Extend(AstExtendRule {
            value,
            is_optional,
            span: self.toks_mut().span_from(start),
        }))
    }

    fn parse_for_rule(
        &mut self,
        child: fn(&mut Self) -> SassResult<AstStmt>,
    ) -> SassResult<AstStmt> {
        let was_in_control_directive = self.flags().in_control_flow();
        self.flags_mut().set(ContextFlags::IN_CONTROL_FLOW, true);

        let var_start = self.toks().cursor();
        let variable = Spanned {
            node: Identifier::from(self.parse_variable_name()?),
            span: self.toks_mut().span_from(var_start),
        };
        self.whitespace()?;

        self.expect_identifier("from", false)?;
        self.whitespace()?;

        let exclusive: Cell<Option<bool>> = Cell::new(None);

        let from = self.parse_expression(
            Some(&|parser| {
                if !parser.looking_at_identifier() {
                    return Ok(false);
                }
                Ok(if parser.scan_identifier("to", false)? {
                    exclusive.set(Some(true));
                    true
                } else if parser.scan_identifier("through", false)? {
                    exclusive.set(Some(false));
                    true
                } else {
                    false
                })
            }),
            None,
            None,
        )?;

        let is_exclusive = match exclusive.get() {
            Some(b) => b,
            None => {
                return Err((
                    "Expected \"to\" or \"through\".",
                    self.toks().current_span(),
                )
                    .into())
            }
        };

        self.whitespace()?;

        let to = self.parse_expression(None, None, None)?;

        let body = self.with_children(child)?.node;

        self.flags_mut()
            .set(ContextFlags::IN_CONTROL_FLOW, was_in_control_directive);

        Ok(AstStmt::For(AstFor {
            variable,
            from,
            to,
            is_exclusive,
            body,
        }))
    }

    fn parse_function_rule(&mut self, start: usize) -> SassResult<AstStmt> {
        let name_start = self.toks().cursor();
        let name = self.parse_identifier(true, false)?;
        let name_span = self.toks_mut().span_from(name_start);
        self.whitespace()?;
        let arguments = self.parse_argument_declaration()?;

        if self.flags().in_mixin() || self.flags().in_content_block() {
            return Err((
                "Mixins may not contain function declarations.",
                self.toks_mut().span_from(start),
            )
                .into());
        } else if self.flags().in_control_flow() {
            return Err((
                "Functions may not be declared in control directives.",
                self.toks_mut().span_from(start),
            )
                .into());
        }

        if RESERVED_IDENTIFIERS.contains(&unvendor(&name)) {
            return Err(("Invalid function name.", self.toks_mut().span_from(start)).into());
        }

        self.whitespace()?;

        let children = self.with_children(Self::function_child)?.node;

        Ok(AstStmt::FunctionDecl(AstFunctionDecl {
            name: Spanned {
                node: Identifier::from(name),
                span: name_span,
            },
            arguments,
            body: children,
        }))
    }

    fn parse_variable_declaration_with_namespace(&mut self) -> SassResult<AstVariableDecl> {
        let start = self.toks().cursor();
        let namespace = self.parse_identifier(false, false)?;
        let namespace_span = self.toks_mut().span_from(start);
        self.expect_char('.')?;
        self.parse_variable_declaration_without_namespace(
            Some(Spanned {
                node: Identifier::from(namespace),
                span: namespace_span,
            }),
            Some(start),
        )
    }

    fn function_child(&mut self) -> SassResult<AstStmt> {
        let start = self.toks().cursor();
        if !self.toks_mut().next_char_is('@') {
            match self.parse_variable_declaration_with_namespace() {
                Ok(decl) => return Ok(AstStmt::VariableDecl(decl)),
                Err(e) => {
                    self.toks_mut().set_cursor(start);
                    let stmt = match self.parse_declaration_or_style_rule() {
                        Ok(stmt) => stmt,
                        Err(..) => return Err(e),
                    };

                    let (is_style_rule, span) = match stmt {
                        AstStmt::RuleSet(ruleset) => (true, ruleset.span),
                        AstStmt::Style(style) => (false, style.span),
                        _ => unreachable!(),
                    };

                    return Err((
                        format!(
                            "@function rules may not contain {}.",
                            if is_style_rule {
                                "style rules"
                            } else {
                                "declarations"
                            }
                        ),
                        span,
                    )
                        .into());
                }
            }
        }

        return match self.plain_at_rule_name()?.as_str() {
            "debug" => self.parse_debug_rule(),
            "each" => self.parse_each_rule(Self::function_child),
            "else" => self.parse_disallowed_at_rule(start),
            "error" => self.parse_error_rule(),
            "for" => self.parse_for_rule(Self::function_child),
            "if" => self.parse_if_rule(Self::function_child),
            "return" => self.parse_return_rule(),
            "warn" => self.parse_warn_rule(),
            "while" => self.parse_while_rule(Self::function_child),
            _ => self.parse_disallowed_at_rule(start),
        };
    }

    fn parse_if_rule(
        &mut self,
        child: fn(&mut Self) -> SassResult<AstStmt>,
    ) -> SassResult<AstStmt> {
        let if_indentation = self.current_indentation();

        let was_in_control_directive = self.flags().in_control_flow();
        self.flags_mut().set(ContextFlags::IN_CONTROL_FLOW, true);
        let condition = self.parse_expression(None, None, None)?.node;
        let body = self.parse_children(child)?;
        self.whitespace_without_comments();

        let mut clauses = vec![AstIfClause { condition, body }];

        let mut last_clause: Option<Vec<AstStmt>> = None;

        while self.scan_else(if_indentation)? {
            self.whitespace()?;
            if self.scan_identifier("if", false)? {
                self.whitespace()?;
                let condition = self.parse_expression(None, None, None)?.node;
                let body = self.parse_children(child)?;
                clauses.push(AstIfClause { condition, body });
            } else {
                last_clause = Some(self.parse_children(child)?);
                break;
            }
        }

        self.flags_mut()
            .set(ContextFlags::IN_CONTROL_FLOW, was_in_control_directive);
        self.whitespace_without_comments();

        Ok(AstStmt::If(AstIf {
            if_clauses: clauses,
            else_clause: last_clause,
        }))
    }

    fn try_parse_import_supports_function(&mut self) -> SassResult<Option<AstSupportsCondition>> {
        if !self.looking_at_interpolated_identifier() {
            return Ok(None);
        }

        let start = self.toks().cursor();
        let name = self.parse_interpolated_identifier()?;
        debug_assert!(name.as_plain() != Some("not"));

        if !self.scan_char('(') {
            self.toks_mut().set_cursor(start);
            return Ok(None);
        }

        let value = self.parse_interpolated_declaration_value(true, true, true)?;
        self.expect_char(')')?;

        Ok(Some(AstSupportsCondition::Function { name, args: value }))
    }

    fn parse_import_supports_query(&mut self) -> SassResult<AstSupportsCondition> {
        Ok(if self.scan_identifier("not", false)? {
            self.whitespace()?;
            AstSupportsCondition::Negation(Box::new(self.supports_condition_in_parens()?))
        } else if self.toks_mut().next_char_is('(') {
            self.parse_supports_condition()?
        } else {
            match self.try_parse_import_supports_function()? {
                Some(function) => function,
                None => {
                    let start = self.toks().cursor();
                    let name = self.parse_expression(None, None, None)?;
                    self.expect_char(':')?;
                    self.supports_declaration_value(name.node, start)?
                }
            }
        })
    }

    fn try_import_modifiers(&mut self) -> SassResult<Option<Interpolation>> {
        // Exit before allocating anything if we're not looking at any modifiers, as
        // is the most common case.
        if !self.looking_at_interpolated_identifier() && !self.toks_mut().next_char_is('(') {
            return Ok(None);
        }

        let mut buffer = Interpolation::new();

        loop {
            if self.looking_at_interpolated_identifier() {
                if !buffer.is_empty() {
                    buffer.add_char(' ');
                }

                let identifier = self.parse_interpolated_identifier()?;
                let name = identifier.as_plain().map(str::to_ascii_lowercase);
                buffer.add_interpolation(identifier);

                if name.as_deref() != Some("and") && self.scan_char('(') {
                    if name.as_deref() == Some("supports") {
                        let query = self.parse_import_supports_query()?;
                        let is_declaration =
                            matches!(query, AstSupportsCondition::Declaration { .. });

                        if !is_declaration {
                            buffer.add_char('(');
                        }

                        buffer.add_expr(AstExpr::Supports(Arc::new(query)).span(self.empty_span()));

                        if !is_declaration {
                            buffer.add_char(')');
                        }
                    } else {
                        buffer.add_char('(');
                        buffer.add_interpolation(
                            self.parse_interpolated_declaration_value(true, true, true)?,
                        );
                        buffer.add_char(')');
                    }

                    self.expect_char(')')?;
                    self.whitespace()?;
                } else {
                    self.whitespace()?;
                    if self.scan_char(',') {
                        buffer.add_char(',');
                        buffer.add_char(' ');
                        buffer.add_interpolation(self.parse_media_query_list()?);
                        return Ok(Some(buffer));
                    }
                }
            } else if self.toks_mut().next_char_is('(') {
                if !buffer.is_empty() {
                    buffer.add_char(' ');
                }

                buffer.add_interpolation(self.parse_media_query_list()?);
                return Ok(Some(buffer));
            } else {
                return Ok(Some(buffer));
            }
        }
    }

    fn try_url_contents(&mut self, name: Option<&str>) -> SassResult<Option<Interpolation>> {
        let start = self.toks().cursor();
        if !self.scan_char('(') {
            return Ok(None);
        }
        self.whitespace_without_comments();

        // Match Ruby Sass's behavior: parse a raw URL() if possible, and if not
        // backtrack and re-parse as a function expression.
        let mut buffer = Interpolation::new();
        buffer.add_string(name.unwrap_or("url").to_owned());
        buffer.add_char('(');

        while let Some(next) = self.toks().peek() {
            match next.kind {
                '\\' => buffer.add_string(self.parse_escape(false)?),
                '!' | '%' | '&' | '*'..='~' | '\u{80}'..=char::MAX => {
                    self.toks_mut().next();
                    buffer.add_char(next.kind);
                }
                '#' => {
                    if matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. })) {
                        let interpolation = self.parse_single_interpolation()?;
                        buffer.add_interpolation(interpolation);
                    } else {
                        self.toks_mut().next();
                        buffer.add_char(next.kind);
                    }
                }
                ')' => {
                    self.toks_mut().next();
                    buffer.add_char(next.kind);
                    return Ok(Some(buffer));
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.whitespace_without_comments();
                    if !self.toks_mut().next_char_is(')') {
                        break;
                    }
                }
                _ => break,
            }
        }

        self.toks_mut().set_cursor(start);

        Ok(None)
    }

    fn parse_dynamic_url(&mut self) -> SassResult<AstExpr> {
        let start = self.toks().cursor();
        self.expect_identifier("url", false)?;

        Ok(match self.try_url_contents(None)? {
            Some(contents) => AstExpr::String(
                StringExpr(contents, QuoteKind::None),
                self.toks_mut().span_from(start),
            ),
            None => AstExpr::InterpolatedFunction(Arc::new(InterpolatedFunction {
                name: Interpolation::new_plain("url".to_owned()),
                arguments: self.parse_argument_invocation(false, false)?,
                span: self.toks_mut().span_from(start),
            })),
        })
    }

    fn parse_import_argument(&mut self, start: usize) -> SassResult<AstImport> {
        if self.toks_mut().next_char_is('u') || self.toks_mut().next_char_is('U') {
            let url = self.parse_dynamic_url()?;
            self.whitespace()?;
            let modifiers = self.try_import_modifiers()?;
            return Ok(AstImport::Plain(AstPlainCssImport {
                url: Interpolation::new_with_expr(url.span(self.toks_mut().span_from(start))),
                modifiers,
                span: self.toks_mut().span_from(start),
            }));
        }

        let start = self.toks().cursor();
        let url = self.parse_string()?;
        let raw_url = self.toks().raw_text(start);
        self.whitespace()?;
        let modifiers = self.try_import_modifiers()?;

        let span = self.toks_mut().span_from(start);

        if is_plain_css_import(&url) || modifiers.is_some() {
            Ok(AstImport::Plain(AstPlainCssImport {
                url: Interpolation::new_plain(raw_url),
                modifiers,
                span,
            }))
        } else {
            // todo: try parseImportUrl
            Ok(AstImport::Sass(AstSassImport { url, span }))
        }
    }

    fn parse_import_rule(&mut self, start: usize) -> SassResult<AstStmt> {
        let mut imports = Vec::new();

        loop {
            self.whitespace()?;
            let argument = self.parse_import_argument(self.toks().cursor())?;

            // todo: _inControlDirective
            if (self.flags().in_control_flow() || self.flags().in_mixin()) && argument.is_dynamic()
            {
                self.parse_disallowed_at_rule(start)?;
            }

            imports.push(argument);
            self.whitespace()?;

            if !self.scan_char(',') {
                break;
            }
        }

        Ok(AstStmt::ImportRule(AstImportRule { imports }))
    }

    fn parse_public_identifier(&mut self) -> SassResult<String> {
        let start = self.toks().cursor();
        let ident = self.parse_identifier(true, false)?;
        Self::assert_public(&ident, self.toks_mut().span_from(start))?;

        Ok(ident)
    }

    fn parse_include_rule(&mut self) -> SassResult<AstStmt> {
        let mut namespace: Option<Spanned<Identifier>> = None;

        let name_start = self.toks().cursor();
        let mut name = self.parse_identifier(false, false)?;

        if self.scan_char('.') {
            let namespace_span = self.toks_mut().span_from(name_start);
            namespace = Some(Spanned {
                node: Identifier::from(name),
                span: namespace_span,
            });
            name = self.parse_public_identifier()?;
        } else {
            name = name.replace('_', "-");
        }

        let name = Identifier::from(name);
        let name_span = self.toks_mut().span_from(name_start);

        self.whitespace()?;

        let args = if self.toks_mut().next_char_is('(') {
            self.parse_argument_invocation(true, false)?
        } else {
            ArgumentInvocation::empty(self.toks().current_span())
        };

        self.whitespace()?;

        let content_args = if self.scan_identifier("using", false)? {
            self.whitespace()?;
            let args = self.parse_argument_declaration()?;
            self.whitespace()?;
            Some(args)
        } else {
            None
        };

        let mut content_block: Option<AstContentBlock> = None;

        if content_args.is_some() || self.looking_at_children()? {
            let content_args = content_args.unwrap_or_else(ArgumentDeclaration::empty);
            let was_in_content_block = self.flags().in_content_block();
            self.flags_mut().set(ContextFlags::IN_CONTENT_BLOCK, true);
            let body = self.with_children(Self::parse_statement)?.node;
            content_block = Some(AstContentBlock {
                args: content_args,
                body,
            });
            self.flags_mut()
                .set(ContextFlags::IN_CONTENT_BLOCK, was_in_content_block);
        } else {
            self.expect_statement_separator(None)?;
        }

        Ok(AstStmt::Include(AstInclude {
            namespace,
            name: Spanned {
                node: name,
                span: name_span,
            },
            args,
            content: content_block,
            span: name_span,
        }))
    }

    fn parse_media_rule(&mut self, start: usize) -> SassResult<AstStmt> {
        let query_start = self.toks().cursor();
        let query = self.parse_media_query_list()?;
        let query_span = self.toks_mut().span_from(query_start);

        let body = self.with_children(Self::parse_statement)?.node;

        Ok(AstStmt::Media(AstMedia {
            query,
            query_span,
            body,
            span: self.toks_mut().span_from(start),
        }))
    }

    fn parse_interpolated_string(&mut self) -> SassResult<Spanned<StringExpr>> {
        let start = self.toks().cursor();
        let quote = match self.toks_mut().next() {
            Some(Token {
                kind: kind @ ('"' | '\''),
                ..
            }) => kind,
            Some(..) | None => unreachable!("Expected string."),
        };

        let mut buffer = Interpolation::new();

        let mut found_match = false;

        while let Some(next) = self.toks().peek() {
            match next.kind {
                c if c == quote => {
                    self.toks_mut().next();
                    found_match = true;
                    break;
                }
                '\n' => break,
                '\\' => {
                    match self.toks().peek_n(1) {
                        // todo: if (second == $cr) scanner.scanChar($lf);
                        // we basically need to stop normalizing to gain parity
                        Some(Token { kind: '\n', .. }) => {
                            self.toks_mut().next();
                            self.toks_mut().next();
                        }
                        _ => buffer.add_char(self.consume_escaped_char()?),
                    }
                }
                '#' => {
                    if matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. })) {
                        buffer.add_interpolation(self.parse_single_interpolation()?);
                    } else {
                        self.toks_mut().next();
                        buffer.add_char(next.kind);
                    }
                }
                _ => {
                    buffer.add_char(next.kind);
                    self.toks_mut().next();
                }
            }
        }

        if !found_match {
            return Err((
                format!("Expected {quote}.", quote = quote),
                self.toks().current_span(),
            )
                .into());
        }

        Ok(Spanned {
            node: StringExpr(buffer, QuoteKind::Quoted),
            span: self.toks_mut().span_from(start),
        })
    }

    fn parse_return_rule(&mut self) -> SassResult<AstStmt> {
        let value = self.parse_expression(None, None, None)?;
        self.expect_statement_separator(None)?;
        Ok(AstStmt::Return(AstReturn {
            val: value.node,
            span: value.span,
        }))
    }

    fn parse_mixin_rule(&mut self, start: usize) -> SassResult<AstStmt> {
        let name = Identifier::from(self.parse_identifier(true, false)?);
        self.whitespace()?;
        let args = if self.toks_mut().next_char_is('(') {
            self.parse_argument_declaration()?
        } else {
            ArgumentDeclaration::empty()
        };

        if self.flags().in_mixin() || self.flags().in_content_block() {
            return Err((
                "Mixins may not contain mixin declarations.",
                self.toks_mut().span_from(start),
            )
                .into());
        } else if self.flags().in_control_flow() {
            return Err((
                "Mixins may not be declared in control directives.",
                self.toks_mut().span_from(start),
            )
                .into());
        }

        self.whitespace()?;

        let old_found_content_rule = self.flags().found_content_rule();
        self.flags_mut()
            .set(ContextFlags::FOUND_CONTENT_RULE, false);
        self.flags_mut().set(ContextFlags::IN_MIXIN, true);

        let body = self.with_children(Self::parse_statement)?.node;

        let has_content = self.flags_mut().found_content_rule();

        self.flags_mut()
            .set(ContextFlags::FOUND_CONTENT_RULE, old_found_content_rule);
        self.flags_mut().set(ContextFlags::IN_MIXIN, false);

        Ok(AstStmt::Mixin(AstMixin {
            name,
            args,
            body,
            has_content,
        }))
    }

    fn _parse_moz_document_rule(&mut self, _name: Interpolation) -> SassResult<AstStmt> {
        todo!("special cased @-moz-document not yet implemented")
    }

    fn unknown_at_rule(&mut self, name: Interpolation, start: usize) -> SassResult<AstStmt> {
        let was_in_unknown_at_rule = self.flags().in_unknown_at_rule();
        self.flags_mut().set(ContextFlags::IN_UNKNOWN_AT_RULE, true);

        let value: Option<Interpolation> =
            if !self.toks_mut().next_char_is('!') && !self.at_end_of_statement() {
                Some(self.almost_any_value(false)?)
            } else {
                None
            };

        let children = if self.looking_at_children()? {
            Some(self.with_children(Self::parse_statement)?.node)
        } else {
            self.expect_statement_separator(None)?;
            None
        };

        self.flags_mut()
            .set(ContextFlags::IN_UNKNOWN_AT_RULE, was_in_unknown_at_rule);

        Ok(AstStmt::UnknownAtRule(AstUnknownAtRule {
            name,
            value,
            body: children,
            span: self.toks_mut().span_from(start),
        }))
    }

    fn try_supports_operation(
        &mut self,
        interpolation: &Interpolation,
        _start: usize,
    ) -> SassResult<Option<AstSupportsCondition>> {
        if interpolation.contents.len() != 1 {
            return Ok(None);
        }

        let expression = match interpolation.contents.first() {
            Some(InterpolationPart::Expr(e)) => e,
            Some(InterpolationPart::String(..)) => return Ok(None),
            None => unreachable!(),
        };

        let before_whitespace = self.toks().cursor();
        self.whitespace()?;

        let mut operation: Option<AstSupportsCondition> = None;
        let mut operator: Option<String> = None;

        while self.looking_at_identifier() {
            if let Some(operator) = &operator {
                self.expect_identifier(operator, false)?;
            } else if self.scan_identifier("and", false)? {
                operator = Some("and".to_owned());
            } else if self.scan_identifier("or", false)? {
                operator = Some("or".to_owned());
            } else {
                self.toks_mut().set_cursor(before_whitespace);
                return Ok(None);
            }

            self.whitespace()?;

            let right = self.supports_condition_in_parens()?;
            operation = Some(AstSupportsCondition::Operation {
                left: Box::new(operation.unwrap_or_else(|| {
                    AstSupportsCondition::Interpolation(expression.clone().node)
                })),
                operator: operator.clone(),
                right: Box::new(right),
            });
            self.whitespace()?;
        }

        Ok(operation)
    }

    fn supports_declaration_value(
        &mut self,
        name: AstExpr,
        start: usize,
    ) -> SassResult<AstSupportsCondition> {
        let value = match &name {
            AstExpr::String(StringExpr(text, QuoteKind::None), ..)
                if text.initial_plain().starts_with("--") =>
            {
                let text = self.parse_interpolated_declaration_value(false, false, true)?;
                AstExpr::String(
                    StringExpr(text, QuoteKind::None),
                    self.toks_mut().span_from(start),
                )
            }
            _ => {
                self.whitespace()?;
                self.parse_expression(None, None, None)?.node
            }
        };

        Ok(AstSupportsCondition::Declaration { name, value })
    }

    fn supports_condition_in_parens(&mut self) -> SassResult<AstSupportsCondition> {
        let start = self.toks().cursor();

        if self.looking_at_interpolated_identifier() {
            let identifier = self.parse_interpolated_identifier()?;
            let ident_span = self.toks_mut().span_from(start);

            if identifier.as_plain().unwrap_or("").to_ascii_lowercase() == "not" {
                return Err((r#""not" is not a valid identifier here."#, ident_span).into());
            }

            if self.scan_char('(') {
                let arguments = self.parse_interpolated_declaration_value(true, true, true)?;
                self.expect_char(')')?;
                return Ok(AstSupportsCondition::Function {
                    name: identifier,
                    args: arguments,
                });
            } else if identifier.contents.len() != 1
                || !matches!(
                    identifier.contents.first(),
                    Some(InterpolationPart::Expr(..))
                )
            {
                return Err(("Expected @supports condition.", ident_span).into());
            } else {
                match identifier.contents.first() {
                    Some(InterpolationPart::Expr(e)) => {
                        return Ok(AstSupportsCondition::Interpolation(e.clone().node))
                    }
                    _ => unreachable!(),
                }
            }
        }

        self.expect_char('(')?;
        self.whitespace()?;

        if self.scan_identifier("not", false)? {
            self.whitespace()?;
            let condition = self.supports_condition_in_parens()?;
            self.expect_char(')')?;
            return Ok(AstSupportsCondition::Negation(Box::new(condition)));
        } else if self.toks_mut().next_char_is('(') {
            let condition = self.parse_supports_condition()?;
            self.expect_char(')')?;
            return Ok(condition);
        }

        // Unfortunately, we may have to backtrack here. The grammar is:
        //
        //       Expression ":" Expression
        //     | InterpolatedIdentifier InterpolatedAnyValue?
        //
        // These aren't ambiguous because this `InterpolatedAnyValue` is forbidden
        // from containing a top-level colon, but we still have to parse the full
        // expression to figure out if there's a colon after it.
        //
        // We could avoid the overhead of a full expression parse by looking ahead
        // for a colon (outside of balanced brackets), but in practice we expect the
        // vast majority of real uses to be `Expression ":" Expression`, so it makes
        // sense to parse that case faster in exchange for less code complexity and
        // a slower backtracking case.

        let name: AstExpr;
        let name_start = self.toks().cursor();
        let was_in_parens = self.flags().in_parens();

        let expr = self.parse_expression(None, None, None);
        let found_colon = self.expect_char(':');
        match (expr, found_colon) {
            (Ok(val), Ok(..)) => {
                name = val.node;
            }
            (Ok(..), Err(e)) | (Err(e), Ok(..)) | (Err(e), Err(..)) => {
                self.toks_mut().set_cursor(name_start);
                self.flags_mut().set(ContextFlags::IN_PARENS, was_in_parens);

                let identifier = self.parse_interpolated_identifier()?;

                // todo: superfluous clone?
                if let Some(operation) = self.try_supports_operation(&identifier, name_start)? {
                    self.expect_char(')')?;
                    return Ok(operation);
                }

                // If parsing an expression fails, try to parse an
                // `InterpolatedAnyValue` instead. But if that value runs into a
                // top-level colon, then this is probably intended to be a declaration
                // after all, so we rethrow the declaration-parsing error.
                let mut contents = Interpolation::new();
                contents.add_interpolation(identifier);
                contents.add_interpolation(
                    self.parse_interpolated_declaration_value(true, true, false)?,
                );

                if self.toks_mut().next_char_is(':') {
                    return Err(e);
                }

                self.expect_char(')')?;

                return Ok(AstSupportsCondition::Anything { contents });
            }
        }

        let declaration = self.supports_declaration_value(name, start)?;
        self.expect_char(')')?;

        Ok(declaration)
    }

    fn parse_supports_condition(&mut self) -> SassResult<AstSupportsCondition> {
        if self.scan_identifier("not", false)? {
            self.whitespace()?;
            return Ok(AstSupportsCondition::Negation(Box::new(
                self.supports_condition_in_parens()?,
            )));
        }

        let mut condition = self.supports_condition_in_parens()?;
        self.whitespace()?;

        let mut operator: Option<String> = None;

        while self.looking_at_identifier() {
            if let Some(operator) = &operator {
                self.expect_identifier(operator, false)?;
            } else if self.scan_identifier("or", false)? {
                operator = Some("or".to_owned());
            } else {
                self.expect_identifier("and", false)?;
                operator = Some("and".to_owned());
            }

            self.whitespace()?;
            let right = self.supports_condition_in_parens()?;
            condition = AstSupportsCondition::Operation {
                left: Box::new(condition),
                operator: operator.clone(),
                right: Box::new(right),
            };
            self.whitespace()?;
        }

        Ok(condition)
    }

    fn parse_supports_rule(&mut self) -> SassResult<AstStmt> {
        let condition = self.parse_supports_condition()?;
        self.whitespace()?;
        let children = self.with_children(Self::parse_statement)?;

        Ok(AstStmt::Supports(AstSupportsRule {
            condition,
            body: children.node,
            span: children.span,
        }))
    }

    fn parse_warn_rule(&mut self) -> SassResult<AstStmt> {
        let value = self.parse_expression(None, None, None)?;
        self.expect_statement_separator(Some("@warn rule"))?;
        Ok(AstStmt::Warn(AstWarn {
            value: value.node,
            span: value.span,
        }))
    }

    fn parse_while_rule(
        &mut self,
        child: fn(&mut Self) -> SassResult<AstStmt>,
    ) -> SassResult<AstStmt> {
        let was_in_control_directive = self.flags().in_control_flow();
        self.flags_mut().set(ContextFlags::IN_CONTROL_FLOW, true);

        let condition = self.parse_expression(None, None, None)?.node;

        let body = self.with_children(child)?.node;

        self.flags_mut()
            .set(ContextFlags::IN_CONTROL_FLOW, was_in_control_directive);

        Ok(AstStmt::While(AstWhile { condition, body }))
    }
    fn parse_forward_rule(&mut self, start: usize) -> SassResult<AstStmt> {
        let url = PathBuf::from(self.parse_url_string()?);
        self.whitespace()?;

        let prefix = if self.scan_identifier("as", false)? {
            self.whitespace()?;
            let prefix = self.parse_identifier(true, false)?;
            self.expect_char('*')?;
            self.whitespace()?;
            Some(prefix)
        } else {
            None
        };

        let mut shown_mixins_and_functions: Option<HashSet<Identifier>> = None;
        let mut shown_variables: Option<HashSet<Identifier>> = None;
        let mut hidden_mixins_and_functions: Option<HashSet<Identifier>> = None;
        let mut hidden_variables: Option<HashSet<Identifier>> = None;

        if self.scan_identifier("show", false)? {
            let members = self.parse_member_list()?;
            shown_mixins_and_functions = Some(members.0);
            shown_variables = Some(members.1);
        } else if self.scan_identifier("hide", false)? {
            let members = self.parse_member_list()?;
            hidden_mixins_and_functions = Some(members.0);
            hidden_variables = Some(members.1);
        }

        let config = self.parse_configuration(true)?;

        self.expect_statement_separator(Some("@forward rule"))?;
        let span = self.toks_mut().span_from(start);

        if !self.flags().is_use_allowed() {
            return Err((
                "@forward rules must be written before any other rules.",
                span,
            )
                .into());
        }

        Ok(AstStmt::Forward(
            if let (Some(shown_mixins_and_functions), Some(shown_variables)) =
                (shown_mixins_and_functions, shown_variables)
            {
                AstForwardRule::show(
                    url,
                    shown_mixins_and_functions,
                    shown_variables,
                    prefix,
                    config,
                    span,
                )
            } else if let (Some(hidden_mixins_and_functions), Some(hidden_variables)) =
                (hidden_mixins_and_functions, hidden_variables)
            {
                AstForwardRule::hide(
                    url,
                    hidden_mixins_and_functions,
                    hidden_variables,
                    prefix,
                    config,
                    span,
                )
            } else {
                AstForwardRule::new(url, prefix, config, span)
            },
        ))
    }

    fn parse_member_list(&mut self) -> SassResult<(HashSet<Identifier>, HashSet<Identifier>)> {
        let mut identifiers = HashSet::new();
        let mut variables = HashSet::new();

        loop {
            self.whitespace()?;

            // todo: withErrorMessage("Expected variable, mixin, or function name"
            if self.toks_mut().next_char_is('$') {
                variables.insert(Identifier::from(self.parse_variable_name()?));
            } else {
                identifiers.insert(Identifier::from(self.parse_identifier(true, false)?));
            }

            self.whitespace()?;

            if !self.scan_char(',') {
                break;
            }
        }

        Ok((identifiers, variables))
    }

    fn parse_url_string(&mut self) -> SassResult<String> {
        // todo: real uri parsing
        self.parse_string()
    }

    fn use_namespace(
        &mut self,
        url: &Path,
        _start: usize,
        url_span: Span,
    ) -> SassResult<Option<String>> {
        if self.scan_identifier("as", false)? {
            self.whitespace()?;
            return Ok(if self.scan_char('*') {
                None
            } else {
                Some(self.parse_identifier(false, false)?)
            });
        }

        let base_name = url
            .file_name()
            .map_or_else(OsString::new, ToOwned::to_owned);
        let base_name = base_name.to_string_lossy();
        let dot = base_name.find('.');

        let start = if base_name.starts_with('_') { 1 } else { 0 };
        let end = dot.unwrap_or(base_name.len());
        let namespace = if url.to_string_lossy().starts_with("sass:") {
            return Ok(Some(url.to_string_lossy().into_owned()));
        } else {
            &base_name[start..end]
        };

        let mut toks = Lexer::new_from_string(namespace, url_span);

        // if namespace is empty, avoid attempting to parse an identifier from
        // an empty string, as there will be no span to emit
        let identifier = if namespace.is_empty() {
            Err(("", self.empty_span()).into())
        } else {
            mem::swap(self.toks_mut(), &mut toks);
            let ident = self.parse_identifier(false, false);
            mem::swap(self.toks_mut(), &mut toks);
            ident
        };

        match (identifier, toks.peek().is_none()) {
            (Ok(i), true) => Ok(Some(i)),
            _ => {
                Err((
                    format!(
                        "The default namespace \"{namespace}\" is not a valid Sass identifier.\n\nRecommendation: add an \"as\" clause to define an explicit namespace.",
                        namespace = namespace
                    ),
                    self.toks_mut().span_from(start)
                ).into())
            }
        }
    }

    fn parse_configuration(
        &mut self,
        // default=false
        allow_guarded: bool,
    ) -> SassResult<Option<Vec<ConfiguredVariable>>> {
        if !self.scan_identifier("with", false)? {
            return Ok(None);
        }

        let mut variable_names = HashSet::new();
        let mut configuration = Vec::new();
        self.whitespace()?;
        self.expect_char('(')?;

        loop {
            self.whitespace()?;
            let var_start = self.toks().cursor();
            let name = Identifier::from(self.parse_variable_name()?);
            let name_span = self.toks_mut().span_from(var_start);
            self.whitespace()?;
            self.expect_char(':')?;
            self.whitespace()?;
            let expr = self.parse_expression_until_comma(false)?;

            let mut is_guarded = false;
            let flag_start = self.toks().cursor();
            if allow_guarded && self.scan_char('!') {
                let flag = self.parse_identifier(false, false)?;
                if flag == "default" {
                    is_guarded = true;
                    self.whitespace()?;
                } else {
                    return Err(
                        ("Invalid flag name.", self.toks_mut().span_from(flag_start)).into(),
                    );
                }
            }

            let span = self.toks_mut().span_from(var_start);
            if variable_names.contains(&name) {
                return Err(("The same variable may only be configured once.", span).into());
            }

            variable_names.insert(name);
            configuration.push(ConfiguredVariable {
                name: Spanned {
                    node: name,
                    span: name_span,
                },
                expr,
                is_guarded,
            });

            if !self.scan_char(',') {
                break;
            }
            self.whitespace()?;
            if !self.looking_at_expression() {
                break;
            }
        }

        self.expect_char(')')?;

        Ok(Some(configuration))
    }

    fn parse_use_rule(&mut self, start: usize) -> SassResult<AstStmt> {
        let url_start = self.toks().cursor();
        let url = self.parse_url_string()?;
        let url_span = self.toks().span_from(url_start);
        self.whitespace()?;

        let path = PathBuf::from(url);

        let namespace = self.use_namespace(path.as_ref(), start, url_span)?;
        self.whitespace()?;
        let configuration = self.parse_configuration(false)?;

        self.expect_statement_separator(Some("@use rule"))?;

        let span = self.toks_mut().span_from(start);

        if !self.flags().is_use_allowed() {
            return Err((
                "@use rules must be written before any other rules.",
                self.toks_mut().span_from(start),
            )
                .into());
        }

        self.expect_statement_separator(Some("@use rule"))?;

        Ok(AstStmt::Use(AstUseRule {
            url: path,
            namespace,
            configuration: configuration.unwrap_or_default(),
            span,
        }))
    }

    fn parse_at_rule(
        &mut self,
        child: fn(&mut Self) -> SassResult<AstStmt>,
    ) -> SassResult<AstStmt> {
        let start = self.toks().cursor();

        self.expect_char('@')?;
        let name = self.parse_interpolated_identifier()?;
        self.whitespace()?;

        // We want to set [_isUseAllowed] to `false` *unless* we're parsing
        // `@charset`, `@forward`, or `@use`. To avoid double-comparing the rule
        // name, we always set it to `false` and then set it back to its previous
        // value if we're parsing an allowed rule.
        let was_use_allowed = self.flags().is_use_allowed();
        self.flags_mut().set(ContextFlags::IS_USE_ALLOWED, false);

        match name.as_plain() {
            Some("at-root") => self.parse_at_root_rule(start),
            Some("content") => self.parse_content_rule(start),
            Some("debug") => self.parse_debug_rule(),
            Some("each") => self.parse_each_rule(child),
            Some("else") | Some("return") => self.parse_disallowed_at_rule(start),
            Some("error") => self.parse_error_rule(),
            Some("extend") => self.parse_extend_rule(start),
            Some("for") => self.parse_for_rule(child),
            Some("forward") => {
                self.flags_mut()
                    .set(ContextFlags::IS_USE_ALLOWED, was_use_allowed);
                // if (!root) {
                //     _disallowedAtRule();
                // }
                self.parse_forward_rule(start)
            }
            Some("function") => self.parse_function_rule(start),
            Some("if") => self.parse_if_rule(child),
            Some("import") => self.parse_import_rule(start),
            Some("include") => self.parse_include_rule(),
            Some("media") => self.parse_media_rule(start),
            Some("mixin") => self.parse_mixin_rule(start),
            // todo: support -moz-document
            // Some("-moz-document") => self.parse_moz_document_rule(name),
            Some("supports") => self.parse_supports_rule(),
            Some("use") => {
                self.flags_mut()
                    .set(ContextFlags::IS_USE_ALLOWED, was_use_allowed);
                // if (!root) {
                //     _disallowedAtRule();
                // }
                self.parse_use_rule(start)
            }
            Some("warn") => self.parse_warn_rule(),
            Some("while") => self.parse_while_rule(child),
            Some(..) | None => self.unknown_at_rule(name, start),
        }
    }

    fn parse_statement(&mut self) -> SassResult<AstStmt> {
        match self.toks().peek() {
            Some(Token { kind: '@', .. }) => self.parse_at_rule(Self::parse_statement),
            Some(Token { kind: '+', .. }) => {
                if !self.is_indented() {
                    return self.parse_style_rule(None, None);
                }

                let start = self.toks().cursor();

                self.toks_mut().next();

                if !self.looking_at_identifier() {
                    self.toks_mut().set_cursor(start);
                    return self.parse_style_rule(None, None);
                }

                self.flags_mut().set(ContextFlags::IS_USE_ALLOWED, false);
                self.parse_include_rule()
            }
            Some(Token { kind: '=', .. }) => {
                if !self.is_indented() {
                    return self.parse_style_rule(None, None);
                }

                self.flags_mut().set(ContextFlags::IS_USE_ALLOWED, false);
                let start = self.toks().cursor();
                self.toks_mut().next();
                self.whitespace()?;
                self.parse_mixin_rule(start)
            }
            Some(Token { kind: '}', .. }) => {
                Err(("unmatched \"}\".", self.toks().current_span()).into())
            }
            _ => {
                if self.flags().in_style_rule()
                    || self.flags().in_unknown_at_rule()
                    || self.flags().in_mixin()
                    || self.flags().in_content_block()
                {
                    self.parse_declaration_or_style_rule()
                } else {
                    self.parse_variable_declaration_or_style_rule()
                }
            }
        }
    }

    fn parse_declaration_or_style_rule(&mut self) -> SassResult<AstStmt> {
        let start = self.toks().cursor();

        if self.is_plain_css() && self.flags().in_style_rule() && !self.flags().in_unknown_at_rule()
        {
            return self.parse_property_or_variable_declaration(true);
        }

        // The indented syntax allows a single backslash to distinguish a style rule
        // from old-style property syntax. We don't support old property syntax, but
        // we do support the backslash because it's easy to do.
        if self.is_indented() && self.scan_char('\\') {
            return self.parse_style_rule(None, None);
        };

        match self.parse_declaration_or_buffer()? {
            DeclarationOrBuffer::Stmt(s) => Ok(s),
            DeclarationOrBuffer::Buffer(existing_buffer) => {
                self.parse_style_rule(Some(existing_buffer), Some(start))
            }
        }
    }

    fn parse_property_or_variable_declaration(
        &mut self,
        // default=true
        parse_custom_properties: bool,
    ) -> SassResult<AstStmt> {
        let start = self.toks().cursor();

        let name = if matches!(
            self.toks().peek(),
            Some(Token {
                kind: ':' | '*' | '.',
                ..
            })
        ) || (matches!(self.toks().peek(), Some(Token { kind: '#', .. }))
            && !matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. })))
        {
            // Allow the "*prop: val", ":prop: val", "#prop: val", and ".prop: val"
            // hacks.
            let mut name_buffer = Interpolation::new();
            name_buffer.add_char(self.toks_mut().next().unwrap().kind);
            name_buffer.add_string(self.raw_text(Self::whitespace));
            name_buffer.add_interpolation(self.parse_interpolated_identifier()?);
            name_buffer
        } else if !self.is_plain_css() {
            match self.parse_variable_declaration_or_interpolation()? {
                VariableDeclOrInterpolation::Interpolation(interpolation) => interpolation,
                VariableDeclOrInterpolation::VariableDecl(decl) => {
                    return Ok(AstStmt::VariableDecl(decl))
                }
            }
        } else {
            self.parse_interpolated_identifier()?
        };

        self.whitespace()?;
        self.expect_char(':')?;

        if parse_custom_properties && name.initial_plain().starts_with("--") {
            let interpolation = self.parse_interpolated_declaration_value(false, false, true)?;
            let value_span = self.toks_mut().span_from(start);
            let value = AstExpr::String(StringExpr(interpolation, QuoteKind::None), value_span)
                .span(value_span);
            self.expect_statement_separator(Some("custom property"))?;
            return Ok(AstStmt::Style(AstStyle {
                name,
                value: Some(value),
                body: Vec::new(),
                span: value_span,
            }));
        }

        self.whitespace()?;

        if self.looking_at_children()? {
            if self.is_plain_css() {
                return Err((
                    "Nested declarations aren't allowed in plain CSS.",
                    self.toks().current_span(),
                )
                    .into());
            }

            if name.initial_plain().starts_with("--") {
                return Err((
                    "Declarations whose names begin with \"--\" may not be nested",
                    self.toks_mut().span_from(start),
                )
                    .into());
            }

            let children = self.with_children(Self::parse_declaration_child)?.node;

            return Ok(AstStmt::Style(AstStyle {
                name,
                value: None,
                body: children,
                span: self.toks_mut().span_from(start),
            }));
        }

        let value = self.parse_expression(None, None, None)?;
        if self.looking_at_children()? {
            if self.is_plain_css() {
                return Err((
                    "Nested declarations aren't allowed in plain CSS.",
                    self.toks().current_span(),
                )
                    .into());
            }

            if name.initial_plain().starts_with("--") && !matches!(value.node, AstExpr::String(..))
            {
                return Err((
                    "Declarations whose names begin with \"--\" may not be nested",
                    self.toks_mut().span_from(start),
                )
                    .into());
            }

            let children = self.with_children(Self::parse_declaration_child)?.node;

            Ok(AstStmt::Style(AstStyle {
                name,
                value: Some(value),
                body: children,
                span: self.toks_mut().span_from(start),
            }))
        } else {
            self.expect_statement_separator(None)?;
            Ok(AstStmt::Style(AstStyle {
                name,
                value: Some(value),
                body: Vec::new(),
                span: self.toks_mut().span_from(start),
            }))
        }
    }

    fn parse_single_interpolation(&mut self) -> SassResult<Interpolation> {
        self.expect_char('#')?;
        self.expect_char('{')?;
        self.whitespace()?;
        let contents = self.parse_expression(None, None, None)?;
        self.expect_char('}')?;

        if self.is_plain_css() {
            return Err(("Interpolation isn't allowed in plain CSS.", contents.span).into());
        }

        let mut interpolation = Interpolation::new();
        interpolation
            .contents
            .push(InterpolationPart::Expr(contents));

        Ok(interpolation)
    }

    fn parse_interpolated_identifier_body(&mut self, buffer: &mut Interpolation) -> SassResult<()> {
        while let Some(next) = self.toks().peek() {
            match next.kind {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '\u{80}'..=std::char::MAX => {
                    buffer.add_char(next.kind);
                    self.toks_mut().next();
                }
                '\\' => {
                    buffer.add_string(self.parse_escape(false)?);
                }
                '#' if matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. })) => {
                    buffer.add_interpolation(self.parse_single_interpolation()?);
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn parse_interpolated_identifier(&mut self) -> SassResult<Interpolation> {
        let mut buffer = Interpolation::new();

        if self.scan_char('-') {
            buffer.add_char('-');

            if self.scan_char('-') {
                buffer.add_char('-');
                self.parse_interpolated_identifier_body(&mut buffer)?;
                return Ok(buffer);
            }
        }

        match self.toks().peek() {
            Some(tok) if is_name_start(tok.kind) => {
                buffer.add_char(tok.kind);
                self.toks_mut().next();
            }
            Some(Token { kind: '\\', .. }) => {
                buffer.add_string(self.parse_escape(true)?);
            }
            Some(Token { kind: '#', .. })
                if matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. })) =>
            {
                buffer.add_interpolation(self.parse_single_interpolation()?);
            }
            Some(..) | None => {
                return Err(("Expected identifier.", self.toks().current_span()).into())
            }
        }

        self.parse_interpolated_identifier_body(&mut buffer)?;

        Ok(buffer)
    }

    fn looking_at_interpolated_identifier(&mut self) -> bool {
        let first = match self.toks().peek() {
            Some(Token { kind: '\\', .. }) => return true,
            Some(Token { kind: '#', .. }) => {
                return matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. }))
            }
            Some(Token { kind, .. }) if is_name_start(kind) => return true,
            Some(tok) => tok,
            None => return false,
        };

        if first.kind != '-' {
            return false;
        }

        match self.toks().peek_n(1) {
            Some(Token { kind: '#', .. }) => {
                matches!(self.toks().peek_n(2), Some(Token { kind: '{', .. }))
            }
            Some(Token {
                kind: '\\' | '-', ..
            }) => true,
            Some(Token { kind, .. }) => is_name_start(kind),
            None => false,
        }
    }

    fn parse_loud_comment(&mut self) -> SassResult<AstLoudComment> {
        let start = self.toks().cursor();
        self.expect_char('/')?;
        self.expect_char('*')?;

        let mut buffer = Interpolation::new_plain("/*".to_owned());

        while let Some(tok) = self.toks().peek() {
            match tok.kind {
                '#' => {
                    if matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. })) {
                        buffer.add_interpolation(self.parse_single_interpolation()?);
                    } else {
                        self.toks_mut().next();
                        buffer.add_char(tok.kind);
                    }
                }
                '*' => {
                    self.toks_mut().next();
                    buffer.add_char(tok.kind);

                    if self.scan_char('/') {
                        buffer.add_char('/');

                        return Ok(AstLoudComment {
                            text: buffer,
                            span: self.toks_mut().span_from(start),
                        });
                    }
                }
                '\r' => {
                    self.toks_mut().next();
                    // todo: does \r even exist at this point? (removed by lexer)
                    if !self.toks_mut().next_char_is('\n') {
                        buffer.add_char('\n');
                    }
                }
                _ => {
                    buffer.add_char(tok.kind);
                    self.toks_mut().next();
                }
            }
        }

        Err(("expected more input.", self.toks().current_span()).into())
    }

    fn parse_interpolated_declaration_value(
        &mut self,
        // default=false
        allow_semicolon: bool,
        // default=false
        allow_empty: bool,
        // default=true
        allow_colon: bool,
    ) -> SassResult<Interpolation> {
        let mut buffer = Interpolation::new();

        let mut brackets = Vec::new();
        let mut wrote_newline = false;

        while let Some(tok) = self.toks().peek() {
            match tok.kind {
                '\\' => {
                    buffer.add_string(self.parse_escape(true)?);
                    wrote_newline = false;
                }
                '"' | '\'' => {
                    buffer.add_interpolation(
                        self.parse_interpolated_string()?
                            .node
                            .as_interpolation(false),
                    );
                    wrote_newline = false;
                }
                '/' => {
                    if matches!(self.toks().peek_n(1), Some(Token { kind: '*', .. })) {
                        let comment = self.fallible_raw_text(Self::skip_loud_comment)?;
                        buffer.add_string(comment);
                    } else {
                        self.toks_mut().next();
                        buffer.add_char(tok.kind);
                    }

                    wrote_newline = false;
                }
                '#' => {
                    if matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. })) {
                        // Add a full interpolated identifier to handle cases like
                        // "#{...}--1", since "--1" isn't a valid identifier on its own.
                        buffer.add_interpolation(self.parse_interpolated_identifier()?);
                    } else {
                        self.toks_mut().next();
                        buffer.add_char(tok.kind);
                    }

                    wrote_newline = false;
                }
                ' ' | '\t' => {
                    if wrote_newline
                        || !matches!(
                            self.toks().peek_n(1),
                            Some(Token {
                                kind: ' ' | '\r' | '\t' | '\n',
                                ..
                            })
                        )
                    {
                        self.toks_mut().next();
                        buffer.add_char(tok.kind);
                    } else {
                        self.toks_mut().next();
                    }
                }
                '\n' | '\r' => {
                    if self.is_indented() {
                        break;
                    }
                    if !matches!(
                        self.toks().peek_n_backwards(1),
                        Some(Token {
                            kind: '\r' | '\n',
                            ..
                        })
                    ) {
                        buffer.add_char('\n');
                    }
                    self.toks_mut().next();
                    wrote_newline = true;
                }
                '(' | '{' | '[' => {
                    self.toks_mut().next();
                    buffer.add_char(tok.kind);
                    brackets.push(opposite_bracket(tok.kind));
                    wrote_newline = false;
                }
                ')' | '}' | ']' => {
                    if brackets.is_empty() {
                        break;
                    }
                    buffer.add_char(tok.kind);
                    self.expect_char(brackets.pop().unwrap())?;
                    wrote_newline = false;
                }
                ';' => {
                    if !allow_semicolon && brackets.is_empty() {
                        break;
                    }
                    buffer.add_char(tok.kind);
                    self.toks_mut().next();
                    wrote_newline = false;
                }
                ':' => {
                    if !allow_colon && brackets.is_empty() {
                        break;
                    }
                    buffer.add_char(tok.kind);
                    self.toks_mut().next();
                    wrote_newline = false;
                }
                'u' | 'U' => {
                    let before_url = self.toks().cursor();

                    if !self.scan_identifier("url", false)? {
                        buffer.add_char(tok.kind);
                        self.toks_mut().next();
                        wrote_newline = false;
                        continue;
                    }

                    match self.try_url_contents(None)? {
                        Some(contents) => {
                            buffer.add_interpolation(contents);
                        }
                        None => {
                            self.toks_mut().set_cursor(before_url);
                            buffer.add_char(tok.kind);
                            self.toks_mut().next();
                        }
                    }

                    wrote_newline = false;
                }
                _ => {
                    if self.looking_at_identifier() {
                        buffer.add_string(self.parse_identifier(false, false)?);
                    } else {
                        buffer.add_char(tok.kind);
                        self.toks_mut().next();
                    }
                    wrote_newline = false;
                }
            }
        }

        if let Some(&last) = brackets.last() {
            self.expect_char(last)?;
        }

        if !allow_empty && buffer.contents.is_empty() {
            return Err(("Expected token.", self.toks().current_span()).into());
        }

        Ok(buffer)
    }

    fn parse_expression_until_comma(
        &mut self,
        // default=false
        single_equals: bool,
    ) -> SassResult<Spanned<AstExpr>> {
        ValueParser::parse_expression(
            self,
            Some(&|parser| {
                Ok(matches!(
                    parser.toks().peek(),
                    Some(Token { kind: ',', .. })
                ))
            }),
            false,
            single_equals,
        )
    }

    fn parse_argument_invocation(
        &mut self,
        for_mixin: bool,
        allow_empty_second_arg: bool,
    ) -> SassResult<ArgumentInvocation> {
        let start = self.toks().cursor();

        self.expect_char('(')?;
        self.whitespace()?;

        let mut positional = Vec::new();
        let mut named = BTreeMap::new();

        let mut rest: Option<AstExpr> = None;
        let mut keyword_rest: Option<AstExpr> = None;

        while self.looking_at_expression() {
            let expression = self.parse_expression_until_comma(!for_mixin)?;
            self.whitespace()?;

            if expression.node.is_variable() && self.scan_char(':') {
                let name = match expression.node {
                    AstExpr::Variable { name, .. } => name,
                    _ => unreachable!(),
                };

                self.whitespace()?;
                if named.contains_key(&name.node) {
                    return Err(("Duplicate argument.", name.span).into());
                }

                named.insert(
                    name.node,
                    self.parse_expression_until_comma(!for_mixin)?.node,
                );
            } else if self.scan_char('.') {
                self.expect_char('.')?;
                self.expect_char('.')?;

                if rest.is_none() {
                    rest = Some(expression.node);
                } else {
                    keyword_rest = Some(expression.node);
                    self.whitespace()?;
                    break;
                }
            } else if !named.is_empty() {
                return Err((
                    "Positional arguments must come before keyword arguments.",
                    expression.span,
                )
                    .into());
            } else {
                positional.push(expression.node);
            }

            self.whitespace()?;
            if !self.scan_char(',') {
                break;
            }
            self.whitespace()?;

            if allow_empty_second_arg
                && positional.len() == 1
                && named.is_empty()
                && rest.is_none()
                && matches!(self.toks().peek(), Some(Token { kind: ')', .. }))
            {
                positional.push(AstExpr::String(
                    StringExpr(Interpolation::new(), QuoteKind::None),
                    self.toks().current_span(),
                ));
                break;
            }
        }

        self.expect_char(')')?;

        Ok(ArgumentInvocation {
            positional,
            named,
            rest,
            keyword_rest,
            span: self.toks_mut().span_from(start),
        })
    }

    fn parse_expression(
        &mut self,
        parse_until: Option<Predicate<'_, Self>>,
        inside_bracketed_list: Option<bool>,
        single_equals: Option<bool>,
    ) -> SassResult<Spanned<AstExpr>> {
        ValueParser::parse_expression(
            self,
            parse_until,
            inside_bracketed_list.unwrap_or(false),
            single_equals.unwrap_or(false),
        )
    }

    fn parse_declaration_or_buffer(&mut self) -> SassResult<DeclarationOrBuffer> {
        let start = self.toks().cursor();
        let mut name_buffer = Interpolation::new();

        // Allow the "*prop: val", ":prop: val", "#prop: val", and ".prop: val"
        // hacks.
        let first = self.toks().peek();
        let mut starts_with_punctuation = false;

        if matches!(
            first,
            Some(Token {
                kind: ':' | '*' | '.',
                ..
            })
        ) || (matches!(first, Some(Token { kind: '#', .. }))
            && !matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. })))
        {
            starts_with_punctuation = true;
            name_buffer.add_char(self.toks_mut().next().unwrap().kind);
            name_buffer.add_string(self.raw_text(Self::whitespace));
        }

        if !self.looking_at_interpolated_identifier() {
            return Ok(DeclarationOrBuffer::Buffer(name_buffer));
        }

        let variable_or_interpolation = if starts_with_punctuation {
            VariableDeclOrInterpolation::Interpolation(self.parse_interpolated_identifier()?)
        } else {
            self.parse_variable_declaration_or_interpolation()?
        };

        match variable_or_interpolation {
            VariableDeclOrInterpolation::Interpolation(int) => name_buffer.add_interpolation(int),
            VariableDeclOrInterpolation::VariableDecl(v) => {
                return Ok(DeclarationOrBuffer::Stmt(AstStmt::VariableDecl(v)))
            }
        }

        self.flags_mut().set(ContextFlags::IS_USE_ALLOWED, false);

        if self.next_matches("/*") {
            name_buffer.add_string(self.fallible_raw_text(Self::skip_loud_comment)?);
        }

        let mut mid_buffer = String::new();
        mid_buffer.push_str(&self.raw_text(Self::whitespace));

        if !self.scan_char(':') {
            if !mid_buffer.is_empty() {
                name_buffer.add_char(' ');
            }
            return Ok(DeclarationOrBuffer::Buffer(name_buffer));
        }
        mid_buffer.push(':');

        // Parse custom properties as declarations no matter what.
        if name_buffer.initial_plain().starts_with("--") {
            let value_start = self.toks().cursor();
            let value = self.parse_interpolated_declaration_value(false, false, true)?;
            let value_span = self.toks_mut().span_from(value_start);
            self.expect_statement_separator(Some("custom property"))?;
            return Ok(DeclarationOrBuffer::Stmt(AstStmt::Style(AstStyle {
                name: name_buffer,
                value: Some(
                    AstExpr::String(StringExpr(value, QuoteKind::None), value_span)
                        .span(value_span),
                ),
                span: self.toks_mut().span_from(start),
                body: Vec::new(),
            })));
        }

        if self.scan_char(':') {
            name_buffer.add_string(mid_buffer);
            name_buffer.add_char(':');
            return Ok(DeclarationOrBuffer::Buffer(name_buffer));
        } else if self.is_indented() && self.looking_at_interpolated_identifier() {
            // In the indented syntax, `foo:bar` is always considered a selector
            // rather than a property.
            name_buffer.add_string(mid_buffer);
            return Ok(DeclarationOrBuffer::Buffer(name_buffer));
        }

        let post_colon_whitespace = self.raw_text(Self::whitespace);
        if self.looking_at_children()? {
            let body = self.with_children(Self::parse_declaration_child)?.node;
            return Ok(DeclarationOrBuffer::Stmt(AstStmt::Style(AstStyle {
                name: name_buffer,
                value: None,
                span: self.toks_mut().span_from(start),
                body,
            })));
        }

        mid_buffer.push_str(&post_colon_whitespace);
        let could_be_selector =
            post_colon_whitespace.is_empty() && self.looking_at_interpolated_identifier();

        let before_decl = self.toks().cursor();

        let mut calculate_value = || {
            let value = self.parse_expression(None, None, None)?;

            if self.looking_at_children()? {
                if could_be_selector {
                    self.expect_statement_separator(None)?;
                }
            } else if !self.at_end_of_statement() {
                self.expect_statement_separator(None)?;
            }

            Ok(value)
        };

        let value = match calculate_value() {
            Ok(v) => v,
            Err(e) => {
                if !could_be_selector {
                    return Err(e);
                }

                self.toks_mut().set_cursor(before_decl);
                let additional = self.almost_any_value(false)?;
                if !self.is_indented() && self.toks_mut().next_char_is(';') {
                    return Err(e);
                }

                name_buffer.add_string(mid_buffer);
                name_buffer.add_interpolation(additional);
                return Ok(DeclarationOrBuffer::Buffer(name_buffer));
            }
        };

        if self.looking_at_children()? {
            let body = self.with_children(Self::parse_declaration_child)?.node;
            Ok(DeclarationOrBuffer::Stmt(AstStmt::Style(AstStyle {
                name: name_buffer,
                value: Some(value),
                span: self.toks_mut().span_from(start),
                body,
            })))
        } else {
            self.expect_statement_separator(None)?;
            Ok(DeclarationOrBuffer::Stmt(AstStmt::Style(AstStyle {
                name: name_buffer,
                value: Some(value),
                span: self.toks_mut().span_from(start),
                body: Vec::new(),
            })))
        }
    }

    fn parse_declaration_child(&mut self) -> SassResult<AstStmt> {
        let start = self.toks().cursor();

        if self.toks_mut().next_char_is('@') {
            self.parse_declaration_at_rule(start)
        } else {
            self.parse_property_or_variable_declaration(false)
        }
    }

    fn parse_plain_at_rule_name(&mut self) -> SassResult<String> {
        self.expect_char('@')?;
        let name = self.parse_identifier(false, false)?;
        self.whitespace()?;
        Ok(name)
    }

    fn parse_declaration_at_rule(&mut self, start: usize) -> SassResult<AstStmt> {
        let name = self.parse_plain_at_rule_name()?;

        match name.as_str() {
            "content" => self.parse_content_rule(start),
            "debug" => self.parse_debug_rule(),
            "each" => self.parse_each_rule(Self::parse_declaration_child),
            "else" => self.parse_disallowed_at_rule(start),
            "error" => self.parse_error_rule(),
            "for" => self.parse_for_rule(Self::parse_declaration_child),
            "if" => self.parse_if_rule(Self::parse_declaration_child),
            "include" => self.parse_include_rule(),
            "warn" => self.parse_warn_rule(),
            "while" => self.parse_while_rule(Self::parse_declaration_child),
            _ => self.parse_disallowed_at_rule(start),
        }
    }

    fn parse_variable_declaration_or_style_rule(&mut self) -> SassResult<AstStmt> {
        let start = self.toks().cursor();

        if self.is_plain_css() {
            return self.parse_style_rule(None, None);
        }

        // The indented syntax allows a single backslash to distinguish a style rule
        // from old-style property syntax. We don't support old property syntax, but
        // we do support the backslash because it's easy to do.
        if self.is_indented() && self.scan_char('\\') {
            return self.parse_style_rule(None, None);
        };

        if !self.looking_at_identifier() {
            return self.parse_style_rule(None, None);
        }

        match self.parse_variable_declaration_or_interpolation()? {
            VariableDeclOrInterpolation::VariableDecl(var) => Ok(AstStmt::VariableDecl(var)),
            VariableDeclOrInterpolation::Interpolation(int) => {
                self.parse_style_rule(Some(int), Some(start))
            }
        }
    }

    fn parse_style_rule(
        &mut self,
        existing_buffer: Option<Interpolation>,
        start: Option<usize>,
    ) -> SassResult<AstStmt> {
        let start = start.unwrap_or_else(|| self.toks().cursor());

        self.flags_mut().set(ContextFlags::IS_USE_ALLOWED, false);
        let mut interpolation = self.parse_style_rule_selector()?;

        if let Some(mut existing_buffer) = existing_buffer {
            existing_buffer.add_interpolation(interpolation);
            interpolation = existing_buffer;
        }

        if interpolation.contents.is_empty() {
            return Err(("expected \"}\".", self.toks().current_span()).into());
        }

        let was_in_style_rule = self.flags().in_style_rule();
        *self.flags_mut() |= ContextFlags::IN_STYLE_RULE;

        let selector_span = self.toks_mut().span_from(start);

        let children = self.with_children(Self::parse_statement)?;

        self.flags_mut()
            .set(ContextFlags::IN_STYLE_RULE, was_in_style_rule);

        let span = selector_span.merge(children.span);

        Ok(AstStmt::RuleSet(AstRuleSet {
            selector: interpolation,
            body: children.node,
            selector_span,
            span,
        }))
    }

    fn parse_silent_comment(&mut self) -> SassResult<AstStmt> {
        let start = self.toks().cursor();
        debug_assert!(self.next_matches("//"));
        self.toks_mut().next();
        self.toks_mut().next();

        let mut buffer = String::new();

        while let Some(tok) = self.toks_mut().next() {
            if tok.kind == '\n' {
                self.whitespace_without_comments();
                if self.next_matches("//") {
                    self.toks_mut().next();
                    self.toks_mut().next();
                    buffer.clear();
                    continue;
                }
                break;
            }

            buffer.push(tok.kind);
        }

        if self.is_plain_css() {
            return Err((
                "Silent comments aren't allowed in plain CSS.",
                self.toks_mut().span_from(start),
            )
                .into());
        }

        self.whitespace_without_comments();

        Ok(AstStmt::SilentComment(AstSilentComment {
            text: buffer,
            span: self.toks_mut().span_from(start),
        }))
    }

    fn next_is_hex(&self) -> bool {
        match self.toks().peek() {
            Some(Token { kind, .. }) => kind.is_ascii_hexdigit(),
            None => false,
        }
    }

    fn assert_public(ident: &str, span: Span) -> SassResult<()> {
        if !ScssParser::is_private(ident) {
            return Ok(());
        }

        Err((
            "Private members can't be accessed from outside their modules.",
            span,
        )
            .into())
    }

    fn is_private(ident: &str) -> bool {
        ident.starts_with('-') || ident.starts_with('_')
    }

    fn parse_variable_declaration_without_namespace(
        &mut self,
        namespace: Option<Spanned<Identifier>>,
        start: Option<usize>,
    ) -> SassResult<AstVariableDecl> {
        let start = start.unwrap_or_else(|| self.toks().cursor());

        let name = self.parse_variable_name()?;

        if namespace.is_some() {
            Self::assert_public(&name, self.toks_mut().span_from(start))?;
        }

        if self.is_plain_css() {
            return Err((
                "Sass variables aren't allowed in plain CSS.",
                self.toks_mut().span_from(start),
            )
                .into());
        }

        self.whitespace()?;
        self.expect_char(':')?;
        self.whitespace()?;

        let value = self.parse_expression(None, None, None)?.node;

        let mut is_guarded = false;
        let mut is_global = false;

        while self.scan_char('!') {
            let flag_start = self.toks().cursor();
            let flag = self.parse_identifier(false, false)?;

            match flag.as_str() {
                "default" => is_guarded = true,
                "global" => {
                    if namespace.is_some() {
                        return Err((
                            "!global isn't allowed for variables in other modules.",
                            self.toks_mut().span_from(flag_start),
                        )
                            .into());
                    }

                    is_global = true;
                }
                _ => {
                    return Err(
                        ("Invalid flag name.", self.toks_mut().span_from(flag_start)).into(),
                    )
                }
            }

            self.whitespace()?;
        }

        self.expect_statement_separator(Some("variable declaration"))?;

        let declaration = AstVariableDecl {
            namespace,
            name: Identifier::from(name),
            value,
            is_guarded,
            is_global,
            span: self.toks_mut().span_from(start),
        };

        if is_global {
            // todo
            // _globalVariables.putIfAbsent(name, () => declaration)
        }

        Ok(declaration)
    }

    fn almost_any_value(
        &mut self,
        // default=false
        omit_comments: bool,
    ) -> SassResult<Interpolation> {
        let mut buffer = Interpolation::new();

        while let Some(tok) = self.toks().peek() {
            match tok.kind {
                '\\' => {
                    // Write a literal backslash because this text will be re-parsed.
                    buffer.add_char(tok.kind);
                    self.toks_mut().next();
                    match self.toks_mut().next() {
                        Some(tok) => buffer.add_char(tok.kind),
                        None => {
                            return Err(("expected more input.", self.toks().current_span()).into())
                        }
                    }
                }
                '"' | '\'' => {
                    buffer.add_interpolation(
                        self.parse_interpolated_string()?
                            .node
                            .as_interpolation(false),
                    );
                }
                '/' => {
                    let comment_start = self.toks().cursor();
                    if self.scan_comment()? {
                        if !omit_comments {
                            buffer.add_string(self.toks().raw_text(comment_start));
                        }
                    } else {
                        buffer.add_char(self.toks_mut().next().unwrap().kind);
                    }
                }
                '#' => {
                    if matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. })) {
                        // Add a full interpolated identifier to handle cases like
                        // "#{...}--1", since "--1" isn't a valid identifier on its own.
                        buffer.add_interpolation(self.parse_interpolated_identifier()?);
                    } else {
                        self.toks_mut().next();
                        buffer.add_char(tok.kind);
                    }
                }
                '\r' | '\n' => {
                    if self.is_indented() {
                        break;
                    }
                    buffer.add_char(self.toks_mut().next().unwrap().kind);
                }
                '!' | ';' | '{' | '}' => break,
                'u' | 'U' => {
                    let before_url = self.toks().cursor();
                    if !self.scan_identifier("url", false)? {
                        self.toks_mut().next();
                        buffer.add_char(tok.kind);
                        continue;
                    }

                    match self.try_url_contents(None)? {
                        Some(contents) => buffer.add_interpolation(contents),
                        None => {
                            self.toks_mut().set_cursor(before_url);
                            self.toks_mut().next();
                            buffer.add_char(tok.kind);
                        }
                    }
                }
                _ => {
                    if self.looking_at_identifier() {
                        buffer.add_string(self.parse_identifier(false, false)?);
                    } else {
                        buffer.add_char(self.toks_mut().next().unwrap().kind);
                    }
                }
            }
        }

        Ok(buffer)
    }

    fn parse_variable_declaration_or_interpolation(
        &mut self,
    ) -> SassResult<VariableDeclOrInterpolation> {
        if !self.looking_at_identifier() {
            return Ok(VariableDeclOrInterpolation::Interpolation(
                self.parse_interpolated_identifier()?,
            ));
        }

        let start = self.toks().cursor();

        let ident = self.parse_identifier(false, false)?;
        if self.next_matches(".$") {
            let namespace_span = self.toks_mut().span_from(start);
            self.expect_char('.')?;
            Ok(VariableDeclOrInterpolation::VariableDecl(
                self.parse_variable_declaration_without_namespace(
                    Some(Spanned {
                        node: Identifier::from(ident),
                        span: namespace_span,
                    }),
                    Some(start),
                )?,
            ))
        } else {
            let mut buffer = Interpolation::new_plain(ident);

            if self.looking_at_interpolated_identifier_body() {
                buffer.add_interpolation(self.parse_interpolated_identifier()?);
            }

            Ok(VariableDeclOrInterpolation::Interpolation(buffer))
        }
    }

    fn looking_at_interpolated_identifier_body(&mut self) -> bool {
        match self.toks().peek() {
            Some(Token { kind: '\\', .. }) => true,
            Some(Token { kind: '#', .. })
                if matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. })) =>
            {
                true
            }
            Some(Token { kind, .. }) if is_name(kind) => true,
            Some(..) | None => false,
        }
    }

    fn expression_until_comparison(&mut self) -> SassResult<Spanned<AstExpr>> {
        let value = self.parse_expression(
            Some(&|parser| {
                Ok(match parser.toks().peek() {
                    Some(Token { kind: '>', .. }) | Some(Token { kind: '<', .. }) => true,
                    Some(Token { kind: '=', .. }) => {
                        !matches!(parser.toks().peek_n(1), Some(Token { kind: '=', .. }))
                    }
                    _ => false,
                })
            }),
            None,
            None,
        )?;
        Ok(value)
    }

    fn parse_media_query_list(&mut self) -> SassResult<Interpolation> {
        let mut buf = Interpolation::new();
        loop {
            self.whitespace()?;
            self.parse_media_query(&mut buf)?;
            self.whitespace()?;
            if !self.scan_char(',') {
                break;
            }
            buf.add_char(',');
            buf.add_char(' ');
        }
        Ok(buf)
    }

    fn parse_media_in_parens(&mut self, buf: &mut Interpolation) -> SassResult<()> {
        self.expect_char_with_message('(', "media condition in parentheses")?;
        buf.add_char('(');
        self.whitespace()?;

        if matches!(self.toks().peek(), Some(Token { kind: '(', .. })) {
            self.parse_media_in_parens(buf)?;
            self.whitespace()?;

            if self.scan_identifier("and", false)? {
                buf.add_string(" and ".to_owned());
                self.expect_whitespace()?;
                self.parse_media_logic_sequence(buf, "and")?;
            } else if self.scan_identifier("or", false)? {
                buf.add_string(" or ".to_owned());
                self.expect_whitespace()?;
                self.parse_media_logic_sequence(buf, "or")?;
            }
        } else if self.scan_identifier("not", false)? {
            buf.add_string("not ".to_owned());
            self.expect_whitespace()?;
            self.parse_media_or_interpolation(buf)?;
        } else {
            buf.add_expr(self.expression_until_comparison()?);

            if self.scan_char(':') {
                self.whitespace()?;
                buf.add_char(':');
                buf.add_char(' ');
                buf.add_expr(self.parse_expression(None, None, None)?);
            } else {
                let next = self.toks().peek();
                if matches!(
                    next,
                    Some(Token {
                        kind: '<' | '>' | '=',
                        ..
                    })
                ) {
                    let next = next.unwrap().kind;
                    buf.add_char(' ');
                    buf.add_char(self.toks_mut().next().unwrap().kind);

                    if (next == '<' || next == '>') && self.scan_char('=') {
                        buf.add_char('=');
                    }

                    buf.add_char(' ');

                    self.whitespace()?;

                    buf.add_expr(self.expression_until_comparison()?);

                    if (next == '<' || next == '>') && self.scan_char(next) {
                        buf.add_char(' ');
                        buf.add_char(next);

                        if self.scan_char('=') {
                            buf.add_char('=');
                        }

                        buf.add_char(' ');

                        self.whitespace()?;
                        buf.add_expr(self.expression_until_comparison()?);
                    }
                }
            }
        }

        self.expect_char(')')?;
        self.whitespace()?;
        buf.add_char(')');

        Ok(())
    }

    fn parse_media_logic_sequence(
        &mut self,
        buf: &mut Interpolation,
        operator: &'static str,
    ) -> SassResult<()> {
        loop {
            self.parse_media_or_interpolation(buf)?;
            self.whitespace()?;

            if !self.scan_identifier(operator, false)? {
                return Ok(());
            }

            self.expect_whitespace()?;

            buf.add_char(' ');
            buf.add_string(operator.to_owned());
            buf.add_char(' ');
        }
    }

    fn parse_media_or_interpolation(&mut self, buf: &mut Interpolation) -> SassResult<()> {
        if self.toks_mut().next_char_is('#') {
            buf.add_interpolation(self.parse_single_interpolation()?);
        } else {
            self.parse_media_in_parens(buf)?;
        }

        Ok(())
    }

    fn parse_media_query(&mut self, buf: &mut Interpolation) -> SassResult<()> {
        if matches!(self.toks().peek(), Some(Token { kind: '(', .. })) {
            self.parse_media_in_parens(buf)?;
            self.whitespace()?;

            if self.scan_identifier("and", false)? {
                buf.add_string(" and ".to_owned());
                self.expect_whitespace()?;
                self.parse_media_logic_sequence(buf, "and")?;
            } else if self.scan_identifier("or", false)? {
                buf.add_string(" or ".to_owned());
                self.expect_whitespace()?;
                self.parse_media_logic_sequence(buf, "or")?;
            }

            return Ok(());
        }

        let ident1 = self.parse_interpolated_identifier()?;

        if ident1.as_plain().unwrap_or("").to_ascii_lowercase() == "not" {
            // For example, "@media not (...) {"
            self.expect_whitespace()?;
            if !self.looking_at_interpolated_identifier() {
                buf.add_string("not ".to_owned());
                self.parse_media_or_interpolation(buf)?;
                return Ok(());
            }
        }

        self.whitespace()?;
        buf.add_interpolation(ident1);
        if !self.looking_at_interpolated_identifier() {
            // For example, "@media screen {".
            return Ok(());
        }

        buf.add_char(' ');

        let ident2 = self.parse_interpolated_identifier()?;

        if ident2.as_plain().unwrap_or("").to_ascii_lowercase() == "and" {
            self.expect_whitespace()?;
            // For example, "@media screen and ..."
            buf.add_string(" and ".to_owned());
        } else {
            self.whitespace()?;
            buf.add_interpolation(ident2);

            if self.scan_identifier("and", false)? {
                // For example, "@media only screen and ..."
                self.expect_whitespace()?;
                buf.add_string(" and ".to_owned());
            } else {
                // For example, "@media only screen {"
                return Ok(());
            }
        }

        // We've consumed either `IDENTIFIER "and"` or
        // `IDENTIFIER IDENTIFIER "and"`.

        if self.scan_identifier("not", false)? {
            // For example, "@media screen and not (...) {"
            self.expect_whitespace()?;
            buf.add_string("not ".to_owned());
            self.parse_media_or_interpolation(buf)?;
            return Ok(());
        }

        self.parse_media_logic_sequence(buf, "and")?;

        Ok(())
    }
}
