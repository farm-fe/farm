use crate::{
    error::SassResult,
    lexer::Lexer,
    utils::{as_hex, hex_char_for, is_name, is_name_start, opposite_bracket},
    Token,
};

pub(crate) trait BaseParser {
    fn toks(&self) -> &Lexer;
    fn toks_mut(&mut self) -> &mut Lexer;

    fn whitespace_without_comments(&mut self) {
        while matches!(
            self.toks().peek(),
            Some(Token {
                kind: ' ' | '\t' | '\n',
                ..
            })
        ) {
            self.toks_mut().next();
        }
    }

    fn whitespace(&mut self) -> SassResult<()> {
        loop {
            self.whitespace_without_comments();

            if !self.scan_comment()? {
                break;
            }
        }

        Ok(())
    }

    fn scan_comment(&mut self) -> SassResult<bool> {
        if !matches!(self.toks().peek(), Some(Token { kind: '/', .. })) {
            return Ok(false);
        }

        Ok(match self.toks().peek_n(1) {
            Some(Token { kind: '/', .. }) => {
                self.skip_silent_comment()?;
                true
            }
            Some(Token { kind: '*', .. }) => {
                self.skip_loud_comment()?;
                true
            }
            _ => false,
        })
    }

    fn skip_silent_comment(&mut self) -> SassResult<()> {
        debug_assert!(self.next_matches("//"));
        self.toks_mut().next();
        self.toks_mut().next();
        while self.toks().peek().is_some() && !self.toks().next_char_is('\n') {
            self.toks_mut().next();
        }
        Ok(())
    }

    fn next_matches(&mut self, s: &str) -> bool {
        for (idx, c) in s.chars().enumerate() {
            match self.toks().peek_n(idx) {
                Some(Token { kind, .. }) if kind == c => {}
                _ => return false,
            }
        }

        true
    }

    fn skip_loud_comment(&mut self) -> SassResult<()> {
        debug_assert!(self.next_matches("/*"));
        self.toks_mut().next();
        self.toks_mut().next();

        while let Some(next) = self.toks_mut().next() {
            if next.kind != '*' {
                continue;
            }

            while self.scan_char('*') {}

            if self.scan_char('/') {
                return Ok(());
            }
        }

        Err(("expected more input.", self.toks().current_span()).into())
    }

    fn scan_char(&mut self, c: char) -> bool {
        if let Some(Token { kind, .. }) = self.toks().peek() {
            if kind == c {
                self.toks_mut().next();
                return true;
            }
        }

        false
    }

    fn scan(&mut self, s: &str) -> bool {
        let start = self.toks().cursor();
        for c in s.chars() {
            if !self.scan_char(c) {
                self.toks_mut().set_cursor(start);
                return false;
            }
        }

        true
    }

    fn expect_whitespace(&mut self) -> SassResult<()> {
        if !matches!(
            self.toks().peek(),
            Some(Token {
                kind: ' ' | '\t' | '\n' | '\r',
                ..
            })
        ) && !self.scan_comment()?
        {
            return Err(("Expected whitespace.", self.toks().current_span()).into());
        }

        self.whitespace()?;

        Ok(())
    }

    fn parse_identifier(
        &mut self,
        // default=false
        normalize: bool,
        // default=false
        unit: bool,
    ) -> SassResult<String> {
        let mut text = String::new();

        if self.scan_char('-') {
            text.push('-');

            if self.scan_char('-') {
                text.push('-');
                self.parse_identifier_body(&mut text, normalize, unit)?;
                return Ok(text);
            }
        }

        match self.toks().peek() {
            Some(Token { kind: '_', .. }) if normalize => {
                self.toks_mut().next();
                text.push('-');
            }
            Some(Token { kind, .. }) if is_name_start(kind) => {
                self.toks_mut().next();
                text.push(kind);
            }
            Some(Token { kind: '\\', .. }) => {
                text.push_str(&self.parse_escape(true)?);
            }
            Some(..) | None => {
                return Err(("Expected identifier.", self.toks().current_span()).into())
            }
        }

        self.parse_identifier_body(&mut text, normalize, unit)?;

        Ok(text)
    }

