use codemap::Span;

use crate::{common::unvendor, error::SassResult, lexer::Lexer, parse::BaseParser, Token};

use super::{
    Attribute, Combinator, ComplexSelector, ComplexSelectorComponent, CompoundSelector, Namespace,
    Pseudo, QualifiedName, SelectorList, SimpleSelector,
};

#[derive(PartialEq)]
enum DevouredWhitespace {
    /// Some whitespace was found
    Whitespace,
    /// A newline and potentially other whitespace was found
    Newline,
    /// No whitespace was found
    None,
}

/// Pseudo-class selectors that take unadorned selectors as arguments.
const SELECTOR_PSEUDO_CLASSES: [&str; 9] = [
    "not",
    "matches",
    "where",
    "is",
    "current",
    "any",
    "has",
    "host",
    "host-context",
];

/// Pseudo-element selectors that take unadorned selectors as arguments.
const SELECTOR_PSEUDO_ELEMENTS: [&str; 1] = ["slotted"];

pub(crate) struct SelectorParser {
    /// Whether this parser allows the parent selector `&`.
    allows_parent: bool,

    /// Whether this parser allows placeholder selectors beginning with `%`.
    allows_placeholder: bool,

    pub toks: Lexer,

    span: Span,
}

impl BaseParser for SelectorParser {
    fn toks(&self) -> &Lexer {
        &self.toks
    }

    fn toks_mut(&mut self) -> &mut Lexer {
        &mut self.toks
    }
}

impl SelectorParser {
    pub fn new(toks: Lexer, allows_parent: bool, allows_placeholder: bool, span: Span) -> Self {
        Self {
            toks,
            allows_parent,
            allows_placeholder,
            span,
        }
    }

    pub fn parse(mut self) -> SassResult<SelectorList> {
        let tmp = self.parse_selector_list()?;
        if self.toks.peek().is_some() {
            return Err(("expected selector.", self.span).into());
        }
        Ok(tmp)
    }

    fn parse_selector_list(&mut self) -> SassResult<SelectorList> {
        let mut components = vec![self.parse_complex_selector(false)?];

        self.whitespace()?;

        let mut line_break = false;

        while self.scan_char(',') {
            line_break = self.eat_whitespace() == DevouredWhitespace::Newline || line_break;
            match self.toks.peek() {
                Some(Token { kind: ',', .. }) => continue,
                Some(..) => {}
                None => break,
            }
            components.push(self.parse_complex_selector(line_break)?);

            line_break = false;
        }

        Ok(SelectorList {
            components,
            span: self.span,
        })
    }

    fn eat_whitespace(&mut self) -> DevouredWhitespace {
        let text = self.raw_text(Self::whitespace);

        if text.contains('\n') {
            DevouredWhitespace::Newline
        } else if !text.is_empty() {
            DevouredWhitespace::Whitespace
        } else {
            DevouredWhitespace::None
        }
    }

