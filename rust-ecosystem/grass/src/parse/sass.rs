use std::path::Path;

use codemap::Span;

use crate::{ast::*, error::SassResult, lexer::Lexer, ContextFlags, Options, Token};

use super::{BaseParser, StylesheetParser};

pub(crate) struct SassParser<'a> {
    pub toks: Lexer,
    pub path: &'a Path,
    pub empty_span: Span,
    pub flags: ContextFlags,
    pub options: &'a Options<'a>,
    pub current_indentation: usize,
    pub next_indentation: Option<usize>,
    pub spaces: Option<bool>,
    pub next_indentation_end: Option<usize>,
}

impl<'a> BaseParser for SassParser<'a> {
    fn toks(&self) -> &Lexer {
        &self.toks
    }

    fn toks_mut(&mut self) -> &mut Lexer {
        &mut self.toks
    }

    fn whitespace_without_comments(&mut self) {
        while let Some(next) = self.toks.peek() {
            if next.kind != '\t' && next.kind != ' ' {
                break;
            }

            self.toks.next();
        }
    }

    fn skip_loud_comment(&mut self) -> SassResult<()> {
        self.expect_char('/')?;
        self.expect_char('*')?;

        loop {
            let mut next = self.toks.next();
            match next {
                Some(Token { kind: '\n', .. }) => {
                    return Err(("expected */.", self.toks.prev_span()).into())
                }
                Some(Token { kind: '*', .. }) => {}
                _ => continue,
            }

            loop {
                next = self.toks.next();

                if !matches!(next, Some(Token { kind: '*', .. })) {
                    break;
                }
            }

            if matches!(next, Some(Token { kind: '/', .. })) {
                break;
            }
        }

        Ok(())
    }
}

