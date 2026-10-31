use crate::{
    builtin::{builtin_imports::*, color::angle_value},
    utils::to_sentence,
    value::fuzzy_round,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum UpdateComponents {
    Change,
    Adjust,
    Scale,
}

fn update_components(
    mut args: ArgumentResult,
    visitor: &mut Visitor,
    update: UpdateComponents,
) -> SassResult<Value> {
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    // todo: what if color is also passed by name
    if args.positional.len() > 1 {
        return Err((
            "Only one positional argument is allowed. All other arguments must be passed by name.",
            span,
        )
            .into());
    }

    let check_num = |num: Spanned<Value>,
                     name: &str,
                     mut max: f64,
                     assert_percent: bool,
                     check_percent: bool|
     -> SassResult<Number> {
        let span = num.span;
        let mut num = num.node.assert_number_with_name(name, span)?;

        if update == UpdateComponents::Scale {
            max = 100.0;
        }

        if assert_percent || update == UpdateComponents::Scale {
            num.assert_unit(&Unit::Percent, name, span)?;
            num.assert_bounds(
                name,
                if update == UpdateComponents::Change {
                    0.0
                } else {
                    -max
                },
                max,
                span,
            )?;
        } else {
            num.assert_bounds_with_unit(
                name,
                if update == UpdateComponents::Change {
                    0.0
                } else {
                    -max
                },
                max,
                if check_percent {
                    &Unit::Percent
                } else {
                    &Unit::None
                },
                span,
            )?;
        }

        // todo: hack to check if rgb channel
        if max == 100.0 {
            num.num /= Number(100.0);
        }

        Ok(num.num)
    };

    let get_arg = |args: &mut ArgumentResult,
                   name: &str,
                   max: f64,
                   assert_percent: bool,
                   check_percent: bool|
     -> SassResult<Option<Number>> {
        Ok(match args.get(usize::MAX, name) {
            Some(v) => Some(check_num(v, name, max, assert_percent, check_percent)?),
            None => None,
        })
    };

    let red = get_arg(&mut args, "red", 255.0, false, false)?;
    let green = get_arg(&mut args, "green", 255.0, false, false)?;
    let blue = get_arg(&mut args, "blue", 255.0, false, false)?;
    let alpha = get_arg(&mut args, "alpha", 1.0, false, false)?;

    let hue = if update == UpdateComponents::Scale {
        None
    } else {
        args.get(usize::MAX, "hue")
            .map(|v| angle_value(v.node, "hue", v.span))
            .transpose()?
    };

    let saturation = get_arg(&mut args, "saturation", 100.0, false, true)?;
    let lightness = get_arg(&mut args, "lightness", 100.0, false, true)?;
    let whiteness = get_arg(&mut args, "whiteness", 100.0, true, true)?;
    let blackness = get_arg(&mut args, "blackness", 100.0, true, true)?;

    if !args.named.is_empty() {
        let argument_word = if args.named.len() == 1 {
            "argument"
        } else {
            "arguments"
        };

        let argument_names = to_sentence(
            args.named
                .keys()
                .map(|key| format!("${key}", key = key))
                .collect(),
            "or",
        );

        return Err((
            format!(
                "No {argument_word} named {argument_names}.",
                argument_word = argument_word,
                argument_names = argument_names
            ),
            span,
        )
            .into());
    }

    let has_rgb = red.is_some() || green.is_some() || blue.is_some();
    let has_sl = saturation.is_some() || lightness.is_some();
    let has_wb = whiteness.is_some() || blackness.is_some();

    if has_rgb && (has_sl || has_wb || hue.is_some()) {
        let param_type = if has_wb { "HWB" } else { "HSL" };
        return Err((
            format!(
                "RGB parameters may not be passed along with {} parameters.",
                param_type
            ),
            span,
        )
            .into());
    }

    if has_sl && has_wb {
        return Err((
            "HSL parameters may not be passed along with HWB parameters.",
            span,
        )
            .into());
    }

    fn update_value(
        current: Number,
        param: Option<Number>,
        max: f64,
        update: UpdateComponents,
    ) -> Number {
        let param = match param {
            Some(p) => p,
            None => return current,
        };

        match update {
            UpdateComponents::Change => param,
            UpdateComponents::Adjust => (param + current).clamp(0.0, max),
            UpdateComponents::Scale => {
                current
                    + if param > Number(0.0) {
                        Number(max) - current
                    } else {
                        current
                    } * param
            }
        }
    }

    fn update_rgb(current: Number, param: Option<Number>, update: UpdateComponents) -> Number {
        Number(fuzzy_round(update_value(current, param, 255.0, update).0))
    }

    let color = if has_rgb {
        Arc::new(Color::from_rgba(
            update_rgb(color.red(), red, update),
            update_rgb(color.green(), green, update),
            update_rgb(color.blue(), blue, update),
            update_value(color.alpha(), alpha, 1.0, update),
        ))
    } else if has_wb {
        Arc::new(Color::from_hwb(
            if update == UpdateComponents::Change {
                hue.unwrap_or_else(|| color.hue())
            } else {
                color.hue() + hue.unwrap_or_else(Number::zero)
            },
            update_value(color.whiteness(), whiteness, 1.0, update) * Number(100.0),
            update_value(color.blackness(), blackness, 1.0, update) * Number(100.0),
            update_value(color.alpha(), alpha, 1.0, update),
        ))
    } else if hue.is_some() || has_sl {
        let (this_hue, this_saturation, this_lightness, this_alpha) = color.as_hsla();
        Arc::new(Color::from_hsla(
            if update == UpdateComponents::Change {
                hue.unwrap_or(this_hue)
            } else {
                this_hue + hue.unwrap_or_else(Number::zero)
            },
            update_value(this_saturation, saturation, 1.0, update),
            update_value(this_lightness, lightness, 1.0, update),
            update_value(this_alpha, alpha, 1.0, update),
        ))
    } else if alpha.is_some() {
        Arc::new(color.with_alpha(update_value(color.alpha(), alpha, 1.0, update)))
    } else {
        color
    };

    Ok(Value::Color(color))
}

pub(crate) fn scale_color(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    update_components(args, visitor, UpdateComponents::Scale)
}

pub(crate) fn change_color(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    update_components(args, visitor, UpdateComponents::Change)
}

pub(crate) fn adjust_color(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    update_components(args, visitor, UpdateComponents::Adjust)
}

pub(crate) fn ie_hex_str(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;
    Ok(Value::String(color.to_ie_hex_str(), QuoteKind::None))
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("change-color", Builtin::new(change_color));
    f.insert("adjust-color", Builtin::new(adjust_color));
    f.insert("scale-color", Builtin::new(scale_color));
    f.insert("ie-hex-str", Builtin::new(ie_hex_str));
}
