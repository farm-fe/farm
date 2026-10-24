use std::{iter::Peekable, str::Chars, sync::Arc};

use codemap::{File, Span};

const FORM_FEED: char = '\x0C';

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Token {
    pub kind: char,
    pos: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct Lexer {
    buf: Vec<Token>,
    entire_span: Span,
    cursor: usize,
    /// If the input this lexer is spanned over is larger than the original span.
    /// This is possible due to interpolation.
    is_expanded: bool,
}

impl Lexer {
    pub fn raw_text(&self, start: usize) -> String {
        self.buf[start..self.cursor]
            .iter()
            .map(|t| t.kind)
            .collect()
    }

    pub fn next_char_is(&self, c: char) -> bool {
        matches!(self.peek(), Some(Token { kind, .. }) if kind == c)
    }

    /// Gets the span of the character at the given index. If the index is out of
    /// bounds, it returns the span of the last character. If the input is empty,
    /// it returns an empty span
    fn span_at_index(&self, idx: usize) -> Span {
        if self.is_expanded {
            return self.entire_span;
        }

        let (start, len) = match self.buf.get(idx) {
            Some(tok) => (tok.pos, tok.kind.len_utf8()),
            None => match self.buf.last() {
                Some(tok) => (tok.pos, tok.kind.len_utf8()),
                None => (0, 0),
            },
        };

        self.entire_span
            .subspan(start as u64, start as u64 + len as u64)
    }

    pub fn span_from(&self, start: usize) -> Span {
        let start = self.span_at_index(start);
        let end = self.prev_span();

        start.merge(end)
    }

    pub fn prev_span(&self) -> Span {
        self.span_at_index(self.cursor.saturating_sub(1))
    }

    pub fn current_span(&self) -> Span {
        self.span_at_index(self.cursor)
    }

    pub fn peek(&self) -> Option<Token> {
        self.buf.get(self.cursor).copied()
    }

    /// Peeks the previous token without modifying the peek cursor
    pub fn peek_previous(&mut self) -> Option<Token> {
        self.buf.get(self.cursor.checked_sub(1)?).copied()
    }

    /// Peeks `n` from current peeked position without modifying cursor
    pub fn peek_n(&self, n: usize) -> Option<Token> {
        self.buf.get(self.cursor + n).copied()
    }

    /// Peeks `n` behind current peeked position without modifying cursor
    pub fn peek_n_backwards(&self, n: usize) -> Option<Token> {
        self.buf.get(self.cursor.checked_sub(n)?).copied()
    }

    /// Set cursor to position and reset peek
    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.get(self.cursor).copied().map(|tok| {
            self.cursor += 1;
            tok
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buf.len() - self.cursor;
        (remaining, Some(remaining))
    }
}

/// Lex a string into a series of tokens
pub(crate) struct TokenLexer<'a> {
    buf: Peekable<Chars<'a>>,
    cursor: u32,
}

// todo: maybe char indices?
impl<'a> TokenLexer<'a> {
    pub fn new(buf: Peekable<Chars<'a>>) -> TokenLexer<'a> {
        Self { buf, cursor: 0 }
    }
}

impl<'a> Iterator for TokenLexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        let kind = match self.buf.next()? {
            FORM_FEED => '\n',
            '\r' => {
                if self.buf.peek() == Some(&'\n') {
                    self.cursor += 1;
                    self.buf.next();
                }
                '\n'
            }
            c => c,
        };
        let len = kind.len_utf8() as u32;
        let pos = self.cursor;
        self.cursor += len;
        Some(Token { pos, kind })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.buf.size_hint()
    }
}

impl Lexer {
    pub fn new_from_file(file: &Arc<File>) -> Self {
        let buf = TokenLexer::new(file.source().chars().peekable()).collect();
        Self::new(buf, file.span, false)
    }

    pub fn new_from_string(s: &str, entire_span: Span) -> Self {
        let is_expanded = s.len() as u64 > entire_span.len();
        let buf = TokenLexer::new(s.chars().peekable()).collect();

        Self::new(buf, entire_span, is_expanded)
    }

    fn new(buf: Vec<Token>, entire_span: Span, is_expanded: bool) -> Self {
        Lexer {
            buf,
            cursor: 0,
            entire_span,
            is_expanded,
        }
    }
}
