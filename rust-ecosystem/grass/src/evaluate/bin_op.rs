use std::cmp::Ordering;

use codemap::Span;

use crate::{
    common::{BinaryOp, QuoteKind},
    error::SassResult,
    unit::Unit,
    value::{SassNumber, Value},
    Options,
};

pub(crate) fn add(left: Value, right: Value, options: &Options, span: Span) -> SassResult<Value> {
    Ok(match left {
        Value::Calculation(..) => match right {
            Value::String(s, quotes) => Value::String(
                format!(
                    "{}{}",
                    left.to_css_string(span, options.is_compressed())?,
                    s
                ),
                quotes,
            ),
            _ => {
                return Err((
                    format!(
                        "Undefined operation \"{} + {}\".",
                        left.inspect(span)?,
                        right.inspect(span)?
                    ),
                    span,
                )
                    .into())
            }
        },
        Value::Map(..) | Value::FunctionRef(..) => {
            return Err((
                format!("{} isn't a valid CSS value.", left.inspect(span)?),
                span,
            )
                .into())
        }
        Value::True | Value::False => match right {
            Value::String(s, QuoteKind::Quoted) => Value::String(
                format!(
                    "{}{}",
                    left.to_css_string(span, options.is_compressed())?,
                    s
                ),
                QuoteKind::Quoted,
            ),
            _ => Value::String(
                format!(
                    "{}{}",
                    left.to_css_string(span, options.is_compressed())?,
                    right.to_css_string(span, options.is_compressed())?
                ),
                QuoteKind::None,
            ),
        },
        Value::Null => match right {
            Value::Null => Value::Null,
            _ => Value::String(
                right.to_css_string(span, options.is_compressed())?,
                QuoteKind::None,
            ),
        },
        Value::Dimension(SassNumber {
            num,
            unit,
            as_slash: _,
        }) => match right {
            Value::Dimension(SassNumber {
                num: num2,
                unit: unit2,
                as_slash: _,
            }) => {
                if !unit.comparable(&unit2) {
                    return Err(
                        (format!("Incompatible units {} and {}.", unit2, unit), span).into(),
                    );
                }
                if unit == unit2 {
                    Value::Dimension(SassNumber {
                        num: num + num2,
                        unit,
                        as_slash: None,
                    })
                } else if unit == Unit::None {
                    Value::Dimension(SassNumber {
                        num: num + num2,
                        unit: unit2,
                        as_slash: None,
                    })
                } else if unit2 == Unit::None {
                    Value::Dimension(SassNumber {
                        num: num + num2,
                        unit,
                        as_slash: None,
                    })
                } else {
                    Value::Dimension(SassNumber {
                        num: num + num2.convert(&unit2, &unit),
                        unit,
                        as_slash: None,
                    })
                }
            }
            Value::String(s, q) => Value::String(
                format!("{}{}{}", num.to_string(options.is_compressed()), unit, s),
                q,
            ),
            Value::Null => Value::String(
                format!("{}{}", num.to_string(options.is_compressed()), unit),
                QuoteKind::None,
            ),
            Value::True | Value::False | Value::List(..) | Value::ArgList(..) => Value::String(
                format!(
                    "{}{}{}",
                    num.to_string(options.is_compressed()),
                    unit,
                    right.to_css_string(span, options.is_compressed())?
                ),
                QuoteKind::None,
            ),
            Value::Map(..) | Value::FunctionRef(..) => {
                return Err((
                    format!("{} isn't a valid CSS value.", right.inspect(span)?),
                    span,
                )
                    .into())
            }
            Value::Color(..) | Value::Calculation(..) => {
                return Err((
                    format!(
                        "Undefined operation \"{}{} + {}\".",
                        num.inspect(),
                        unit,
                        right.inspect(span)?
                    ),
                    span,
                )
                    .into())
            }
        },
        c @ Value::Color(..) => match right {
            // todo: we really can't add to any other types?
            Value::String(..) | Value::Null | Value::List(..) => Value::String(
                format!(
                    "{}{}",
                    c.to_css_string(span, options.is_compressed())?,
                    right.to_css_string(span, options.is_compressed())?,
                ),
                QuoteKind::None,
            ),
            _ => {
                return Err((
                    format!(
                        "Undefined operation \"{} + {}\".",
                        c.inspect(span)?,
                        right.inspect(span)?
                    ),
                    span,
                )
                    .into())
            }
        },
        Value::String(text, quotes) => match right {
            Value::String(text2, ..) => Value::String(text + &text2, quotes),
            _ => Value::String(
                text + &right.to_css_string(span, options.is_compressed())?,
                quotes,
            ),
        },
        Value::List(..) | Value::ArgList(..) => match right {
            Value::String(s, q) => Value::String(
                format!(
                    "{}{}",
                    left.to_css_string(span, options.is_compressed())?,
                    s
                ),
                q,
            ),
            _ => Value::String(
                format!(
                    "{}{}",
                    left.to_css_string(span, options.is_compressed())?,
                    right.to_css_string(span, options.is_compressed())?
                ),
                QuoteKind::None,
            ),
        },
    })
}