    fn parse_identifier_body(
        &mut self,
        buffer: &mut String,
        normalize: bool,
        unit: bool,
    ) -> SassResult<()> {
        while let Some(tok) = self.toks().peek() {
            if unit && tok.kind == '-' {
                // Disallow `-` followed by a dot or a digit digit in units.
                let second = match self.toks().peek_n(1) {
                    Some(v) => v,
                    None => break,
                };

                if second.kind == '.' || second.kind.is_ascii_digit() {
                    break;
                }

                self.toks_mut().next();
                buffer.push('-');
            } else if normalize && tok.kind == '_' {
                buffer.push('-');
                self.toks_mut().next();
            } else if is_name(tok.kind) {
                self.toks_mut().next();
                buffer.push(tok.kind);
            } else if tok.kind == '\\' {
                buffer.push_str(&self.parse_escape(false)?);
            } else {
                break;
            }
        }

        Ok(())
    }

    fn parse_escape(&mut self, identifier_start: bool) -> SassResult<String> {
        let start = self.toks().cursor();
        self.expect_char('\\')?;
        let mut value = 0;
        let first = match self.toks().peek() {
            Some(t) => t,
            None => return Err(("Expected expression.", self.toks().current_span()).into()),
        };
        if first.kind == '\n' {
            return Err(("Expected escape sequence.", self.toks().current_span()).into());
        } else if first.kind.is_ascii_hexdigit() {
            for _ in 0..6 {
                let next = match self.toks().peek() {
                    Some(t) => t,
                    None => break,
                };
                if !next.kind.is_ascii_hexdigit() {
                    break;
                }
                value *= 16;
                value += as_hex(next.kind);
                self.toks_mut().next();
            }
            if matches!(
                self.toks().peek(),
                Some(Token { kind: ' ', .. })
                    | Some(Token { kind: '\n', .. })
                    | Some(Token { kind: '\t', .. })
            ) {
                self.toks_mut().next();
            }
        } else {
            value = first.kind as u32;
            self.toks_mut().next();
        }

        let c = std::char::from_u32(value)
            .ok_or_else(|| ("Invalid Unicode code point.", self.toks().span_from(start)))?;
        if (identifier_start && is_name_start(c) && !c.is_ascii_digit())
            || (!identifier_start && is_name(c))
        {
            Ok(c.to_string())
        } else if value <= 0x1F || value == 0x7F || (identifier_start && c.is_ascii_digit()) {
            let mut buf = String::with_capacity(4);
            buf.push('\\');
            if value > 0xF {
                buf.push(hex_char_for(value >> 4));
            }
            buf.push(hex_char_for(value & 0xF));
            buf.push(' ');
            Ok(buf)
        } else {
            Ok(format!("\\{}", c))
        }
    }

    fn expect_char(&mut self, c: char) -> SassResult<()> {
        match self.toks().peek() {
            Some(tok) if tok.kind == c => {
                self.toks_mut().next();
                Ok(())
            }
            Some(..) | None => {
                Err((format!("expected \"{}\".", c), self.toks().current_span()).into())
            }
        }
    }

    fn expect_char_with_message(&mut self, c: char, msg: &'static str) -> SassResult<()> {
        match self.toks().peek() {
            Some(tok) if tok.kind == c => {
                self.toks_mut().next();
                Ok(())
            }
            Some(..) | None => Err((format!("expected {}.", msg), self.toks().prev_span()).into()),
        }
    }

    fn parse_string(&mut self) -> SassResult<String> {
        let quote = match self.toks_mut().next() {
            Some(Token {
                kind: q @ ('\'' | '"'),
                ..
            }) => q,
            Some(..) | None => return Err(("Expected string.", self.toks().current_span()).into()),
        };

        let mut buffer = String::new();

        let mut found_matching_quote = false;

        while let Some(next) = self.toks().peek() {
            if next.kind == quote {
                self.toks_mut().next();
                found_matching_quote = true;
                break;
            } else if next.kind == '\n' || next.kind == '\r' {
                break;
            } else if next.kind == '\\' {
                if matches!(
                    self.toks().peek_n(1),
                    Some(Token {
                        kind: '\n' | '\r',
                        ..
                    })
                ) {
                    self.toks_mut().next();
                    self.toks_mut().next();
                } else {
                    buffer.push(self.consume_escaped_char()?);
                }
            } else {
                self.toks_mut().next();
                buffer.push(next.kind);
            }
        }

        if !found_matching_quote {
            return Err((
                format!("Expected {quote}.", quote = quote),
                self.toks().current_span(),
            )
                .into());
        }

        Ok(buffer)
    }