    /// Consumes a complex selector.
    ///
    /// If `line_break` is `true`, that indicates that there was a line break
    /// before this selector.
    fn parse_complex_selector(&mut self, line_break: bool) -> SassResult<ComplexSelector> {
        let mut components = Vec::new();

        loop {
            self.whitespace()?;

            // todo: can we do while let Some(..) = self.toks.peek() ?
            match self.toks.peek() {
                Some(Token { kind: '+', .. }) => {
                    self.toks.next();
                    components.push(ComplexSelectorComponent::Combinator(
                        Combinator::NextSibling,
                    ));
                }
                Some(Token { kind: '>', .. }) => {
                    self.toks.next();
                    components.push(ComplexSelectorComponent::Combinator(Combinator::Child));
                }
                Some(Token { kind: '~', .. }) => {
                    self.toks.next();
                    components.push(ComplexSelectorComponent::Combinator(
                        Combinator::FollowingSibling,
                    ));
                }
                Some(Token { kind: '[', .. })
                | Some(Token { kind: '.', .. })
                | Some(Token { kind: '#', .. })
                | Some(Token { kind: '%', .. })
                | Some(Token { kind: ':', .. })
                // todo: ampersand?
                | Some(Token { kind: '&', .. })
                | Some(Token { kind: '*', .. })
                | Some(Token { kind: '|', .. }) => {
                    components.push(ComplexSelectorComponent::Compound(
                        self.parse_compound_selector()?,
                    ));
                    if let Some(Token { kind: '&', .. }) = self.toks.peek() {
                        return Err(("\"&\" may only used at the beginning of a compound selector.", self.span).into());
                    }
                }
                Some(..) => {
                    if !self.looking_at_identifier() {
                        break;
                    }
                    components.push(ComplexSelectorComponent::Compound(
                        self.parse_compound_selector()?,
                    ));
                    if let Some(Token { kind: '&', .. }) = self.toks.peek() {
                        return Err(("\"&\" may only used at the beginning of a compound selector.", self.span).into());
                    }
                }
                None => break,
            }
        }

        if components.is_empty() {
            return Err(("expected selector.", self.span).into());
        }

        Ok(ComplexSelector::new(components, line_break))
    }

    fn parse_compound_selector(&mut self) -> SassResult<CompoundSelector> {
        let mut components = vec![self.parse_simple_selector(None)?];

        while let Some(Token { kind, .. }) = self.toks.peek() {
            if !is_simple_selector_start(kind) {
                break;
            }

            components.push(self.parse_simple_selector(Some(false))?);
        }

        Ok(CompoundSelector { components })
    }

    /// Consumes a simple selector.
    ///
    /// If `allows_parent` is `Some`, this will override `self.allows_parent`. If `allows_parent`
    /// is `None`, it will fallback to `self.allows_parent`.
    fn parse_simple_selector(&mut self, allows_parent: Option<bool>) -> SassResult<SimpleSelector> {
        match self.toks.peek() {
            Some(Token { kind: '[', .. }) => self.parse_attribute_selector(),
            Some(Token { kind: '.', .. }) => self.parse_class_selector(),
            Some(Token { kind: '#', .. }) => self.parse_id_selector(),
            Some(Token { kind: '%', .. }) => {
                if !self.allows_placeholder {
                    return Err(("Placeholder selectors aren't allowed here.", self.span).into());
                }
                self.parse_placeholder_selector()
            }
            Some(Token { kind: ':', .. }) => self.parse_pseudo_selector(),
            Some(Token { kind: '&', .. }) => {
                let allows_parent = allows_parent.unwrap_or(self.allows_parent);
                if !allows_parent {
                    return Err(("Parent selectors aren't allowed here.", self.span).into());
                }

                self.parse_parent_selector()
            }
            _ => self.parse_type_or_universal_selector(),
        }
    }

    fn parse_attribute_selector(&mut self) -> SassResult<SimpleSelector> {
        self.toks.next();
        Ok(SimpleSelector::Attribute(Box::new(Attribute::from_tokens(
            self,
        )?)))
    }

    fn parse_class_selector(&mut self) -> SassResult<SimpleSelector> {
        self.toks.next();
        Ok(SimpleSelector::Class(self.parse_identifier(false, false)?))
    }

    fn parse_id_selector(&mut self) -> SassResult<SimpleSelector> {
        self.toks.next();
        Ok(SimpleSelector::Id(self.parse_identifier(false, false)?))
    }

