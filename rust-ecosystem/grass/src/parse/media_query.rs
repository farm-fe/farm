use crate::{ast::MediaQuery, error::SassResult, lexer::Lexer};

use super::BaseParser;

pub(crate) struct MediaQueryParser {
    pub toks: Lexer,
}

impl BaseParser for MediaQueryParser {
    fn toks(&self) -> &Lexer {
        &self.toks
    }

    fn toks_mut(&mut self) -> &mut Lexer {
        &mut self.toks
    }
}

impl MediaQueryParser {
    pub fn new(toks: Lexer) -> MediaQueryParser {
        MediaQueryParser { toks }
    }

    pub fn parse(&mut self) -> SassResult<Vec<MediaQuery>> {
        let mut queries = Vec::new();
        loop {
            self.whitespace()?;
            queries.push(self.parse_media_query()?);
            self.whitespace()?;

            if !self.scan_char(',') {
                break;
            }
        }

        if self.toks.next().is_some() {
            return Err(("expected no more input.", self.toks.current_span()).into());
        }

        Ok(queries)
    }

    fn parse_media_query(&mut self) -> SassResult<MediaQuery> {
        if self.toks.next_char_is('(') {
            let mut conditions = vec![self.parse_media_in_parens()?];
            self.whitespace()?;

            let mut conjunction = true;

            if self.scan_identifier("and", false)? {
                self.expect_whitespace()?;
                conditions.append(&mut self.parse_media_logic_sequence("and")?);
            } else if self.scan_identifier("or", false)? {
                self.expect_whitespace()?;
                conjunction = false;
                conditions.append(&mut self.parse_media_logic_sequence("or")?);
            }

            return Ok(MediaQuery::condition(conditions, conjunction));
        }

        let mut modifier: Option<String> = None;
        let media_type: Option<String>;
        let identifier1 = self.parse_identifier(false, false)?;

        if identifier1.to_ascii_lowercase() == "not" {
            self.expect_whitespace()?;
            if !self.looking_at_identifier() {
                return Ok(MediaQuery::condition(
                    vec![format!("(not {})", self.parse_media_in_parens()?)],
                    true,
                ));
            }
        }

        self.whitespace()?;

        if !self.looking_at_identifier() {
            return Ok(MediaQuery::media_type(Some(identifier1), None, None));
        }

        let identifier2 = self.parse_identifier(false, false)?;

        if identifier2.to_ascii_lowercase() == "and" {
            self.expect_whitespace()?;
            media_type = Some(identifier1);
        } else {
            self.whitespace()?;
            modifier = Some(identifier1);
            media_type = Some(identifier2);
            if self.scan_identifier("and", false)? {
                // For example, "@media only screen and ..."
                self.expect_whitespace()?;
            } else {
                // For example, "@media only screen {"
                return Ok(MediaQuery::media_type(media_type, modifier, None));
            }
        }

        // We've consumed either `IDENTIFIER "and"` or
        // `IDENTIFIER IDENTIFIER "and"`.

        if self.scan_identifier("not", false)? {
            // For example, "@media screen and not (...) {"
            self.expect_whitespace()?;
            return Ok(MediaQuery::media_type(
                media_type,
                modifier,
                Some(vec![format!("(not {})", self.parse_media_in_parens()?)]),
            ));
        }

        Ok(MediaQuery::media_type(
            media_type,
            modifier,
            Some(self.parse_media_logic_sequence("and")?),
        ))
    }

    fn parse_media_in_parens(&mut self) -> SassResult<String> {
        self.expect_char('(')?;
        let result = format!("({})", self.declaration_value(false)?);
        self.expect_char(')')?;
        Ok(result)
    }

    fn parse_media_logic_sequence(&mut self, operator: &'static str) -> SassResult<Vec<String>> {
        let mut result = Vec::new();
        loop {
            result.push(self.parse_media_in_parens()?);
            self.whitespace()?;
            if !self.scan_identifier(operator, false)? {
                return Ok(result);
            }
            self.expect_whitespace()?;
        }
    }
}