impl<'a> StylesheetParser<'a> for SassParser<'a> {
    fn is_plain_css(&self) -> bool {
        false
    }

    fn is_indented(&self) -> bool {
        true
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
        self.current_indentation
    }

    fn empty_span(&self) -> Span {
        self.empty_span
    }

    fn parse_style_rule_selector(&mut self) -> SassResult<Interpolation> {
        let mut buffer = Interpolation::new();

        loop {
            buffer.add_interpolation(self.almost_any_value(true)?);
            buffer.add_char('\n');

            if !(buffer.trailing_string().trim_end().ends_with(',') && self.scan_char('\n')) {
                break;
            }
        }

        Ok(buffer)
    }

    fn expect_statement_separator(&mut self, _name: Option<&str>) -> SassResult<()> {
        if !self.at_end_of_statement() {
            self.expect_newline()?;
        }

        if self.peek_indentation()? <= self.current_indentation {
            return Ok(());
        }

        // todo: position: _nextIndentationEnd!.position
        // todo: error message, "Nothing may be indented ${name == null ? 'here' : 'beneath a $name'}."

        Err(("Nothing may be indented here", self.toks.current_span()).into())
    }

    fn at_end_of_statement(&self) -> bool {
        matches!(self.toks.peek(), Some(Token { kind: '\n', .. }) | None)
    }

    fn looking_at_children(&mut self) -> SassResult<bool> {
        Ok(self.at_end_of_statement() && self.peek_indentation()? > self.current_indentation)
    }

    fn scan_else(&mut self, if_indentation: usize) -> SassResult<bool> {
        if self.peek_indentation()? != if_indentation {
            return Ok(false);
        }

        let start = self.toks.cursor();
        let start_indentation = self.current_indentation;
        let start_next_indentation = self.next_indentation;
        let start_next_indentation_end = self.next_indentation_end;

        self.read_indentation()?;
        if self.scan_char('@') && self.scan_identifier("else", false)? {
            return Ok(true);
        }

        self.toks.set_cursor(start);
        self.current_indentation = start_indentation;
        self.next_indentation = start_next_indentation;
        self.next_indentation_end = start_next_indentation_end;
        Ok(false)
    }

    fn parse_children(
        &mut self,
        child: fn(&mut Self) -> SassResult<AstStmt>,
    ) -> SassResult<Vec<AstStmt>> {
        let mut children = Vec::new();
        self.while_indented_lower(|parser| {
            if let Some(parsed_child) = parser.parse_child(|parser| Ok(Some(child(parser)?)))? {
                children.push(parsed_child);
            }

            Ok(())
        })?;

        Ok(children)
    }

    fn parse_statements(
        &mut self,
        statement: fn(&mut Self) -> SassResult<Option<AstStmt>>,
    ) -> SassResult<Vec<AstStmt>> {
        if self.toks.next_char_is(' ') || self.toks.next_char_is('\t') {
            return Err((
                "Indenting at the beginning of the document is illegal.",
                self.toks.current_span(),
            )
                .into());
        }

        let mut statements = Vec::new();

        while self.toks.peek().is_some() {
            if let Some(child) = self.parse_child(statement)? {
                statements.push(child);
            }

            let indentation = self.read_indentation()?;
            assert_eq!(indentation, 0);
        }

        Ok(statements)
    }

    fn parse_silent_comment(&mut self) -> SassResult<AstStmt> {
        let start = self.toks.cursor();
        self.expect_char('/')?;
        self.expect_char('/')?;

        let mut buffer = String::new();

        let parent_indentation = self.current_indentation;

        'outer: loop {
            let comment_prefix = if self.scan_char('/') { "///" } else { "//" };

            loop {
                buffer.push_str(comment_prefix);
                //     buffer.write(commentPrefix);

                // Skip the initial characters because we're already writing the
                // slashes.
                for _ in comment_prefix.len()..(self.current_indentation - parent_indentation) {
                    buffer.push(' ');
                }

                while self.toks.peek().is_some() && !self.toks.next_char_is('\n') {
                    buffer.push(self.toks.next().unwrap().kind);
                }

                buffer.push('\n');

                if self.peek_indentation()? < parent_indentation {
                    break 'outer;
                }

                if self.peek_indentation()? == parent_indentation {
                    // Look ahead to the next line to see if it starts another comment.
                    if matches!(
                        self.toks.peek_n(1 + parent_indentation),
                        Some(Token { kind: '/', .. })
                    ) && matches!(
                        self.toks.peek_n(2 + parent_indentation),
                        Some(Token { kind: '/', .. })
                    ) {
                        self.read_indentation()?;
                    }
                    break;
                }

                self.read_indentation()?;
            }

            if !self.scan("//") {
                break;
            }
        }

        Ok(AstStmt::SilentComment(AstSilentComment {
            text: buffer,
            span: self.toks.span_from(start),
        }))
    }

    fn parse_loud_comment(&mut self) -> SassResult<AstLoudComment> {
        let start = self.toks.cursor();
        self.expect_char('/')?;
        self.expect_char('*')?;

        let mut first = true;

        let mut buffer = Interpolation::new_plain("/*".to_owned());
        let parent_indentation = self.current_indentation;

        loop {
            if first {
                let beginning_of_comment = self.toks.cursor();

                self.spaces();

                if self.toks.next_char_is('\n') {
                    self.read_indentation()?;
                    buffer.add_char(' ');
                } else {
                    buffer.add_string(self.toks.raw_text(beginning_of_comment));
                }
            } else {
                buffer.add_string("\n * ".to_owned());
            }

            first = false;

            for _ in 3..(self.current_indentation - parent_indentation) {
                buffer.add_char(' ');
            }

            while self.toks.peek().is_some() {
                match self.toks.peek() {
                    Some(Token {
                        kind: '\n' | '\r', ..
                    }) => break,
                    Some(Token { kind: '#', .. }) => {
                        if matches!(self.toks.peek_n(1), Some(Token { kind: '{', .. })) {
                            buffer.add_interpolation(self.parse_single_interpolation()?);
                        } else {
                            buffer.add_char('#');
                            self.toks.next();
                        }
                    }
                    Some(Token { kind, .. }) => {
                        buffer.add_char(kind);
                        self.toks.next();
                    }
                    None => todo!(),
                }
            }

            if self.peek_indentation()? <= parent_indentation {
                break;
            }

            // Preserve empty lines.
            while self.looking_at_double_newline() {
                self.expect_newline()?;
                buffer.add_char('\n');
                buffer.add_char(' ');
                buffer.add_char('*');
            }

            self.read_indentation()?;
        }

        if !buffer.trailing_string().trim_end().ends_with("*/") {
            buffer.add_string(" */".to_owned());
        }

        Ok(AstLoudComment {
            text: buffer,
            span: self.toks.span_from(start),
        })
    }
}

impl<'a> SassParser<'a> {
    pub fn new(
        toks: Lexer,
        options: &'a Options<'a>,
        empty_span: Span,
        file_name: &'a Path,
    ) -> Self {
        let mut flags = ContextFlags::empty();

        flags.set(ContextFlags::IS_USE_ALLOWED, true);

        SassParser {
            toks,
            path: file_name,
            empty_span,
            flags,
            options,
            current_indentation: 0,
            next_indentation: None,
            next_indentation_end: None,
            spaces: None,
        }
    }