    fn parse_pseudo_selector(&mut self) -> SassResult<SimpleSelector> {
        self.toks.next();
        let element = self.scan_char(':');
        let name = self.parse_identifier(false, false)?;

        match self.toks.peek() {
            Some(Token { kind: '(', .. }) => self.toks.next(),
            _ => {
                return Ok(SimpleSelector::Pseudo(Pseudo {
                    is_class: !element && !is_fake_pseudo_element(&name),
                    name,
                    selector: None,
                    is_syntactic_class: !element,
                    argument: None,
                    span: self.span,
                }));
            }
        };

        self.whitespace()?;

        let unvendored = unvendor(&name);

        let mut argument: Option<Box<str>> = None;
        let mut selector: Option<Box<SelectorList>> = None;

        if element {
            // todo: lowercase?
            if SELECTOR_PSEUDO_ELEMENTS.contains(&unvendored) {
                selector = Some(Box::new(self.parse_selector_list()?));
                self.whitespace()?;
            } else {
                argument = Some(self.declaration_value(true)?.into_boxed_str());
            }

            self.expect_char(')')?;
        } else if SELECTOR_PSEUDO_CLASSES.contains(&unvendored) {
            selector = Some(Box::new(self.parse_selector_list()?));
            self.whitespace()?;
            self.expect_char(')')?;
        } else if unvendored == "nth-child" || unvendored == "nth-last-child" {
            let mut this_arg = self.parse_a_n_plus_b()?;
            self.whitespace()?;

            let last_was_whitespace = matches!(
                self.toks.peek_n_backwards(1),
                Some(Token {
                    kind: ' ' | '\t' | '\n' | '\r',
                    ..
                })
            );
            if last_was_whitespace && !matches!(self.toks.peek(), Some(Token { kind: ')', .. })) {
                self.expect_identifier("of", false)?;
                this_arg.push_str(" of");
                self.whitespace()?;
                selector = Some(Box::new(self.parse_selector_list()?));
            }

            self.expect_char(')')?;
            argument = Some(this_arg.into_boxed_str());
        } else {
            argument = Some(
                self.declaration_value(true)?
                    .trim_end()
                    .to_owned()
                    .into_boxed_str(),
            );

            self.expect_char(')')?;
        }

        Ok(SimpleSelector::Pseudo(Pseudo {
            is_class: !element && !is_fake_pseudo_element(&name),
            name,
            selector,
            is_syntactic_class: !element,
            argument,
            span: self.span,
        }))
    }

    fn parse_parent_selector(&mut self) -> SassResult<SimpleSelector> {
        self.toks.next();
        let suffix = if self.looking_at_identifier_body() {
            let mut buffer = String::new();
            self.parse_identifier_body(&mut buffer, false, false)?;
            Some(buffer)
        } else {
            None
        };
        Ok(SimpleSelector::Parent(suffix))
    }

    fn parse_placeholder_selector(&mut self) -> SassResult<SimpleSelector> {
        self.toks.next();
        Ok(SimpleSelector::Placeholder(
            self.parse_identifier(false, false)?,
        ))
    }

    /// Consumes a type selector or a universal selector.
    ///
    /// These are combined because either one could start with `*`.
    fn parse_type_or_universal_selector(&mut self) -> SassResult<SimpleSelector> {
        match self.toks.peek() {
            Some(Token { kind: '*', .. }) => {
                self.toks.next();
                if let Some(Token { kind: '|', .. }) = self.toks.peek() {
                    self.toks.next();
                    if let Some(Token { kind: '*', .. }) = self.toks.peek() {
                        self.toks.next();
                        return Ok(SimpleSelector::Universal(Namespace::Asterisk));
                    }

                    return Ok(SimpleSelector::Type(QualifiedName {
                        ident: self.parse_identifier(false, false)?,
                        namespace: Namespace::Asterisk,
                    }));
                }

                return Ok(SimpleSelector::Universal(Namespace::None));
            }
            Some(Token { kind: '|', .. }) => {
                self.toks.next();
                match self.toks.peek() {
                    Some(Token { kind: '*', .. }) => {
                        self.toks.next();
                        return Ok(SimpleSelector::Universal(Namespace::Empty));
                    }
                    _ => {
                        return Ok(SimpleSelector::Type(QualifiedName {
                            ident: self.parse_identifier(false, false)?,
                            namespace: Namespace::Empty,
                        }));
                    }
                }
            }
            _ => {}
        }

        let name_or_namespace = self.parse_identifier(false, false)?;

        Ok(match self.toks.peek() {
            Some(Token { kind: '|', .. }) => {
                self.toks.next();
                if let Some(Token { kind: '*', .. }) = self.toks.peek() {
                    self.toks.next();
                    SimpleSelector::Universal(Namespace::Other(name_or_namespace.into_boxed_str()))
                } else {
                    SimpleSelector::Type(QualifiedName {
                        ident: self.parse_identifier(false, false)?,
                        namespace: Namespace::Other(name_or_namespace.into_boxed_str()),
                    })
                }
            }
            Some(..) | None => SimpleSelector::Type(QualifiedName {
                ident: name_or_namespace,
                namespace: Namespace::None,
            }),
        })
    }