    fn consume_escaped_char(&mut self) -> SassResult<char> {
        self.expect_char('\\')?;

        match self.toks().peek() {
            None => Ok('\u{FFFD}'),
            Some(Token {
                kind: '\n' | '\r', ..
            }) => Err(("Expected escape sequence.", self.toks().current_span()).into()),
            Some(Token { kind, .. }) if kind.is_ascii_hexdigit() => {
                let mut value = 0;
                for _ in 0..6 {
                    let next = match self.toks().peek() {
                        Some(c) => c,
                        None => break,
                    };
                    if !next.kind.is_ascii_hexdigit() {
                        break;
                    }
                    self.toks_mut().next();
                    value = (value << 4) + as_hex(next.kind);
                }

                if self.toks().peek().is_some()
                    && self.toks().peek().unwrap().kind.is_ascii_whitespace()
                {
                    self.toks_mut().next();
                }

                if value == 0 || (0xD800..=0xDFFF).contains(&value) || value >= 0x0010_FFFF {
                    Ok('\u{FFFD}')
                } else {
                    Ok(char::from_u32(value).unwrap())
                }
            }
            Some(Token { kind, .. }) => {
                self.toks_mut().next();
                Ok(kind)
            }
        }
    }

    fn declaration_value(&mut self, allow_empty: bool) -> SassResult<String> {
        let mut buffer = String::new();

        let mut brackets = Vec::new();
        let mut wrote_newline = false;

        while let Some(tok) = self.toks().peek() {
            match tok.kind {
                '\\' => {
                    buffer.push_str(&self.parse_escape(true)?);
                    wrote_newline = false;
                }
                '"' | '\'' => {
                    buffer.push_str(&self.fallible_raw_text(Self::parse_string)?);
                    wrote_newline = false;
                }
                '/' => {
                    if matches!(self.toks().peek_n(1), Some(Token { kind: '*', .. })) {
                        buffer.push_str(&self.fallible_raw_text(Self::skip_loud_comment)?);
                    } else {
                        buffer.push('/');
                        self.toks_mut().next();
                    }

                    wrote_newline = false;
                }
                '#' => {
                    if matches!(self.toks().peek_n(1), Some(Token { kind: '{', .. })) {
                        let s = self.parse_identifier(false, false)?;
                        buffer.push_str(&s);
                    } else {
                        buffer.push('#');
                        self.toks_mut().next();
                    }

                    wrote_newline = false;
                }
                c @ (' ' | '\t') => {
                    if wrote_newline
                        || !self
                            .toks()
                            .peek_n(1)
                            .map_or(false, |tok| tok.kind.is_ascii_whitespace())
                    {
                        buffer.push(c);
                    }

                    self.toks_mut().next();
                }
                '\n' | '\r' => {
                    if !wrote_newline {
                        buffer.push('\n');
                    }

                    wrote_newline = true;

                    self.toks_mut().next();
                }

                '[' | '(' | '{' => {
                    buffer.push(tok.kind);

                    self.toks_mut().next();

                    brackets.push(opposite_bracket(tok.kind));
                    wrote_newline = false;
                }
                ']' | ')' | '}' => {
                    if let Some(end) = brackets.pop() {
                        buffer.push(tok.kind);
                        self.expect_char(end)?;
                    } else {
                        break;
                    }

                    wrote_newline = false;
                }
                ';' => {
                    if brackets.is_empty() {
                        break;
                    }

                    self.toks_mut().next();
                    buffer.push(';');
                    wrote_newline = false;
                }
                'u' | 'U' => {
                    if let Some(url) = self.try_parse_url()? {
                        buffer.push_str(&url);
                    } else {
                        buffer.push(tok.kind);
                        self.toks_mut().next();
                    }

                    wrote_newline = false;
                }
                c => {
                    if self.looking_at_identifier() {
                        buffer.push_str(&self.parse_identifier(false, false)?);
                    } else {
                        self.toks_mut().next();
                        buffer.push(c);
                    }

                    wrote_newline = false;
                }
            }
        }

        if let Some(last) = brackets.pop() {
            self.expect_char(last)?;
        }

        if !allow_empty && buffer.is_empty() {
            return Err(("Expected token.", self.toks().current_span()).into());
        }

        Ok(buffer)
    }

    /// Returns whether the scanner is immediately before a plain CSS identifier.
    ///
    /// This is based on [the CSS algorithm][], but it assumes all backslashes
    /// start escapes.
    ///
    /// [the CSS algorithm]: https://drafts.csswg.org/css-syntax-3/#would-start-an-identifier
    fn looking_at_identifier(&self) -> bool {
        match self.toks().peek() {
            Some(Token { kind, .. }) if is_name_start(kind) || kind == '\\' => return true,
            Some(Token { kind: '-', .. }) => {}
            Some(..) | None => return false,
        }

        match self.toks().peek_n(1) {
            Some(Token { kind, .. }) if is_name_start(kind) || kind == '-' || kind == '\\' => true,
            Some(..) | None => false,
        }
    }