    fn peek_indentation(&mut self) -> SassResult<usize> {
        if let Some(next) = self.next_indentation {
            return Ok(next);
        }

        if self.toks.peek().is_none() {
            self.next_indentation = Some(0);
            self.next_indentation_end = Some(self.toks.cursor());
            return Ok(0);
        }

        let start = self.toks.cursor();

        if !self.scan_char('\n') {
            return Err(("Expected newline.", self.toks.current_span()).into());
        }

        let mut contains_tab;
        let mut contains_space;
        let mut next_indentation;

        loop {
            contains_tab = false;
            contains_space = false;
            next_indentation = 0;

            while let Some(next) = self.toks.peek() {
                match next.kind {
                    ' ' => contains_space = true,
                    '\t' => contains_tab = true,
                    _ => break,
                }

                next_indentation += 1;
                self.toks.next();
            }

            if self.toks.peek().is_none() {
                self.next_indentation = Some(0);
                self.next_indentation_end = Some(self.toks.cursor());
                self.toks.set_cursor(start);
                return Ok(0);
            }

            if !self.scan_char('\n') {
                break;
            }
        }

        self.check_indentation_consistency(contains_tab, contains_space, start)?;

        self.next_indentation = Some(next_indentation);

        if next_indentation > 0 {
            self.spaces.get_or_insert(contains_space);
        }

        self.next_indentation_end = Some(self.toks.cursor());
        self.toks.set_cursor(start);

        Ok(next_indentation)
    }

    fn check_indentation_consistency(
        &mut self,
        contains_tab: bool,
        contains_space: bool,
        start: usize,
    ) -> SassResult<()> {
        // NOTE: error message spans here start from the beginning of the line
        if contains_tab {
            if contains_space {
                return Err((
                    "Tabs and spaces may not be mixed.",
                    self.toks.span_from(start),
                )
                    .into());
            } else if self.spaces == Some(true) {
                return Err(("Expected spaces, was tabs.", self.toks.span_from(start)).into());
            }
        } else if contains_space && self.spaces == Some(false) {
            return Err(("Expected tabs, was spaces.", self.toks.span_from(start)).into());
        }

        Ok(())
    }

    fn expect_newline(&mut self) -> SassResult<()> {
        match self.toks.peek() {
            Some(Token { kind: ';', .. }) => Err((
                "semicolons aren't allowed in the indented syntax.",
                self.toks.current_span(),
            )
                .into()),
            Some(Token { kind: '\r', .. }) => {
                self.toks.next();
                self.scan_char('\n');
                Ok(())
            }
            Some(Token { kind: '\n', .. }) => {
                self.toks.next();
                Ok(())
            }
            _ => Err(("expected newline.", self.toks.current_span()).into()),
        }
    }

    fn read_indentation(&mut self) -> SassResult<usize> {
        self.current_indentation = match self.next_indentation {
            Some(indent) => indent,
            None => {
                let indent = self.peek_indentation()?;
                self.next_indentation = Some(indent);
                indent
            }
        };

        self.toks.set_cursor(self.next_indentation_end.unwrap());
        self.next_indentation = None;
        self.next_indentation_end = None;

        Ok(self.current_indentation)
    }

    fn while_indented_lower(
        &mut self,
        mut body: impl FnMut(&mut Self) -> SassResult<()>,
    ) -> SassResult<()> {
        let parent_indentation = self.current_indentation;
        let mut child_indentation = None;

        while self.peek_indentation()? > parent_indentation {
            let indentation = self.read_indentation()?;
            let child_indent = *child_indentation.get_or_insert(indentation);

            if child_indent != indentation {
                return Err((
                    format!(
                        "Inconsistent indentation, expected {child_indent} spaces.",
                        child_indent = child_indent
                    ),
                    self.toks.current_span(),
                )
                    .into());
            }

            body(self)?;
        }

        Ok(())
    }

    fn parse_child(
        &mut self,
        child: impl FnOnce(&mut Self) -> SassResult<Option<AstStmt>>,
    ) -> SassResult<Option<AstStmt>> {
        Ok(Some(match self.toks.peek() {
            Some(Token {
                kind: '\n' | '\r', ..
            }) => return Ok(None),
            Some(Token { kind: '$', .. }) => AstStmt::VariableDecl(
                self.parse_variable_declaration_without_namespace(None, None)?,
            ),
            Some(Token { kind: '/', .. }) => match self.toks.peek_n(1) {
                Some(Token { kind: '/', .. }) => self.parse_silent_comment()?,
                Some(Token { kind: '*', .. }) => AstStmt::LoudComment(self.parse_loud_comment()?),
                _ => return child(self),
            },
            _ => return child(self),
        }))
    }

    fn looking_at_double_newline(&mut self) -> bool {
        match self.toks.peek() {
            // todo: is this branch reachable
            Some(Token { kind: '\r', .. }) => match self.toks.peek_n(1) {
                Some(Token { kind: '\n', .. }) => {
                    matches!(self.toks.peek_n(2), Some(Token { kind: '\n', .. }))
                }
                Some(Token { kind: '\r', .. }) => true,
                _ => false,
            },
            Some(Token { kind: '\n', .. }) => matches!(
                self.toks.peek_n(1),
                Some(Token {
                    kind: '\n' | '\r',
                    ..
                })
            ),
            _ => false,
        }
    }
}
