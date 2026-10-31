use crate::{builtin::builtin_imports::*, evaluate::div};

pub(crate) fn percentage(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let num = args
        .get_err(0, "number")?
        .assert_number_with_name("number", args.span)?;
    num.assert_no_units("number", args.span)?;

    Ok(Value::Dimension(SassNumber {
        num: Number(num.num.0 * 100.0),
        unit: Unit::Percent,
        as_slash: None,
    }))
}

pub(crate) fn round(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let mut number = args
        .get_err(0, "number")?
        .assert_number_with_name("number", args.span())?;

    if !number.num.is_finite() {
        return Err(("Infinity or NaN toInt", args.span()).into());
    }

    number.num = number.num.round();

    Ok(Value::Dimension(number))
}

pub(crate) fn ceil(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let mut number = args
        .get_err(0, "number")?
        .assert_number_with_name("number", args.span())?;

    if !number.num.is_finite() {
        return Err(("Infinity or NaN toInt", args.span()).into());
    }

    number.num = number.num.ceil();

    Ok(Value::Dimension(number))
}

pub(crate) fn floor(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let mut number = args
        .get_err(0, "number")?
        .assert_number_with_name("number", args.span())?;

    if !number.num.is_finite() {
        return Err(("Infinity or NaN toInt", args.span()).into());
    }

    number.num = number.num.floor();

    Ok(Value::Dimension(number))
}

pub(crate) fn abs(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let mut num = args
        .get_err(0, "number")?
        .assert_number_with_name("number", args.span())?;

    num.num = num.num.abs();

    Ok(Value::Dimension(num))
}

pub(crate) fn comparable(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let unit1 = args
        .get_err(0, "number1")?
        .assert_number_with_name("number1", args.span())?
        .unit;

    let unit2 = args
        .get_err(1, "number2")?
        .assert_number_with_name("number2", args.span())?
        .unit;

    Ok(Value::bool(unit1.comparable(&unit2)))
}

#[cfg(feature = "random")]
pub(crate) fn random(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let limit = args.default_arg(0, "limit", Value::Null);

    if matches!(limit, Value::Null) {
        let mut rng = rand::thread_rng();
        return Ok(Value::Dimension(SassNumber::new_unitless(
            rng.gen_range(0.0..1.0),
        )));
    }

    let limit = limit.assert_number_with_name("limit", args.span())?;
    let limit_int = limit.assert_int_with_name("limit", args.span())?;
    let limit = limit.num;

    if limit.is_one() {
        return Ok(Value::Dimension(SassNumber::new_unitless(1.0)));
    }

    if limit.is_zero() || limit.is_negative() {
        return Err((
            format!("$limit: Must be greater than 0, was {}.", limit.inspect()),
            args.span(),
        )
            .into());
    }

    let mut rng = rand::thread_rng();
    Ok(Value::Dimension(SassNumber::new_unitless(
        rng.gen_range(0..limit_int) + 1,
    )))
}

pub(crate) fn min(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.min_args(1)?;
    let span = args.span();
    let mut nums = args
        .get_variadic()?
        .into_iter()
        .map(|val| match val.node {
            Value::Dimension(SassNumber {
                num: number,
                unit,
                as_slash: _,
            }) => Ok((number, unit)),
            v => Err((format!("{} is not a number.", v.inspect(span)?), span).into()),
        })
        .collect::<SassResult<Vec<(Number, Unit)>>>()?
        .into_iter();

    let mut min = match nums.next() {
        Some((n, u)) => (n, u),
        None => unreachable!(),
    };

    for (num, unit) in nums {
        let lhs = Value::Dimension(SassNumber {
            num,
            unit: unit.clone(),
            as_slash: None,
        });
        let rhs = Value::Dimension(SassNumber {
            num: (min.0),
            unit: min.1.clone(),
            as_slash: None,
        });

        if crate::evaluate::cmp(&lhs, &rhs, visitor.options, span, BinaryOp::LessThan)?.is_truthy()
        {
            min = (num, unit);
        }
    }
    Ok(Value::Dimension(SassNumber {
        num: (min.0),
        unit: min.1,
        as_slash: None,
    }))
}

pub(crate) fn max(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.min_args(1)?;
    let span = args.span();
    let mut nums = args
        .get_variadic()?
        .into_iter()
        .map(|val| match val.node {
            Value::Dimension(SassNumber {
                num: number,
                unit,
                as_slash: _,
            }) => Ok((number, unit)),
            v => Err((format!("{} is not a number.", v.inspect(span)?), span).into()),
        })
        .collect::<SassResult<Vec<(Number, Unit)>>>()?
        .into_iter();

    let mut max = match nums.next() {
        Some((n, u)) => (n, u),
        None => unreachable!(),
    };

    for (num, unit) in nums {
        let lhs = Value::Dimension(SassNumber {
            num,
            unit: unit.clone(),
            as_slash: None,
        });
        let rhs = Value::Dimension(SassNumber {
            num: (max.0),
            unit: max.1.clone(),
            as_slash: None,
        });

        if crate::evaluate::cmp(&lhs, &rhs, visitor.options, span, BinaryOp::GreaterThan)?
            .is_truthy()
        {
            max = (num, unit);
        }
    }
    Ok(Value::Dimension(SassNumber {
        num: (max.0),
        unit: max.1,
        as_slash: None,
    }))
}

pub(crate) fn divide(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;

    let number1 = args.get_err(0, "number1")?;
    let number2 = args.get_err(1, "number2")?;

    div(number1, number2, visitor.options, args.span())
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("percentage", Builtin::new(percentage));
    f.insert("round", Builtin::new(round));
    f.insert("ceil", Builtin::new(ceil));
    f.insert("floor", Builtin::new(floor));
    f.insert("abs", Builtin::new(abs));
    f.insert("min", Builtin::new(min));
    f.insert("max", Builtin::new(max));
    f.insert("comparable", Builtin::new(comparable));
    #[cfg(feature = "random")]
    f.insert("random", Builtin::new(random));
}
