use std::collections::HashSet;

use crate::{ast::AtRootQuery, error::SassResult, lexer::Lexer};

use super::BaseParser;

pub(crate) struct AtRootQueryParser {
    toks: Lexer,
}

impl BaseParser for AtRootQueryParser {
    fn toks(&self) -> &Lexer {
        &self.toks
    }

    fn toks_mut(&mut self) -> &mut Lexer {
        &mut self.toks
    }
}

impl AtRootQueryParser {
    pub fn new(toks: Lexer) -> AtRootQueryParser {
        AtRootQueryParser { toks }
    }

    pub fn parse(&mut self) -> SassResult<AtRootQuery> {
        self.expect_char('(')?;
        self.whitespace()?;
        let include = self.scan_identifier("with", false)?;

        if !include {
            self.expect_identifier("without", false)?;
        }

        self.whitespace()?;
        self.expect_char(':')?;
        self.whitespace()?;

        let mut names = HashSet::new();

        loop {
            names.insert(self.parse_identifier(false, false)?.to_ascii_lowercase());
            self.whitespace()?;

            if !self.looking_at_identifier() {
                break;
            }
        }

        self.expect_char(')')?;
        self.expect_done()?;

        Ok(AtRootQuery::new(include, names))
    }
}
