use codemap::Span;

use crate::{
    builtin::builtin_imports::Unit,
    error::SassResult,
    value::{conversion_factor, Number, Value},
};

use super::GlobalFunctionMap;

pub mod hsl;
pub mod hwb;
pub mod opacity;
pub mod other;
pub mod rgb;

#[derive(Debug, Clone)]
pub(crate) enum ParsedChannels {
    String(String),
    List(Vec<Value>),
}

pub(crate) fn angle_value(num: Value, name: &str, span: Span) -> SassResult<Number> {
    let angle = num.assert_number_with_name(name, span)?;

    if angle.has_compatible_units(&Unit::Deg) {
        let factor = conversion_factor(&angle.unit, &Unit::Deg).unwrap();

        return Ok(angle.num * Number(factor));
    }

    Ok(angle.num)
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    hsl::declare(f);
    opacity::declare(f);
    other::declare(f);
    rgb::declare(f);
}
