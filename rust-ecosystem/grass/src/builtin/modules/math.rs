use crate::builtin::builtin_imports::*;

use crate::builtin::{
    math::{abs, ceil, comparable, divide, floor, max, min, percentage, round},
    meta::{unit, unitless},
    modules::Module,
};

#[cfg(feature = "random")]
use crate::builtin::math::random;
use crate::value::{conversion_factor, SassNumber};

fn coerce_to_rad(num: f64, unit: Unit) -> f64 {
    debug_assert!(matches!(
        unit,
        Unit::None | Unit::Rad | Unit::Deg | Unit::Grad | Unit::Turn
    ));

    if unit == Unit::None {
        return num;
    }

    let factor = conversion_factor(&unit, &Unit::Rad).unwrap();

    num * factor
}

fn clamp(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(3)?;
    let span = args.span();

    let min = match args.get_err(0, "min")? {
        v @ Value::Dimension(SassNumber { .. }) => v,
        v => {
            return Err((
                format!("$min: {} is not a number.", v.inspect(args.span())?),
                span,
            )
                .into())
        }
    };

    let number = match args.get_err(1, "number")? {
        v @ Value::Dimension(SassNumber { .. }) => v,
        v => {
            return Err((
                format!("$number: {} is not a number.", v.inspect(span)?),
                span,
            )
                .into())
        }
    };

    let max = match args.get_err(2, "max")? {
        v @ Value::Dimension(SassNumber { .. }) => v,
        v => return Err((format!("$max: {} is not a number.", v.inspect(span)?), span).into()),
    };

    // ensure that `min` and `max` are compatible
    min.cmp(&max, span, BinaryOp::LessThan)?;

    let min_unit = match min {
        Value::Dimension(SassNumber {
            num: _,
            unit: ref u,
            as_slash: _,
        }) => u,
        _ => unreachable!(),
    };
    let number_unit = match number {
        Value::Dimension(SassNumber {
            num: _,
            unit: ref u,
            as_slash: _,
        }) => u,
        _ => unreachable!(),
    };
    let max_unit = match max {
        Value::Dimension(SassNumber {
            num: _,
            unit: ref u,
            as_slash: _,
        }) => u,
        _ => unreachable!(),
    };

    if min_unit == &Unit::None && number_unit != &Unit::None {
        return Err((
            format!(
                "$min is unitless but $number has unit {}. Arguments must all have units or all be unitless.",
                number_unit
            ), span).into());
    } else if min_unit != &Unit::None && number_unit == &Unit::None {
        return Err((
                format!(
                    "$min has unit {} but $number is unitless. Arguments must all have units or all be unitless.",
                    min_unit
                ), span).into());
    } else if min_unit != &Unit::None && max_unit == &Unit::None {
        return Err((
            format!(
                "$min has unit {} but $max is unitless. Arguments must all have units or all be unitless.",
                min_unit
            ), span).into());
    }

    match min.cmp(&number, span, BinaryOp::LessThan)? {
        Some(Ordering::Greater) => return Ok(min),
        Some(Ordering::Equal) => return Ok(number),
        Some(Ordering::Less) | None => {}
    }

    match max.cmp(&number, span, BinaryOp::GreaterThan)? {
        Some(Ordering::Less) => return Ok(max),
        Some(Ordering::Equal) => return Ok(number),
        Some(Ordering::Greater) | None => {}
    }

    Ok(number)
}

fn hypot(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.min_args(1)?;

    let span = args.span();

    let mut numbers = args.get_variadic()?.into_iter().map(|v| -> SassResult<_> {
        match v.node {
            Value::Dimension(SassNumber { num, unit, .. }) => Ok((num, unit)),
            v => Err((format!("{} is not a number.", v.inspect(span)?), span).into()),
        }
    });

    let (n, u) = numbers.next().unwrap()?;
    let first: (Number, Unit) = (n * n, u);

    let rest = numbers
        .enumerate()
        .map(|(idx, val)| -> SassResult<Number> {
            let (number, unit) = val?;
            if first.1 == Unit::None {
                if unit == Unit::None {
                    Ok(number * number)
                } else {
                    Err((
                        format!(
                            "Argument 1 is unitless but argument {} has unit {}. \
                            Arguments must all have units or all be unitless.",
                            idx + 2,
                            unit
                        ),
                        span,
                    )
                        .into())
                }
            } else if unit == Unit::None {
                Err((
                    format!(
                        "Argument 1 has unit {} but argument {} is unitless. \
                        Arguments must all have units or all be unitless.",
                        first.1,
                        idx + 2,
                    ),
                    span,
                )
                    .into())
            } else if first.1.comparable(&unit) {
                let n = number.convert(&unit, &first.1);
                Ok(n * n)
            } else {
                Err((
                    format!("Incompatible units {} and {}.", first.1, unit),
                    span,
                )
                    .into())
            }
        })
        .collect::<SassResult<Vec<Number>>>()?;

    let sum = first.0 + rest.into_iter().fold(Number::zero(), |a, b| a + b);

    Ok(Value::Dimension(SassNumber {
        num: sum.sqrt(),
        unit: first.1,
        as_slash: None,
    }))
}