    fn try_parse_url(&mut self) -> SassResult<Option<String>> {
        let start = self.toks().cursor();

        if !self.scan_identifier("url", false)? {
            return Ok(None);
        }

        if !self.scan_char('(') {
            self.toks_mut().set_cursor(start);
            return Ok(None);
        }

        self.whitespace()?;

        // Match Ruby Sass's behavior: parse a raw URL() if possible, and if not
        // backtrack and re-parse as a function expression.
        let mut buffer = "url(".to_owned();

        while let Some(next) = self.toks().peek() {
            match next.kind {
                '\\' => {
                    buffer.push_str(&self.parse_escape(false)?);
                }
                '!' | '#' | '%' | '&' | '*'..='~' | '\u{80}'..=char::MAX => {
                    self.toks_mut().next();
                    buffer.push(next.kind);
                }
                ')' => {
                    self.toks_mut().next();
                    buffer.push(next.kind);

                    return Ok(Some(buffer));
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.whitespace_without_comments();

                    if !self.toks().next_char_is(')') {
                        break;
                    }
                }
                _ => break,
            }
        }

        self.toks_mut().set_cursor(start);
        Ok(None)
    }

    fn raw_text<T>(&mut self, func: impl Fn(&mut Self) -> T) -> String {
        let start = self.toks().cursor();
        func(self);
        self.toks().raw_text(start)
    }

    fn fallible_raw_text<T>(
        &mut self,
        func: impl Fn(&mut Self) -> SassResult<T>,
    ) -> SassResult<String> {
        let start = self.toks().cursor();
        func(self)?;
        Ok(self.toks().raw_text(start))
    }

    /// Peeks to see if the `ident` is at the current position. If it is,
    /// consume the identifier
    fn scan_identifier(
        &mut self,
        ident: &'static str,
        // default=false
        case_sensitive: bool,
    ) -> SassResult<bool> {
        if !self.looking_at_identifier() {
            return Ok(false);
        }

        let start = self.toks().cursor();

        if self.consume_identifier(ident, case_sensitive)? && !self.looking_at_identifier_body() {
            Ok(true)
        } else {
            self.toks_mut().set_cursor(start);
            Ok(false)
        }
    }

    fn consume_identifier(&mut self, ident: &str, case_sensitive: bool) -> SassResult<bool> {
        for c in ident.chars() {
            if !self.scan_ident_char(c, case_sensitive)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn scan_ident_char(&mut self, c: char, case_sensitive: bool) -> SassResult<bool> {
        let matches = |actual: char| {
            if case_sensitive {
                actual == c
            } else {
                actual.to_ascii_lowercase() == c.to_ascii_lowercase()
            }
        };

        Ok(match self.toks().peek() {
            Some(Token { kind, .. }) if matches(kind) => {
                self.toks_mut().next();
                true
            }
            Some(Token { kind: '\\', .. }) => {
                let start = self.toks().cursor();
                if matches(self.consume_escaped_char()?) {
                    return Ok(true);
                }
                self.toks_mut().set_cursor(start);
                false
            }
            Some(..) | None => false,
        })
    }

    fn expect_ident_char(&mut self, c: char, case_sensitive: bool) -> SassResult<()> {
        if self.scan_ident_char(c, case_sensitive)? {
            return Ok(());
        }

        Err((format!("Expected \"{}\".", c), self.toks().current_span()).into())
    }

    fn looking_at_identifier_body(&mut self) -> bool {
        matches!(self.toks().peek(), Some(t) if is_name(t.kind) || t.kind == '\\')
    }

    fn parse_variable_name(&mut self) -> SassResult<String> {
        self.expect_char('$')?;
        self.parse_identifier(true, false)
    }

    fn expect_identifier(&mut self, ident: &str, case_sensitive: bool) -> SassResult<()> {
        let start = self.toks().cursor();

        for c in ident.chars() {
            if !self.scan_ident_char(c, case_sensitive)? {
                return Err((
                    format!("Expected \"{}\".", ident),
                    self.toks_mut().span_from(start),
                )
                    .into());
            }
        }

        if !self.looking_at_identifier_body() {
            return Ok(());
        }

        Err((
            format!("Expected \"{}\".", ident),
            self.toks_mut().span_from(start),
        )
            .into())
    }

    // todo: not real impl
    fn expect_done(&mut self) -> SassResult<()> {
        debug_assert!(self.toks().peek().is_none());

        Ok(())
    }

    fn spaces(&mut self) {
        while self.toks().next_char_is(' ') || self.toks().next_char_is('\t') {
            self.toks_mut().next();
        }
    }
}
