use crate::builtin::builtin_imports::*;

pub(crate) fn to_upper_case(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let (mut s, q) = args
        .get_err(0, "string")?
        .assert_string_with_name("string", args.span())?;

    s.make_ascii_uppercase();

    Ok(Value::String(s, q))
}

pub(crate) fn to_lower_case(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let (mut s, q) = args
        .get_err(0, "string")?
        .assert_string_with_name("string", args.span())?;

    s.make_ascii_lowercase();

    Ok(Value::String(s, q))
}

pub(crate) fn str_length(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let s = args
        .get_err(0, "string")?
        .assert_string_with_name("string", args.span())?
        .0;

    Ok(Value::Dimension(SassNumber::new_unitless(
        s.chars().count(),
    )))
}

pub(crate) fn quote(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let s = args
        .get_err(0, "string")?
        .assert_string_with_name("string", args.span())?
        .0;

    Ok(Value::String(s, QuoteKind::Quoted))
}

pub(crate) fn unquote(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let s = args
        .get_err(0, "string")?
        .assert_string_with_name("string", args.span())?
        .0;

    Ok(Value::String(s, QuoteKind::None))
}

pub(crate) fn str_slice(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(3)?;

    let span = args.span();

    let (string, quotes) = args
        .get_err(0, "string")?
        .assert_string_with_name("string", args.span())?;

    let str_len = string.chars().count();

    let start = args
        .get_err(1, "start-at")?
        .assert_number_with_name("start-at", span)?;
    start.assert_no_units("start-at", span)?;

    let start = start.num.assert_int(span)?;

    let start = if start == 0 {
        1
    } else if start > 0 {
        (start as usize).min(str_len + 1)
    } else {
        (start + str_len as i64 + 1).max(1) as usize
    };

    let end = args
        .default_arg(
            2,
            "end-at",
            Value::Dimension(SassNumber::new_unitless(-1.0)),
        )
        .assert_number_with_name("end-at", span)?;

    end.assert_no_units("end-at", span)?;

    let mut end = end.num.assert_int(span)?;

    if end < 0 {
        end += str_len as i64 + 1;
    }

    let end = (end.max(0) as usize).min(str_len + 1);

    if start > end || start > str_len {
        Ok(Value::String(String::new(), quotes))
    } else {
        Ok(Value::String(
            string
                .chars()
                .skip(start - 1)
                .take(end - start + 1)
                .collect(),
            quotes,
        ))
    }
}

/// https://sass-lang.com/documentation/modules/string/#split
///
/// Returns a bracketed, comma-separated list of substrings of $string
/// that are separated by $separator. The $separators aren’t included
/// in these substrings.
///
/// If $limit is a number 1 or higher, this splits on at most that many
/// $separators (and so returns at most $limit + 1 strings). The last
/// substring contains the rest of the string, including any remaining
/// $separators.
pub(crate) fn str_split(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(3)?;

    let s1 = args
        .get_err(0, "string")?
        .assert_string_with_name("string", args.span())?
        .0;

    let separator = args
        .get_err(1, "separator")?
        .assert_string_with_name("separator", args.span())?
        .0;

    let limit = args.default_arg(2, "limit", Value::Null);

    let vec = if matches!(limit, Value::Null) {
        s1.split(&separator)
            .map(|s| Value::String(s.to_string(), QuoteKind::Quoted))
            .collect()
    } else {
        let limit = limit.assert_number_with_name("limit", args.span())?;
        let limit_int = limit.assert_int_with_name("limit", args.span())?;
        if limit_int < 1 {
            return Err((
                format!("$limit: Must be 1 or greater, was {}.", limit_int),
                args.span(),
            )
                .into());
        }
        // note: `1 + limit_int` is required to match dart-sass
        s1.splitn(1 + limit_int as usize, &separator)
            .map(|s| Value::String(s.to_string(), QuoteKind::Quoted))
            .collect()
    };
    Ok(Value::List(vec, ListSeparator::Comma, Brackets::Bracketed))
}

pub(crate) fn str_index(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let s1 = args
        .get_err(0, "string")?
        .assert_string_with_name("string", args.span())?
        .0;

    let substr = args
        .get_err(1, "substring")?
        .assert_string_with_name("substring", args.span())?
        .0;

    let char_position = match s1.find(&substr) {
        Some(i) => s1[0..i].chars().count() + 1,
        None => return Ok(Value::Null),
    };

    Ok(Value::Dimension(SassNumber::new_unitless(char_position)))
}

pub(crate) fn str_insert(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(3)?;
    let span = args.span();

    let (s1, quotes) = args
        .get_err(0, "string")?
        .assert_string_with_name("string", args.span())?;

    let substr = args
        .get_err(1, "insert")?
        .assert_string_with_name("insert", args.span())?
        .0;

    let index = args
        .get_err(2, "index")?
        .assert_number_with_name("index", span)?;
    index.assert_no_units("index", span)?;
    let index_int = index.assert_int_with_name("index", span)?;

    if s1.is_empty() {
        return Ok(Value::String(substr, quotes));
    }

    let len = s1.chars().count();

    // Insert substring at char position, rather than byte position
    let insert = |idx, s1: String, s2| {
        s1.chars()
            .enumerate()
            .map(|(i, c)| {
                if i + 1 == idx {
                    c.to_string() + s2
                } else if idx == 0 && i == 0 {
                    s2.to_owned() + &c.to_string()
                } else {
                    c.to_string()
                }
            })
            .collect::<String>()
    };

    let string = if index_int > 0 {
        insert((index_int as usize - 1).min(len), s1, &substr)
    } else if index_int == 0 {
        insert(0, s1, &substr)
    } else {
        let idx = (len as i64 + index_int + 1).max(0) as usize;
        insert(idx, s1, &substr)
    };

    Ok(Value::String(string, quotes))
}

#[cfg(feature = "random")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn unique_id(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(0)?;
    let mut rng = thread_rng();
    let string: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(12)
        .collect();
    Ok(Value::String(format!("id-{}", string), QuoteKind::None))
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("to-upper-case", Builtin::new(to_upper_case));
    f.insert("to-lower-case", Builtin::new(to_lower_case));
    f.insert("str-length", Builtin::new(str_length));
    f.insert("quote", Builtin::new(quote));
    f.insert("unquote", Builtin::new(unquote));
    f.insert("str-slice", Builtin::new(str_slice));
    f.insert("str-index", Builtin::new(str_index));
    f.insert("str-insert", Builtin::new(str_insert));
    #[cfg(feature = "random")]
    f.insert("unique-id", Builtin::new(unique_id));
}