fn log(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;

    let span = args.span();

    let number = args
        .get_err(0, "number")?
        .assert_number_with_name("number", span)?;
    number.assert_no_units("number", span)?;
    let number = number.num;

    let base = match args.default_arg(1, "base", Value::Null) {
        Value::Null => None,
        v => {
            let base = v.assert_number_with_name("base", span)?;
            base.assert_no_units("base", span)?;
            Some(base.num)
        }
    };

    Ok(Value::Dimension(SassNumber::new_unitless(
        if let Some(base) = base {
            if base.is_zero() {
                Number::zero()
            } else {
                number.log(base)
            }
        // todo: test with negative 0
        } else if number.is_negative() && !number.is_zero() {
            Number(f64::NAN)
        } else if number.is_zero() {
            Number(f64::NEG_INFINITY)
        } else {
            number.ln()
        },
    )))
}

fn pow(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;

    let span = args.span();

    let base = args
        .get_err(0, "base")?
        .assert_number_with_name("base", span)?;
    base.assert_no_units("base", span)?;

    let exponent = args
        .get_err(1, "exponent")?
        .assert_number_with_name("exponent", span)?;
    exponent.assert_no_units("exponent", span)?;

    Ok(Value::Dimension(SassNumber::new_unitless(
        base.num.pow(exponent.num),
    )))
}

fn sqrt(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let number = args
        .get_err(0, "number")?
        .assert_number_with_name("number", args.span())?;
    number.assert_no_units("number", args.span())?;

    Ok(Value::Dimension(SassNumber::new_unitless(
        number.num.sqrt(),
    )))
}

macro_rules! trig_fn {
    ($name:ident) => {
        fn $name(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
            args.max_args(1)?;
            let number = args.get_err(0, "number")?;

            Ok(match number {
                Value::Dimension(SassNumber {
                    num,
                    unit: unit @ (Unit::None | Unit::Rad | Unit::Deg | Unit::Grad | Unit::Turn),
                    ..
                }) => {
                    Value::Dimension(SassNumber::new_unitless(coerce_to_rad(num.0, unit).$name()))
                }
                v @ Value::Dimension(..) => {
                    return Err((
                        format!(
                            "$number: Expected {} to have an angle unit (deg, grad, rad, turn).",
                            v.inspect(args.span())?
                        ),
                        args.span(),
                    )
                        .into())
                }
                v => {
                    return Err((
                        format!("$number: {} is not a number.", v.inspect(args.span())?),
                        args.span(),
                    )
                        .into())
                }
            })
        }
    };
}

trig_fn!(cos);
trig_fn!(sin);
trig_fn!(tan);

fn acos(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let span = args.span();

    let number = args
        .get_err(0, "number")?
        .assert_number_with_name("number", span)?;
    number.assert_no_units("number", span)?;
    let number = number.num;

    Ok(Value::Dimension(SassNumber {
        num: if number > Number(1.0) || number < Number(-1.0) {
            Number(f64::NAN)
        } else if number.is_one() {
            Number::zero()
        } else {
            number.acos()
        },
        unit: Unit::Deg,
        as_slash: None,
    }))
}

fn asin(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let span = args.span();

    let number = args
        .get_err(0, "number")?
        .assert_number_with_name("number", span)?;
    number.assert_no_units("number", span)?;
    let number = number.num;

    if number > Number(1.0) || number < Number(-1.0) {
        return Ok(Value::Dimension(SassNumber {
            num: Number(f64::NAN),
            unit: Unit::Deg,
            as_slash: None,
        }));
    } else if number.is_zero() {
        return Ok(Value::Dimension(SassNumber {
            num: Number::zero(),
            unit: Unit::Deg,
            as_slash: None,
        }));
    }

    Ok(Value::Dimension(SassNumber {
        num: number.asin(),
        unit: Unit::Deg,
        as_slash: None,
    }))
}

