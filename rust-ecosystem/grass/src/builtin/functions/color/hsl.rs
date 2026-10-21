use std::collections::{BTreeMap, BTreeSet};

use crate::{builtin::builtin_imports::*, serializer::serialize_number, value::SassNumber};

use super::{
    angle_value,
    rgb::{function_string, parse_channels, percentage_or_unitless},
    ParsedChannels,
};

fn hsl_3_args(
    name: &'static str,
    mut args: ArgumentResult,
    visitor: &mut Visitor,
) -> SassResult<Value> {
    let span = args.span();

    let hue = args.get_err(0, "hue")?;
    let saturation = args.get_err(1, "saturation")?;
    let lightness = args.get_err(2, "lightness")?;
    let alpha = args.default_arg(3, "alpha", Value::Dimension(SassNumber::new_unitless(1.0)));

    if [&hue, &saturation, &lightness, &alpha]
        .iter()
        .copied()
        .any(Value::is_special_function)
    {
        return Ok(Value::String(
            format!(
                "{}({})",
                name,
                Value::List(
                    if args.len() == 4 {
                        vec![hue, saturation, lightness, alpha]
                    } else {
                        vec![hue, saturation, lightness]
                    },
                    ListSeparator::Comma,
                    Brackets::None
                )
                .to_css_string(args.span(), false)?
            ),
            QuoteKind::None,
        ));
    }

    let hue = angle_value(hue, "hue", span)?;
    let saturation = saturation.assert_number_with_name("saturation", span)?;
    let lightness = lightness.assert_number_with_name("lightness", span)?;
    let alpha = percentage_or_unitless(
        &alpha.assert_number_with_name("alpha", span)?,
        1.0,
        "alpha",
        span,
        visitor,
    )?;

    Ok(Value::Color(Arc::new(Color::from_hsla_fn(
        Number(hue.rem_euclid(360.0)),
        saturation.num / Number(100.0),
        lightness.num / Number(100.0),
        Number(alpha),
    ))))
}

fn inner_hsl(
    name: &'static str,
    mut args: ArgumentResult,
    visitor: &mut Visitor,
) -> SassResult<Value> {
    args.max_args(4)?;
    let span = args.span();

    let len = args.len();

    if len == 1 || len == 0 {
        match parse_channels(
            name,
            &["hue", "saturation", "lightness"],
            args.get_err(0, "channels")?,
            visitor,
            args.span(),
        )? {
            ParsedChannels::String(s) => Ok(Value::String(s, QuoteKind::None)),
            ParsedChannels::List(list) => {
                let args = ArgumentResult {
                    positional: list,
                    named: BTreeMap::new(),
                    separator: ListSeparator::Comma,
                    span: args.span(),
                    touched: BTreeSet::new(),
                };

                hsl_3_args(name, args, visitor)
            }
        }
    } else if len == 2 {
        let hue = args.get_err(0, "hue")?;
        let saturation = args.get_err(1, "saturation")?;

        if hue.is_var() || saturation.is_var() {
            Ok(Value::String(
                function_string(name, &[hue, saturation], visitor, span)?,
                QuoteKind::None,
            ))
        } else {
            Err(("Missing argument $lightness.", args.span()).into())
        }
    } else {
        hsl_3_args(name, args, visitor)
    }
}

pub(crate) fn hsl(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    inner_hsl("hsl", args, visitor)
}

pub(crate) fn hsla(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    inner_hsl("hsla", args, visitor)
}

pub(crate) fn hue(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::Dimension(SassNumber {
        num: color.hue(),
        unit: Unit::Deg,
        as_slash: None,
    }))
}

pub(crate) fn saturation(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::Dimension(SassNumber {
        num: color.saturation(),
        unit: Unit::Percent,
        as_slash: None,
    }))
}

pub(crate) fn lightness(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::Dimension(SassNumber {
        num: color.lightness(),
        unit: Unit::Percent,
        as_slash: None,
    }))
}

pub(crate) fn adjust_hue(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;
    let degrees = angle_value(args.get_err(1, "degrees")?, "degrees", args.span())?;

    Ok(Value::Color(Arc::new(color.adjust_hue(degrees))))
}

