use crate::builtin::builtin_imports::*;

/// Check if `s` matches the regex `^[a-zA-Z]+\s*=`
fn is_ms_filter(s: &str) -> bool {
    let mut bytes = s.bytes();

    if !bytes.next().map_or(false, |c| c.is_ascii_alphabetic()) {
        return false;
    }

    bytes
        .skip_while(u8::is_ascii_alphabetic)
        .find(|c| !matches!(c, b' ' | b'\t' | b'\n'))
        == Some(b'=')
}

#[cfg(test)]
mod test {
    use super::is_ms_filter;
    #[test]
    fn test_is_ms_filter() {
        assert!(is_ms_filter("a=a"));
        assert!(is_ms_filter("a="));
        assert!(is_ms_filter("a  \t\n  =a"));
        assert!(!is_ms_filter("a  \t\n  a=a"));
        assert!(!is_ms_filter("aa"));
        assert!(!is_ms_filter("   aa"));
        assert!(!is_ms_filter("=a"));
        assert!(!is_ms_filter("1=a"));
    }
}

pub(crate) fn alpha(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    if args.len() <= 1 {
        let color = args.get_err(0, "color")?;

        if let Value::String(s, QuoteKind::None) = &color {
            if is_ms_filter(s) {
                return Ok(Value::String(format!("alpha({})", s), QuoteKind::None));
            }
        }

        let color = color.assert_color_with_name("color", args.span())?;

        Ok(Value::Dimension(SassNumber::new_unitless(color.alpha())))
    } else {
        let err = args.max_args(1);
        let args = args
            .get_variadic()?
            .into_iter()
            .map(|arg| match arg.node {
                Value::String(s, QuoteKind::None) if is_ms_filter(&s) => Ok(s),
                _ => {
                    err.clone()?;
                    unreachable!()
                }
            })
            .collect::<SassResult<Vec<String>>>()?;

        Ok(Value::String(
            format!("alpha({})", args.join(", "),),
            QuoteKind::None,
        ))
    }
}

pub(crate) fn opacity(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    match args.get_err(0, "color")? {
        Value::Color(c) => Ok(Value::Dimension(SassNumber::new_unitless(c.alpha()))),
        Value::Dimension(SassNumber {
            num,
            unit,
            as_slash: _,
        }) => Ok(Value::String(
            format!("opacity({}{})", num.inspect(), unit),
            QuoteKind::None,
        )),
        v => Err((
            format!("$color: {} is not a color.", v.inspect(args.span())?),
            args.span(),
        )
            .into()),
    }
}

fn opacify(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;
    let amount = args
        .get_err(1, "amount")?
        .assert_number_with_name("amount", args.span())?;

    amount.assert_bounds_with_unit("amount", 0.0, 1.0, &Unit::None, args.span())?;

    Ok(Value::Color(Arc::new(color.fade_in(amount.num))))
}

fn transparentize(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    let amount = args
        .get_err(1, "amount")?
        .assert_number_with_name("amount", args.span())?;

    amount.assert_bounds_with_unit("amount", 0.0, 1.0, &Unit::None, args.span())?;

    Ok(Value::Color(Arc::new(color.fade_out(amount.num))))
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("alpha", Builtin::new(alpha));
    f.insert("opacity", Builtin::new(opacity));
    f.insert("opacify", Builtin::new(opacify));
    f.insert("fade-in", Builtin::new(opacify));
    f.insert("transparentize", Builtin::new(transparentize));
    f.insert("fade-out", Builtin::new(transparentize));
}
