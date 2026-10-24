use std::{cmp::Ordering, sync::Arc};

use codemap::{Span, Spanned};

use crate::{
    color::Color,
    common::{BinaryOp, Brackets, ListSeparator, QuoteKind},
    error::SassResult,
    evaluate::Visitor,
    selector::Selector,
    serializer::{inspect_value, serialize_value},
    unit::Unit,
    utils::is_special_function,
    Options, OutputStyle,
};

pub use arglist::ArgList;
pub use calculation::*;
pub use map::SassMap;
pub use number::*;
pub use sass_function::{SassFunction, UserDefinedFunction};
pub(crate) use sass_number::conversion_factor;
pub use sass_number::SassNumber;

mod arglist;
mod calculation;
mod map;
mod number;
mod sass_function;
mod sass_number;

#[derive(Debug, Clone)]
pub enum Value {
    True,
    False,
    Null,
    Dimension(SassNumber),
    List(Vec<Value>, ListSeparator, Brackets),
    Color(Arc<Color>),
    String(String, QuoteKind),
    Map(SassMap),
    ArgList(ArgList),
    /// Returned by `get-function()`
    FunctionRef(Box<SassFunction>),
    Calculation(SassCalculation),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Calculation(calc1) => match other {
                Value::Calculation(calc2) => calc1 == calc2,
                _ => false,
            },
            Value::String(s1, ..) => match other {
                Value::String(s2, ..) => s1 == s2,
                _ => false,
            },
            Value::Dimension(n1) => match other {
                Value::Dimension(n2) => n1 == n2,
                _ => false,
            },
            Value::List(list1, sep1, brackets1) => match other {
                Value::List(list2, sep2, brackets2) => {
                    if sep1 != sep2 || brackets1 != brackets2 || list1.len() != list2.len() {
                        false
                    } else {
                        for (a, b) in list1.iter().zip(list2) {
                            if a != b {
                                return false;
                            }
                        }
                        true
                    }
                }
                _ => false,
            },
            Value::Null => matches!(other, Value::Null),
            Value::True => matches!(other, Value::True),
            Value::False => matches!(other, Value::False),
            Value::FunctionRef(fn1) => {
                if let Value::FunctionRef(fn2) = other {
                    fn1 == fn2
                } else {
                    false
                }
            }
            Value::Map(map1) => {
                if let Value::Map(map2) = other {
                    map1 == map2
                } else {
                    false
                }
            }
            Value::Color(color1) => {
                if let Value::Color(color2) = other {
                    color1 == color2
                } else {
                    false
                }
            }
            Value::ArgList(list1) => match other {
                Value::ArgList(list2) => list1 == list2,
                Value::List(list2, ListSeparator::Comma, ..) => {
                    if list1.len() != list2.len() {
                        return false;
                    }

                    for (el1, el2) in list1.elems.iter().zip(list2) {
                        if el1 != el2 {
                            return false;
                        }
                    }

                    true
                }
                _ => false,
            },
        }
    }
}

impl Eq for Value {}

impl Value {
    pub fn with_slash(
        self,
        numerator: SassNumber,
        denom: SassNumber,
        span: Span,
    ) -> SassResult<Self> {
        let mut number = self.assert_number(span)?;
        number.as_slash = Some(Arc::new((numerator, denom)));
        Ok(Value::Dimension(number))
    }

    pub fn assert_number(self, span: Span) -> SassResult<SassNumber> {
        match self {
            Value::Dimension(n) => Ok(n),
            _ => Err((format!("{} is not a number.", self.inspect(span)?), span).into()),
        }
    }

    pub fn assert_number_with_name(self, name: &str, span: Span) -> SassResult<SassNumber> {
        match self {
            Value::Dimension(n) => Ok(n),
            _ => Err((
                format!(
                    "${name}: {} is not a number.",
                    self.inspect(span)?,
                    name = name,
                ),
                span,
            )
                .into()),
        }
    }

    pub fn assert_color_with_name(self, name: &str, span: Span) -> SassResult<Arc<Color>> {
        match self {
            Value::Color(c) => Ok(c),
            _ => Err((
                format!(
                    "${name}: {} is not a color.",
                    self.inspect(span)?,
                    name = name,
                ),
                span,
            )
                .into()),
        }
    }

    pub fn assert_map_with_name(self, name: &str, span: Span) -> SassResult<SassMap> {
        match self {
            Value::Map(m) => Ok(m),
            Value::List(v, ..) if v.is_empty() => Ok(SassMap::new()),
            Value::ArgList(v) if v.is_empty() => Ok(SassMap::new()),
            _ => Err((
                format!(
                    "${name}: {} is not a map.",
                    self.inspect(span)?,
                    name = name,
                ),
                span,
            )
                .into()),
        }
    }

    pub fn assert_string_with_name(
        self,
        name: &str,
        span: Span,
    ) -> SassResult<(String, QuoteKind)> {
        match self {
            Value::String(s, quotes) => Ok((s, quotes)),
            _ => Err((
                format!(
                    "${name}: {} is not a string.",
                    self.inspect(span)?,
                    name = name,
                ),
                span,
            )
                .into()),
        }
    }