    /// Consumes an [`An+B` production][An+B] and returns its text.
    ///
    /// [An+B]: https://drafts.csswg.org/css-syntax-3/#anb-microsyntax
    fn parse_a_n_plus_b(&mut self) -> SassResult<String> {
        let mut buf = String::new();

        match self.toks.peek() {
            Some(Token { kind: 'e', .. }) | Some(Token { kind: 'E', .. }) => {
                self.expect_identifier("even", false)?;
                return Ok("even".to_owned());
            }
            Some(Token { kind: 'o', .. }) | Some(Token { kind: 'O', .. }) => {
                self.expect_identifier("odd", false)?;
                return Ok("odd".to_owned());
            }
            Some(t @ Token { kind: '+', .. }) | Some(t @ Token { kind: '-', .. }) => {
                buf.push(t.kind);
                self.toks.next();
            }
            _ => {}
        }

        match self.toks.peek() {
            Some(t) if t.kind.is_ascii_digit() => {
                while let Some(t) = self.toks.peek() {
                    if !t.kind.is_ascii_digit() {
                        break;
                    }
                    buf.push(t.kind);
                    self.toks.next();
                }
                self.whitespace()?;
                if !self.scan_ident_char('n', false)? {
                    return Ok(buf);
                }
            }
            Some(..) => self.expect_ident_char('n', false)?,
            None => return Err(("expected more input.", self.span).into()),
        }

        buf.push('n');

        self.whitespace()?;

        if let Some(t @ Token { kind: '+', .. }) | Some(t @ Token { kind: '-', .. }) =
            self.toks.peek()
        {
            buf.push(t.kind);
            self.toks.next();
            self.whitespace()?;
            match self.toks.peek() {
                Some(t) if !t.kind.is_ascii_digit() => {
                    return Err(("Expected a number.", self.span).into())
                }
                None => return Err(("Expected a number.", self.span).into()),
                Some(..) => {}
            }

            while let Some(t) = self.toks.peek() {
                if !t.kind.is_ascii_digit() {
                    break;
                }
                buf.push(t.kind);
                self.toks.next();
            }
        }
        Ok(buf)
    }
}

/// Returns whether `c` can start a simple selector other than a type
/// selector.
fn is_simple_selector_start(c: char) -> bool {
    matches!(c, '*' | '[' | '.' | '#' | '%' | ':')
}

/// Returns whether `name` is the name of a pseudo-element that can be written
/// with pseudo-class syntax (`:before`, `:after`, `:first-line`, or
/// `:first-letter`)
fn is_fake_pseudo_element(name: &str) -> bool {
    match name.as_bytes().first() {
        Some(b'a') | Some(b'A') => name.to_ascii_lowercase() == "after",
        Some(b'b') | Some(b'B') => name.to_ascii_lowercase() == "before",
        Some(b'f') | Some(b'F') => matches!(
            name.to_ascii_lowercase().as_str(),
            "first-line" | "first-letter"
        ),
        _ => false,
    }
}