fn atan(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let span = args.span();

    let number = args
        .get_err(0, "number")?
        .assert_number_with_name("number", span)?;
    number.assert_no_units("number", span)?;

    if number.num.is_zero() {
        return Ok(Value::Dimension(SassNumber {
            num: (Number::zero()),
            unit: Unit::Deg,
            as_slash: None,
        }));
    }

    Ok(Value::Dimension(SassNumber {
        num: number.num.atan(),
        unit: Unit::Deg,
        as_slash: None,
    }))
}

fn atan2(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let (y_num, y_unit) = match args.get_err(0, "y")? {
        Value::Dimension(SassNumber {
            num: n, unit: u, ..
        }) => (n, u),
        v => {
            return Err((
                format!("$y: {} is not a number.", v.inspect(args.span())?),
                args.span(),
            )
                .into())
        }
    };

    let (x_num, x_unit) = match args.get_err(1, "x")? {
        Value::Dimension(SassNumber {
            num: n, unit: u, ..
        }) => (n, u),
        v => {
            return Err((
                format!("$x: {} is not a number.", v.inspect(args.span())?),
                args.span(),
            )
                .into())
        }
    };

    let (x_num, y_num) = if x_unit == Unit::None && y_unit == Unit::None {
        (x_num, y_num)
    } else if y_unit == Unit::None {
        return Err((
            format!(
                "$y is unitless but $x has unit {}. \
            Arguments must all have units or all be unitless.",
                x_unit
            ),
            args.span(),
        )
            .into());
    } else if x_unit == Unit::None {
        return Err((
            format!(
                "$y has unit {} but $x is unitless. \
                Arguments must all have units or all be unitless.",
                y_unit
            ),
            args.span(),
        )
            .into());
    } else if x_unit.comparable(&y_unit) {
        (x_num, y_num.convert(&y_unit, &x_unit))
    } else {
        return Err((
            format!("Incompatible units {} and {}.", y_unit, x_unit),
            args.span(),
        )
            .into());
    };

    Ok(Value::Dimension(SassNumber {
        num: Number(y_num.0.atan2(x_num.0).to_degrees()),
        unit: Unit::Deg,
        as_slash: None,
    }))
}

pub(crate) fn declare(f: &mut Module) {
    f.insert_builtin("ceil", ceil);
    f.insert_builtin("floor", floor);
    f.insert_builtin("max", max);
    f.insert_builtin("min", min);
    f.insert_builtin("round", round);
    f.insert_builtin("abs", abs);
    f.insert_builtin("compatible", comparable);
    f.insert_builtin("is-unitless", unitless);
    f.insert_builtin("unit", unit);
    f.insert_builtin("percentage", percentage);
    f.insert_builtin("clamp", clamp);
    f.insert_builtin("sqrt", sqrt);
    f.insert_builtin("cos", cos);
    f.insert_builtin("sin", sin);
    f.insert_builtin("tan", tan);
    f.insert_builtin("acos", acos);
    f.insert_builtin("asin", asin);
    f.insert_builtin("atan", atan);
    f.insert_builtin("log", log);
    f.insert_builtin("pow", pow);
    f.insert_builtin("hypot", hypot);
    f.insert_builtin("div", divide);
    f.insert_builtin("atan2", atan2);
    #[cfg(feature = "random")]
    f.insert_builtin("random", random);

    f.insert_builtin_var(
        "e",
        Value::Dimension(SassNumber::new_unitless(std::f64::consts::E)),
    );
    f.insert_builtin_var(
        "pi",
        Value::Dimension(SassNumber::new_unitless(std::f64::consts::PI)),
    );
    f.insert_builtin_var(
        "epsilon",
        Value::Dimension(SassNumber::new_unitless(f64::EPSILON)),
    );
    f.insert_builtin_var(
        "max-safe-integer",
        Value::Dimension(SassNumber::new_unitless(9007199254740991.0)),
    );
    f.insert_builtin_var(
        "min-safe-integer",
        Value::Dimension(SassNumber::new_unitless(-9007199254740991.0)),
    );
    f.insert_builtin_var(
        "max-number",
        Value::Dimension(SassNumber::new_unitless(f64::MAX)),
    );
    f.insert_builtin_var(
        "min-number",
        Value::Dimension(SassNumber::new_unitless(f64::MIN_POSITIVE)),
    );
}