pub(crate) fn sub(left: Value, right: Value, options: &Options, span: Span) -> SassResult<Value> {
    Ok(match left {
        Value::Calculation(..) => {
            return Err((
                format!(
                    "Undefined operation \"{} - {}\".",
                    left.inspect(span)?,
                    right.inspect(span)?
                ),
                span,
            )
                .into())
        }
        Value::Null => Value::String(
            format!("-{}", right.to_css_string(span, options.is_compressed())?),
            QuoteKind::None,
        ),
        Value::Dimension(SassNumber {
            num,
            unit,
            as_slash: _,
        }) => match right {
            Value::Dimension(SassNumber {
                num: num2,
                unit: unit2,
                as_slash: _,
            }) => {
                if !unit.comparable(&unit2) {
                    return Err(
                        (format!("Incompatible units {} and {}.", unit2, unit), span).into(),
                    );
                }
                if unit == unit2 {
                    Value::Dimension(SassNumber {
                        num: num - num2,
                        unit,
                        as_slash: None,
                    })
                } else if unit == Unit::None {
                    Value::Dimension(SassNumber {
                        num: num - num2,
                        unit: unit2,
                        as_slash: None,
                    })
                } else if unit2 == Unit::None {
                    Value::Dimension(SassNumber {
                        num: num - num2,
                        unit,
                        as_slash: None,
                    })
                } else {
                    Value::Dimension(SassNumber {
                        num: num - num2.convert(&unit2, &unit),
                        unit,
                        as_slash: None,
                    })
                }
            }
            Value::List(..)
            | Value::String(..)
            | Value::True
            | Value::False
            | Value::ArgList(..) => Value::String(
                format!(
                    "{}{}-{}",
                    num.to_string(options.is_compressed()),
                    unit,
                    right.to_css_string(span, options.is_compressed())?
                ),
                QuoteKind::None,
            ),
            Value::Map(..) | Value::FunctionRef(..) => {
                return Err((
                    format!("{} isn't a valid CSS value.", right.inspect(span)?),
                    span,
                )
                    .into())
            }
            Value::Color(..) | Value::Calculation(..) => {
                return Err((
                    format!(
                        "Undefined operation \"{}{} - {}\".",
                        num.inspect(),
                        unit,
                        right.inspect(span)?
                    ),
                    span,
                )
                    .into())
            }
            Value::Null => Value::String(
                format!("{}{}-", num.to_string(options.is_compressed()), unit),
                QuoteKind::None,
            ),
        },
        c @ Value::Color(..) => match right {
            Value::Dimension(SassNumber { .. }) | Value::Color(..) => {
                return Err((
                    format!(
                        "Undefined operation \"{} - {}\".",
                        c.inspect(span)?,
                        right.inspect(span)?
                    ),
                    span,
                )
                    .into())
            }
            _ => Value::String(
                format!(
                    "{}-{}",
                    c.to_css_string(span, options.is_compressed())?,
                    right.to_css_string(span, options.is_compressed())?
                ),
                QuoteKind::None,
            ),
        },
        Value::String(..) => Value::String(
            format!(
                "{}-{}",
                left.to_css_string(span, options.is_compressed())?,
                right.to_css_string(span, options.is_compressed())?
            ),
            QuoteKind::None,
        ),
        // todo: can be greatly simplified
        _ => match right {
            Value::String(s, q) => Value::String(
                format!(
                    "{}-{}{}{}",
                    left.to_css_string(span, options.is_compressed())?,
                    q,
                    s,
                    q
                ),
                QuoteKind::None,
            ),
            Value::Null => Value::String(
                format!("{}-", left.to_css_string(span, options.is_compressed())?),
                QuoteKind::None,
            ),
            _ => Value::String(
                format!(
                    "{}-{}",
                    left.to_css_string(span, options.is_compressed())?,
                    right.to_css_string(span, options.is_compressed())?
                ),
                QuoteKind::None,
            ),
        },
    })
}

