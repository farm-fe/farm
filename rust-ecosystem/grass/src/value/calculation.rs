use core::fmt;
use std::iter::Iterator;

use codemap::Span;

use crate::{
    common::BinaryOp,
    error::SassResult,
    serializer::inspect_number,
    unit::Unit,
    value::{SassNumber, Value},
    Options,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalculationArg {
    Number(SassNumber),
    Calculation(SassCalculation),
    String(String),
    Operation {
        lhs: Box<Self>,
        op: BinaryOp,
        rhs: Box<Self>,
    },
    Interpolation(String),
}

impl CalculationArg {
    pub fn parenthesize_calculation_rhs(outer: BinaryOp, right: BinaryOp) -> bool {
        if outer == BinaryOp::Div {
            true
        } else if outer == BinaryOp::Plus {
            false
        } else {
            right == BinaryOp::Plus || right == BinaryOp::Minus
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalculationName {
    Calc,
    Min,
    Max,
    Clamp,
}

impl fmt::Display for CalculationName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalculationName::Calc => f.write_str("calc"),
            CalculationName::Min => f.write_str("min"),
            CalculationName::Max => f.write_str("max"),
            CalculationName::Clamp => f.write_str("clamp"),
        }
    }
}

impl CalculationName {
    pub(crate) fn in_min_or_max(self) -> bool {
        self == CalculationName::Min || self == CalculationName::Max
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SassCalculation {
    pub name: CalculationName,
    pub args: Vec<CalculationArg>,
}

impl SassCalculation {
    pub fn unsimplified(name: CalculationName, args: Vec<CalculationArg>) -> Self {
        Self { name, args }
    }

    pub fn calc(arg: CalculationArg) -> Value {
        let arg = Self::simplify(arg);
        match arg {
            CalculationArg::Number(n) => Value::Dimension(n),
            CalculationArg::Calculation(c) => Value::Calculation(c),
            _ => Value::Calculation(SassCalculation {
                name: CalculationName::Calc,
                args: vec![arg],
            }),
        }
    }

    pub fn min(args: Vec<CalculationArg>, options: &Options, span: Span) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        debug_assert!(!args.is_empty(), "min() must have at least one argument.");

        let mut minimum: Option<SassNumber> = None;

        for arg in &args {
            match arg {
                CalculationArg::Number(n)
                    if minimum.is_some() && !minimum.as_ref().unwrap().is_comparable_to(n) =>
                {
                    minimum = None;
                    break;
                }
                CalculationArg::Number(n)
                    if minimum.is_none()
                        || minimum.as_ref().unwrap().num
                            > n.num.convert(&n.unit, &minimum.as_ref().unwrap().unit) =>
                {
                    minimum = Some(n.clone());
                }
                CalculationArg::Number(..) => continue,
                _ => {
                    minimum = None;
                    break;
                }
            }
        }

        Ok(match minimum {
            Some(min) => Value::Dimension(min),
            None => {
                Self::verify_compatible_numbers(&args, options, span)?;

                Value::Calculation(SassCalculation {
                    name: CalculationName::Min,
                    args,
                })
            }
        })
    }

    pub fn max(args: Vec<CalculationArg>, options: &Options, span: Span) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        if args.is_empty() {
            return Err(("max() must have at least one argument.", span).into());
        }

        let mut maximum: Option<SassNumber> = None;

        for arg in &args {
            match arg {
                CalculationArg::Number(n)
                    if maximum.is_some() && !maximum.as_ref().unwrap().is_comparable_to(n) =>
                {
                    maximum = None;
                    break;
                }
                CalculationArg::Number(n)
                    if maximum.is_none()
                        || maximum.as_ref().unwrap().num
                            < n.num.convert(&n.unit, &maximum.as_ref().unwrap().unit) =>
                {
                    maximum = Some(n.clone());
                }
                CalculationArg::Number(..) => continue,
                _ => {
                    maximum = None;
                    break;
                }
            }
        }

        Ok(match maximum {
            Some(max) => Value::Dimension(max),
            None => {
                Self::verify_compatible_numbers(&args, options, span)?;

                Value::Calculation(SassCalculation {
                    name: CalculationName::Max,
                    args,
                })
            }
        })
    }

    pub fn clamp(
        min: CalculationArg,
        value: Option<CalculationArg>,
        max: Option<CalculationArg>,
        options: &Options,
        span: Span,
    ) -> SassResult<Value> {
        if value.is_none() && max.is_some() {
            return Err(("If value is null, max must also be null.", span).into());
        }

        let min = Self::simplify(min);
        let value = value.map(Self::simplify);
        let max = max.map(Self::simplify);

        match (min.clone(), value.clone(), max.clone()) {
            (
                CalculationArg::Number(min),
                Some(CalculationArg::Number(value)),
                Some(CalculationArg::Number(max)),
            ) => {
                if min.is_comparable_to(&value) && min.is_comparable_to(&max) {
                    if value.num <= min.num.convert(min.unit(), value.unit()) {
                        return Ok(Value::Dimension(min));
                    }

                    if value.num >= max.num.convert(max.unit(), value.unit()) {
                        return Ok(Value::Dimension(max));
                    }

                    return Ok(Value::Dimension(value));
                }
            }
            _ => {}
        }

        let mut args = vec![min];

        if let Some(value) = value {
            args.push(value);
        }

        if let Some(max) = max {
            args.push(max);
        }

        Self::verify_length(&args, 3, span)?;
        Self::verify_compatible_numbers(&args, options, span)?;

        Ok(Value::Calculation(SassCalculation {
            name: CalculationName::Clamp,
            args,
        }))
    }

    fn verify_length(args: &[CalculationArg], len: usize, span: Span) -> SassResult<()> {
        if args.len() == len {
            return Ok(());
        }

        if args.iter().any(|arg| {
            matches!(
                arg,
                CalculationArg::String(..) | CalculationArg::Interpolation(..)
            )
        }) {
            return Ok(());
        }

        let was_or_were = if args.len() == 1 { "was" } else { "were" };

        Err((
            format!(
                "{len} arguments required, but only {} {was_or_were} passed.",
                args.len(),
                len = len,
                was_or_were = was_or_were,
            ),
            span,
        )
            .into())
    }

    #[allow(clippy::needless_range_loop)]
    fn verify_compatible_numbers(
        args: &[CalculationArg],
        options: &Options,
        span: Span,
    ) -> SassResult<()> {
        for arg in args {
            match arg {
                CalculationArg::Number(num) => match &num.unit {
                    Unit::Complex(complex) => {
                        if complex.numer.len() > 1 || !complex.denom.is_empty() {
                            let num = num.clone();
                            let value = Value::Dimension(num);
                            return Err((
                                format!(
                                    "Number {} isn't compatible with CSS calculations.",
                                    value.inspect(span)?
                                ),
                                span,
                            )
                                .into());
                        }
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        for i in 0..args.len() {
            let number1 = match &args[i] {
                CalculationArg::Number(num) => num,
                _ => continue,
            };

            for j in (i + 1)..args.len() {
                let number2 = match &args[j] {
                    CalculationArg::Number(num) => num,
                    _ => continue,
                };

                if number1.has_possibly_compatible_units(number2) {
                    continue;
                }

                return Err((
                    format!(
                        "{} and {} are incompatible.",
                        inspect_number(number1, options, span)?,
                        inspect_number(number2, options, span)?
                    ),
                    span,
                )
                    .into());
            }
        }

        Ok(())
    }

    pub fn operate_internal(
        mut op: BinaryOp,
        left: CalculationArg,
        right: CalculationArg,
        in_min_or_max: bool,
        simplify: bool,
        options: &Options,
        span: Span,
    ) -> SassResult<CalculationArg> {
        if !simplify {
            return Ok(CalculationArg::Operation {
                lhs: Box::new(left),
                op,
                rhs: Box::new(right),
            });
        }

        let left = Self::simplify(left);
        let mut right = Self::simplify(right);

        if op == BinaryOp::Plus || op == BinaryOp::Minus {
            match (&left, &right) {
                (CalculationArg::Number(left), CalculationArg::Number(right))
                    if if in_min_or_max {
                        left.is_comparable_to(right)
                    } else {
                        left.has_compatible_units(&right.unit)
                    } =>
                {
                    if op == BinaryOp::Plus {
                        return Ok(CalculationArg::Number(left.clone() + right.clone()));
                    } else {
                        return Ok(CalculationArg::Number(left.clone() - right.clone()));
                    }
                }
                _ => {}
            }

            Self::verify_compatible_numbers(&[left.clone(), right.clone()], options, span)?;

            if let CalculationArg::Number(mut n) = right {
                if n.num.is_negative() {
                    n.num.0 *= -1.0;
                    op = if op == BinaryOp::Plus {
                        BinaryOp::Minus
                    } else {
                        BinaryOp::Plus
                    }
                } else {
                    // todo: do we need this branch?
                }
                right = CalculationArg::Number(n);
            }

            return Ok(CalculationArg::Operation {
                lhs: Box::new(left),
                op,
                rhs: Box::new(right),
            });
        }

        match (left, right) {
            (CalculationArg::Number(num1), CalculationArg::Number(num2)) => {
                if op == BinaryOp::Mul {
                    Ok(CalculationArg::Number(num1 * num2))
                } else {
                    Ok(CalculationArg::Number(num1 / num2))
                }
            }
            (left, right) => Ok(CalculationArg::Operation {
                lhs: Box::new(left),
                op,
                rhs: Box::new(right),
            }),
        }

        //   _verifyCompatibleNumbers([left, right]);

        // Ok(CalculationArg::Operation {
        //     lhs: Box::new(left),
        //     op,
        //     rhs: Box::new(right),
        // })
    }

    fn simplify(arg: CalculationArg) -> CalculationArg {
        match arg {
            CalculationArg::Number(..)
            | CalculationArg::Operation { .. }
            | CalculationArg::Interpolation(..)
            | CalculationArg::String(..) => arg,
            CalculationArg::Calculation(mut calc) => {
                if calc.name == CalculationName::Calc {
                    calc.args.remove(0)
                } else {
                    CalculationArg::Calculation(calc)
                }
            }
        }
    }

    fn simplify_arguments(args: Vec<CalculationArg>) -> Vec<CalculationArg> {
        args.into_iter().map(Self::simplify).collect()
    }
}