    pub fn is_blank(&self) -> bool {
        match self {
            Value::Null => true,
            Value::String(i, QuoteKind::None) if i.is_empty() => true,
            Value::List(_, _, Brackets::Bracketed) => false,
            Value::List(v, ..) => v.iter().all(Value::is_blank),
            Value::ArgList(v, ..) => v.is_blank(),
            _ => false,
        }
    }

    pub fn is_empty_list(&self) -> bool {
        match self {
            Value::List(v, ..) => v.is_empty(),
            Value::Map(m) => m.is_empty(),
            Value::ArgList(v) => v.elems.is_empty(),
            _ => false,
        }
    }

    pub fn to_css_string(&self, span: Span, is_compressed: bool) -> SassResult<String> {
        serialize_value(
            self,
            &Options::default().style(if is_compressed {
                OutputStyle::Compressed
            } else {
                OutputStyle::Expanded
            }),
            span,
        )
    }

    pub fn inspect(&self, span: Span) -> SassResult<String> {
        inspect_value(self, &Options::default(), span)
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Null | Value::False)
    }

    pub fn unquote(self) -> Self {
        match self {
            Value::String(s1, _) => Value::String(s1, QuoteKind::None),
            Value::List(v, sep, bracket) => {
                Value::List(v.into_iter().map(Value::unquote).collect(), sep, bracket)
            }
            v => v,
        }
    }

    pub const fn span(self, span: Span) -> Spanned<Self> {
        Spanned { node: self, span }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Value::Color(..) => "color",
            Value::String(..) => "string",
            Value::Calculation(..) => "calculation",
            Value::Dimension(..) => "number",
            Value::List(..) => "list",
            Value::FunctionRef(..) => "function",
            Value::ArgList(..) => "arglist",
            Value::True | Value::False => "bool",
            Value::Null => "null",
            Value::Map(..) => "map",
        }
    }

    pub fn as_slash(&self) -> Option<Arc<(SassNumber, SassNumber)>> {
        match self {
            Value::Dimension(SassNumber { as_slash, .. }) => as_slash.clone(),
            _ => None,
        }
    }

    pub fn without_slash(self) -> Self {
        match self {
            Value::Dimension(SassNumber {
                num,
                unit,
                as_slash: _,
            }) => Value::Dimension(SassNumber {
                num,
                unit,
                as_slash: None,
            }),
            _ => self,
        }
    }

    pub fn is_special_function(&self) -> bool {
        match self {
            Value::String(s, QuoteKind::None) => is_special_function(s),
            Value::Calculation(..) => true,
            _ => false,
        }
    }

    pub fn is_var(&self) -> bool {
        match self {
            Value::String(s, QuoteKind::None) => {
                if s.len() < "var(--_)".len() {
                    return false;
                }

                s.starts_with("var(")
            }
            Value::Calculation(..) => true,
            _ => false,
        }
    }

    pub fn try_map(&self) -> Option<SassMap> {
        match &self {
            Value::Map(m) => Some(m.clone()),
            Value::List(v, ..) if v.is_empty() => Some(SassMap::new()),
            Value::ArgList(v) if v.is_empty() => Some(SassMap::new()),
            _ => None,
        }
    }

    pub fn bool(b: bool) -> Self {
        if b {
            Value::True
        } else {
            Value::False
        }
    }

    pub fn cmp(&self, other: &Self, span: Span, op: BinaryOp) -> SassResult<Option<Ordering>> {
        Ok(match self {
            Value::Dimension(SassNumber { num, unit, .. }) => match &other {
                Value::Dimension(SassNumber {
                    num: num2,
                    unit: unit2,
                    ..
                }) => {
                    if !unit.comparable(unit2) {
                        return Err(
                            (format!("Incompatible units {} and {}.", unit2, unit), span).into(),
                        );
                    }
                    if unit == unit2 || unit == &Unit::None || unit2 == &Unit::None {
                        num.partial_cmp(num2)
                    } else {
                        num.partial_cmp(&num2.convert(unit2, unit))
                    }
                }
                _ => {
                    return Err((
                        format!(
                            "Undefined operation \"{} {} {}\".",
                            self.inspect(span)?,
                            op,
                            other.inspect(span)?
                        ),
                        span,
                    )
                        .into())
                }
            },
            _ => {
                return Err((
                    format!(
                        "Undefined operation \"{} {} {}\".",
                        self.inspect(span)?,
                        op,
                        other.inspect(span)?
                    ),
                    span,
                )
                    .into());
            }
        })
    }

    pub fn not_equals(&self, other: &Self) -> bool {
        match self {
            Value::String(s1, ..) => match other {
                Value::String(s2, ..) => s1 != s2,
                _ => true,
            },
            Value::Dimension(SassNumber {
                num: n,
                unit,
                as_slash: _,
            }) if !n.is_nan() => match other {
                Value::Dimension(SassNumber {
                    num: n2,
                    unit: unit2,
                    as_slash: _,
                }) if !n2.is_nan() => {
                    if !unit.comparable(unit2) {
                        true
                    } else if unit == unit2 {
                        n != n2
                    } else if unit == &Unit::None || unit2 == &Unit::None {
                        true
                    } else {
                        n != &n2.convert(unit2, unit)
                    }
                }
                _ => true,
            },
            Value::List(list1, sep1, brackets1) => match other {
                Value::List(list2, sep2, brackets2) => {
                    if sep1 != sep2 || brackets1 != brackets2 || list1.len() != list2.len() {
                        true
                    } else {
                        for (a, b) in list1.iter().zip(list2) {
                            if a.not_equals(b) {
                                return true;
                            }
                        }
                        false
                    }
                }
                _ => true,
            },
            s => s != other,
        }
    }

    pub fn as_list(self) -> Vec<Value> {
        match self {
            Value::List(v, ..) => v,
            Value::Map(m) => m.as_list(),
            Value::ArgList(v) => v.elems,
            v => vec![v],
        }
    }

    pub fn separator(&self) -> ListSeparator {
        match self {
            Value::List(_, list_separator, _) => *list_separator,
            Value::Map(..) | Value::ArgList(..) => ListSeparator::Comma,
            _ => ListSeparator::Space,
        }
    }

    /// Parses `self` as a selector list, in the same manner as the
    /// `selector-parse()` function.
    ///
    /// Returns a `SassError` if `self` isn't a type that can be parsed as a
    /// selector, or if parsing fails. If `allow_parent` is `true`, this allows
    /// parent selectors. Otherwise, they're considered parse errors.
    ///
    /// `name` is the argument name. It's used for error reporting.
    pub fn to_selector(
        self,
        visitor: &mut Visitor,
        name: &str,
        allows_parent: bool,
        span: Span,
    ) -> SassResult<Selector> {
        let string = match self.clone().selector_string()? {
            Some(v) => v,
            None => return Err((format!("${}: {} is not a valid selector: it must be a string,\n a list of strings, or a list of lists of strings.", name, self.inspect(span)?), span).into()),
        };
        Ok(Selector(visitor.parse_selector_from_string(
            &string,
            allows_parent,
            true,
            span,
        )?))
    }

    fn selector_string(self) -> SassResult<Option<String>> {
        Ok(Some(match self {
            Value::String(text, ..) => text,
            Value::List(list, sep, ..) if !list.is_empty() => {
                let mut result = Vec::new();
                match sep {
                    ListSeparator::Comma => {
                        for complex in list {
                            if let Value::String(text, ..) = complex {
                                result.push(text);
                            } else if let Value::List(
                                _,
                                ListSeparator::Space | ListSeparator::Undecided,
                                ..,
                            ) = complex
                            {
                                result.push(match complex.selector_string()? {
                                    Some(v) => v,
                                    None => return Ok(None),
                                });
                            } else {
                                return Ok(None);
                            }
                        }
                    }
                    ListSeparator::Slash => return Ok(None),
                    ListSeparator::Space | ListSeparator::Undecided => {
                        for compound in list {
                            if let Value::String(text, ..) = compound {
                                result.push(text);
                            } else {
                                return Ok(None);
                            }
                        }
                    }
                }

                result.join(sep.as_str())
            }
            _ => return Ok(None),
        }))
    }

    pub fn unary_plus(self, visitor: &mut Visitor, span: Span) -> SassResult<Self> {
        Ok(match self {
            Self::Dimension(SassNumber { .. }) => self,
            Self::Calculation(..) => {
                return Err((
                    format!("Undefined operation \"+{}\".", self.inspect(span)?),
                    span,
                )
                    .into())
            }
            _ => Self::String(
                format!(
                    "+{}",
                    &self.to_css_string(span, visitor.options.is_compressed())?
                ),
                QuoteKind::None,
            ),
        })
    }

    pub fn unary_neg(self, visitor: &mut Visitor, span: Span) -> SassResult<Self> {
        Ok(match self {
            Self::Calculation(..) => {
                return Err((
                    format!("Undefined operation \"-{}\".", self.inspect(span)?),
                    span,
                )
                    .into())
            }
            Self::Dimension(SassNumber {
                num,
                unit,
                as_slash,
            }) => Self::Dimension(SassNumber {
                num: -num,
                unit,
                as_slash,
            }),
            _ => Self::String(
                format!(
                    "-{}",
                    &self.to_css_string(span, visitor.options.is_compressed())?
                ),
                QuoteKind::None,
            ),
        })
    }

    pub fn unary_div(self, visitor: &mut Visitor, span: Span) -> SassResult<Self> {
        Ok(Self::String(
            format!(
                "/{}",
                &self.to_css_string(span, visitor.options.is_compressed())?
            ),
            QuoteKind::None,
        ))
    }

    pub fn unary_not(self) -> Self {
        match self {
            Self::False | Self::Null => Self::True,
            _ => Self::False,
        }
    }
}