fn lighten(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    let mut amount = args
        .get_err(1, "amount")?
        .assert_number_with_name("amount", args.span())?;

    amount.assert_bounds("amount", 0.0, 100.0, args.span())?;

    amount.num /= Number(100.0);

    Ok(Value::Color(Arc::new(color.lighten(amount.num))))
}

fn darken(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    let mut amount = args
        .get_err(1, "amount")?
        .assert_number_with_name("amount", args.span())?;

    amount.assert_bounds("amount", 0.0, 100.0, args.span())?;

    amount.num /= Number(100.0);

    Ok(Value::Color(Arc::new(color.darken(amount.num))))
}

fn saturate(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    if args.len() == 1 {
        let amount = args
            .get_err(0, "amount")?
            .assert_number_with_name("amount", args.span())?;

        return Ok(Value::String(
            format!(
                "saturate({})",
                serialize_number(&amount, &Options::default(), args.span())?,
            ),
            QuoteKind::None,
        ));
    }

    let mut amount = args
        .get_err(1, "amount")?
        .assert_number_with_name("amount", args.span())?;

    amount.assert_bounds("amount", 0.0, 100.0, args.span())?;

    amount.num /= Number(100.0);

    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::Color(Arc::new(color.saturate(amount.num))))
}

fn desaturate(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    let mut amount = args
        .get_err(1, "amount")?
        .assert_number_with_name("amount", args.span())?;

    amount.assert_bounds("amount", 0.0, 100.0, args.span())?;

    amount.num /= Number(100.0);

    Ok(Value::Color(Arc::new(color.desaturate(amount.num))))
}

pub(crate) fn grayscale(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = match args.get_err(0, "color")? {
        Value::Color(c) => c,
        Value::Dimension(SassNumber {
            num: n,
            unit: u,
            as_slash: _,
        }) => {
            return Ok(Value::String(
                format!("grayscale({}{})", n.inspect(), u),
                QuoteKind::None,
            ))
        }
        v => {
            return Err((
                format!("$color: {} is not a color.", v.inspect(args.span())?),
                args.span(),
            )
                .into())
        }
    };
    Ok(Value::Color(Arc::new(color.desaturate(Number::one()))))
}

pub(crate) fn complement(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;
    Ok(Value::Color(Arc::new(color.complement())))
}

pub(crate) fn invert(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let weight = args
        .get(1, "weight")
        .map::<SassResult<_>, _>(|weight| {
            let mut weight = weight.node.assert_number_with_name("weight", span)?;

            weight.assert_bounds("weight", 0.0, 100.0, span)?;

            weight.num /= Number(100.0);

            Ok(weight.num)
        })
        .transpose()?;

    match args.get_err(0, "color")? {
        Value::Color(c) => Ok(Value::Color(Arc::new(
            c.invert(weight.unwrap_or_else(Number::one)),
        ))),
        Value::Dimension(SassNumber {
            num: n,
            unit: u,
            as_slash: _,
        }) => {
            if weight.is_some() {
                return Err((
                    "Only one argument may be passed to the plain-CSS invert() function.",
                    args.span(),
                )
                    .into());
            }
            Ok(Value::String(
                format!("invert({}{})", n.inspect(), u),
                QuoteKind::None,
            ))
        }
        v => Err((
            format!("$color: {} is not a color.", v.inspect(args.span())?),
            args.span(),
        )
            .into()),
    }
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("hsl", Builtin::new(hsl));
    f.insert("hsla", Builtin::new(hsla));
    f.insert("hue", Builtin::new(hue));
    f.insert("saturation", Builtin::new(saturation));
    f.insert("adjust-hue", Builtin::new(adjust_hue));
    f.insert("lightness", Builtin::new(lightness));
    f.insert("lighten", Builtin::new(lighten));
    f.insert("darken", Builtin::new(darken));
    f.insert("saturate", Builtin::new(saturate));
    f.insert("desaturate", Builtin::new(desaturate));
    f.insert("grayscale", Builtin::new(grayscale));
    f.insert("complement", Builtin::new(complement));
    f.insert("invert", Builtin::new(invert));
}