pub(crate) fn mul(left: Value, right: Value, _: &Options, span: Span) -> SassResult<Value> {
    Ok(match left {
        Value::Dimension(SassNumber {
            num,
            unit,
            as_slash: _,
        }) => match right {
            Value::Dimension(SassNumber {
                num: num2,
                unit: unit2,
                as_slash: _,
            }) => {
                if unit2 == Unit::None {
                    return Ok(Value::Dimension(SassNumber {
                        num: num * num2,
                        unit,
                        as_slash: None,
                    }));
                }

                let n = SassNumber {
                    num,
                    unit,
                    as_slash: None,
                } * SassNumber {
                    num: num2,
                    unit: unit2,
                    as_slash: None,
                };

                Value::Dimension(n)
            }
            _ => {
                return Err((
                    format!(
                        "Undefined operation \"{}{} * {}\".",
                        num.inspect(),
                        unit,
                        right.inspect(span)?
                    ),
                    span,
                )
                    .into())
            }
        },
        _ => {
            return Err((
                format!(
                    "Undefined operation \"{} * {}\".",
                    left.inspect(span)?,
                    right.inspect(span)?
                ),
                span,
            )
                .into())
        }
    })
}

pub(crate) fn cmp(
    left: &Value,
    right: &Value,
    _: &Options,
    span: Span,
    op: BinaryOp,
) -> SassResult<Value> {
    let ordering = match left.cmp(right, span, op)? {
        Some(ord) => ord,
        None => return Ok(Value::False),
    };

    Ok(match op {
        BinaryOp::GreaterThan => match ordering {
            Ordering::Greater => Value::True,
            Ordering::Less | Ordering::Equal => Value::False,
        },
        BinaryOp::GreaterThanEqual => match ordering {
            Ordering::Greater | Ordering::Equal => Value::True,
            Ordering::Less => Value::False,
        },
        BinaryOp::LessThan => match ordering {
            Ordering::Less => Value::True,
            Ordering::Greater | Ordering::Equal => Value::False,
        },
        BinaryOp::LessThanEqual => match ordering {
            Ordering::Less | Ordering::Equal => Value::True,
            Ordering::Greater => Value::False,
        },
        _ => unreachable!(),
    })
}

pub(crate) fn single_eq(
    left: &Value,
    right: &Value,
    options: &Options,
    span: Span,
) -> SassResult<Value> {
    Ok(Value::String(
        format!(
            "{}={}",
            left.to_css_string(span, options.is_compressed())?,
            right.to_css_string(span, options.is_compressed())?
        ),
        QuoteKind::None,
    ))
}

pub(crate) fn div(left: Value, right: Value, options: &Options, span: Span) -> SassResult<Value> {
    Ok(match (left, right) {
        (Value::Dimension(num1), Value::Dimension(num2)) => {
            if num2.unit == Unit::None {
                return Ok(Value::Dimension(SassNumber {
                    num: num1.num / num2.num,
                    unit: num1.unit,
                    as_slash: None,
                }));
            }

            let n = SassNumber {
                num: num1.num,
                unit: num1.unit,
                as_slash: None,
            } / SassNumber {
                num: num2.num,
                unit: num2.unit,
                as_slash: None,
            };

            Value::Dimension(n)
        }
        (
            left @ Value::Color(..),
            right @ (Value::Dimension(SassNumber { .. }) | Value::Color(..)),
        ) => {
            return Err((
                format!(
                    "Undefined operation \"{} / {}\".",
                    left.inspect(span)?,
                    right.inspect(span)?
                ),
                span,
            )
                .into())
        }
        (left, right) => Value::String(
            format!(
                "{}/{}",
                left.to_css_string(span, options.is_compressed())?,
                right.to_css_string(span, options.is_compressed())?
            ),
            QuoteKind::None,
        ),
    })
}

pub(crate) fn rem(left: Value, right: Value, _: &Options, span: Span) -> SassResult<Value> {
    Ok(match (left, right) {
        (Value::Dimension(num1), Value::Dimension(num2)) => {
            if !num1.unit.comparable(&num2.unit) {
                return Err((
                    format!("Incompatible units {} and {}.", num1.unit, num2.unit),
                    span,
                )
                    .into());
            }

            let new_num = num1.num % num2.num.convert(&num2.unit, &num1.unit);
            let new_unit = if num1.unit == num2.unit {
                num1.unit
            } else if num1.unit == Unit::None {
                num2.unit
            } else {
                num1.unit
            };
            Value::Dimension(SassNumber {
                num: new_num,
                unit: new_unit,
                as_slash: None,
            })
        }
        (left, right) => {
            return Err((
                format!(
                    "Undefined operation \"{} % {}\".",
                    left.inspect(span)?,
                    right.inspect(span)?
                ),
                span,
            )
                .into())
        }
    })
}
