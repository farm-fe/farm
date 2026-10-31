use std::{
    fmt::{self, Display, Write},
    hash::{Hash, Hasher},
};

use codemap::Span;

use crate::{
    common::QuoteKind, error::SassResult, parse::BaseParser, utils::is_ident, value::Value, Token,
};

use super::{Namespace, QualifiedName, SelectorParser};

#[derive(Clone, Debug)]
pub(crate) struct Attribute {
    attr: QualifiedName,
    value: String,
    modifier: Option<char>,
    op: AttributeOp,
    span: Span,
}

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        self.attr == other.attr
            && self.value == other.value
            && self.modifier == other.modifier
            && self.op == other.op
    }
}

impl Eq for Attribute {}

impl Hash for Attribute {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.attr.hash(state);
        self.value.hash(state);
        self.modifier.hash(state);
        self.op.hash(state);
    }
}

// todo: rewrite
fn attribute_name(parser: &mut SelectorParser) -> SassResult<QualifiedName> {
    let next = parser
        .toks
        .peek()
        .ok_or_else(|| ("Expected identifier.", parser.toks.current_span()))?;
    if next.kind == '*' {
        parser.toks.next();
        parser.expect_char('|')?;

        let ident = parser.parse_identifier(false, false)?;
        return Ok(QualifiedName {
            ident,
            namespace: Namespace::Asterisk,
        });
    }

    let name_or_namespace = parser.parse_identifier(false, false)?;
    match parser.toks.peek() {
        Some(v) if v.kind != '|' => {
            return Ok(QualifiedName {
                ident: name_or_namespace,
                namespace: Namespace::None,
            });
        }
        Some(..) => {}
        None => return Err(("expected more input.", parser.toks.current_span()).into()),
    }
    match parser.toks.peek_n(1) {
        Some(v) if v.kind == '=' => {
            return Ok(QualifiedName {
                ident: name_or_namespace,
                namespace: Namespace::None,
            });
        }
        Some(..) => {}
        None => return Err(("expected more input.", parser.toks.current_span()).into()),
    }
    parser.toks.next();
    let ident = parser.parse_identifier(false, false)?;
    Ok(QualifiedName {
        ident,
        namespace: Namespace::Other(name_or_namespace.into_boxed_str()),
    })
}

fn attribute_operator(parser: &mut SelectorParser) -> SassResult<AttributeOp> {
    let op = match parser.toks.next() {
        Some(Token { kind: '=', .. }) => return Ok(AttributeOp::Equals),
        Some(Token { kind: '~', .. }) => AttributeOp::Include,
        Some(Token { kind: '|', .. }) => AttributeOp::Dash,
        Some(Token { kind: '^', .. }) => AttributeOp::Prefix,
        Some(Token { kind: '$', .. }) => AttributeOp::Suffix,
        Some(Token { kind: '*', .. }) => AttributeOp::Contains,
        Some(..) | None => return Err(("Expected \"]\".", parser.toks.current_span()).into()),
    };

    parser.expect_char('=')?;

    Ok(op)
}
impl Attribute {
    pub fn from_tokens(parser: &mut SelectorParser) -> SassResult<Attribute> {
        let start = parser.toks.cursor();
        parser.whitespace_without_comments();
        let attr = attribute_name(parser)?;
        parser.whitespace_without_comments();
        if parser
            .toks
            .peek()
            .ok_or_else(|| ("expected more input.", parser.toks.current_span()))?
            .kind
            == ']'
        {
            parser.toks.next();
            return Ok(Attribute {
                attr,
                value: String::new(),
                modifier: None,
                op: AttributeOp::Any,
                span: parser.toks.span_from(start),
            });
        }

        let op = attribute_operator(parser)?;
        parser.whitespace_without_comments();

        let peek = parser
            .toks
            .peek()
            .ok_or_else(|| ("expected more input.", parser.toks.current_span()))?;

        let value = match peek.kind {
            '\'' | '"' => parser.parse_string()?,
            _ => parser.parse_identifier(false, false)?,
        };
        parser.whitespace_without_comments();

        let modifier = match parser.toks.peek() {
            Some(Token {
                kind: c @ 'a'..='z',
                ..
            })
            | Some(Token {
                kind: c @ 'A'..='Z',
                ..
            }) => {
                parser.toks.next();
                parser.whitespace_without_comments();
                Some(c)
            }
            _ => None,
        };

        parser.expect_char(']')?;

        Ok(Attribute {
            op,
            attr,
            value,
            modifier,
            span: parser.toks.span_from(start),
        })
    }
}

impl Display for Attribute {
    #[allow(clippy::branches_sharing_code)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('[')?;
        write!(f, "{}", self.attr)?;

        if self.op != AttributeOp::Any {
            f.write_str(self.op.into())?;
            if is_ident(&self.value) && !self.value.starts_with("--") {
                f.write_str(&self.value)?;

                if self.modifier.is_some() {
                    f.write_char(' ')?;
                }
            } else {
                // todo: remove unwrap by not doing this in display
                // or having special emitter for quoted strings?
                // (also avoids the clone because we can consume/modify self)
                f.write_str(
                    &Value::String(self.value.clone(), QuoteKind::Quoted)
                        .to_css_string(self.span, false)
                        .unwrap(),
                )?;
                // todo: this space is not emitted when `compressed` output
                if self.modifier.is_some() {
                    f.write_char(' ')?;
                }
            }

            if let Some(c) = self.modifier {
                f.write_char(c)?;
            }
        }

        f.write_char(']')?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum AttributeOp {
    /// \[attr\]
    ///
    /// Represents elements with an attribute name of `attr`
    Any,

    /// [attr=value]
    ///
    /// Represents elements with an attribute name of `attr`
    /// whose value is exactly `value`
    Equals,

    /// [attr~=value]
    ///
    /// Represents elements with an attribute name of `attr`
    /// whose value is a whitespace-separated list of words,
    /// one of which is exactly `value`
    Include,

    /// [attr|=value]
    ///
    /// Represents elements with an attribute name of `attr`
    /// whose value can be exactly value or can begin with
    /// `value` immediately followed by a hyphen (`-`)
    Dash,

    /// [attr^=value]
    Prefix,

    /// [attr$=value]
    Suffix,

    /// [attr*=value]
    ///
    /// Represents elements with an attribute name of `attr`
    /// whose value contains at least one occurrence of
    /// `value` within the string
    Contains,
}

impl From<AttributeOp> for &'static str {
    #[inline]
    fn from(op: AttributeOp) -> Self {
        match op {
            AttributeOp::Any => "",
            AttributeOp::Equals => "=",
            AttributeOp::Include => "~=",
            AttributeOp::Dash => "|=",
            AttributeOp::Prefix => "^=",
            AttributeOp::Suffix => "$=",
            AttributeOp::Contains => "*=",
        }
    }
}
