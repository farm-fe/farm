use std::{
    ops::{Add, Div, Mul, Sub},
    sync::Arc,
};

use codemap::Span;

use crate::{
    error::SassResult,
    serializer::{inspect_float, inspect_number},
    unit::{are_any_convertible, known_compatibilities_by_unit, Unit, UNIT_CONVERSION_TABLE},
    Options,
};

use super::{fuzzy_as_int, Number};

#[derive(Debug, Clone)]
pub struct SassNumber {
    pub num: Number,
    pub unit: Unit,
    pub as_slash: Option<Arc<(Self, Self)>>,
}

pub(crate) fn conversion_factor(from: &Unit, to: &Unit) -> Option<f64> {
    if from == to {
        return Some(1.0);
    }

    UNIT_CONVERSION_TABLE.get(to)?.get(from).copied()
}

impl SassNumber {
    pub fn new_unitless<N: Into<Number>>(n: N) -> Self {
        Self {
            num: n.into(),
            unit: Unit::None,
            as_slash: None,
        }
    }

    pub fn has_comparable_units(&self, other_unit: &Unit) -> bool {
        self.unit.comparable(other_unit)
    }

    /// Unlike [`SassNumber::has_comparable_units`], this considers `Unit::None`
    /// to be compatible only with itself
    pub fn has_compatible_units(&self, other_unit: &Unit) -> bool {
        if (self.unit == Unit::None || *other_unit == Unit::None) && self.unit != *other_unit {
            return false;
        }

        self.has_comparable_units(other_unit)
    }

    #[allow(clippy::collapsible_if)]
    pub(crate) fn multiply_units(&self, mut num: f64, other_unit: Unit) -> SassNumber {
        let (numer_units, denom_units) = self.unit.clone().numer_and_denom();
        let (other_numer, other_denom) = other_unit.numer_and_denom();

        if numer_units.is_empty() {
            if other_denom.is_empty() && !are_any_convertible(&denom_units, &other_numer) {
                return SassNumber {
                    num: Number(num),
                    unit: Unit::new(other_numer, denom_units),
                    as_slash: None,
                };
            } else if denom_units.is_empty() {
                return SassNumber {
                    num: Number(num),
                    unit: Unit::new(other_numer, other_denom),
                    as_slash: None,
                };
            }
        } else if other_numer.is_empty() {
            if other_denom.is_empty()
                || (denom_units.is_empty() && !are_any_convertible(&numer_units, &other_denom))
            {
                return SassNumber {
                    num: Number(num),
                    unit: Unit::new(numer_units, other_denom),
                    as_slash: None,
                };
            }
        }

        let mut new_numer = Vec::new();

        let mut mutable_other_denom = other_denom;

        for numer in numer_units {
            let mut has_removed = false;
            mutable_other_denom.retain(|denom| {
                if has_removed {
                    return true;
                }

                if let Some(factor) = conversion_factor(denom, &numer) {
                    num /= factor;
                    has_removed = true;
                    return false;
                }

                true
            });

            if !has_removed {
                new_numer.push(numer);
            }
        }

        let mut mutable_denom = denom_units;
        for numer in other_numer {
            let mut has_removed = false;
            mutable_denom.retain(|denom| {
                if has_removed {
                    return true;
                }

                if let Some(factor) = conversion_factor(denom, &numer) {
                    num /= factor;
                    has_removed = true;
                    return false;
                }

                true
            });

            if !has_removed {
                new_numer.push(numer);
            }
        }

        mutable_denom.append(&mut mutable_other_denom);

        SassNumber {
            num: Number(num),
            unit: Unit::new(new_numer, mutable_denom),
            as_slash: None,
        }
    }

    pub fn assert_no_units(&self, name: &str, span: Span) -> SassResult<()> {
        if self.unit == Unit::None {
            Ok(())
        } else {
            Err((
                format!(
                    "${name}: Expected {} to have no units.",
                    inspect_number(self, &Options::default(), span)?,
                    name = name,
                ),
                span,
            )
                .into())
        }
    }

    pub fn assert_unit(&self, unit: &Unit, name: &str, span: Span) -> SassResult<()> {
        if self.unit == *unit {
            Ok(())
        } else {
            Err((
                format!(
                    "${name}: Expected {} to have unit \"{unit}\".",
                    inspect_number(self, &Options::default(), span)?,
                    name = name,
                    unit = unit,
                ),
                span,
            )
                .into())
        }
    }

    pub fn assert_bounds(&self, name: &str, min: f64, max: f64, span: Span) -> SassResult<()> {
        self.assert_bounds_with_unit(name, min, max, &self.unit, span)
    }

    pub fn assert_int_with_name(&self, name: &'static str, span: Span) -> SassResult<i64> {
        match fuzzy_as_int(self.num.0) {
            Some(i) => Ok(i),
            None => Err((
                format!(
                    "${name}: {} is not an int.",
                    inspect_number(self, &Options::default(), span)?,
                    name = name,
                ),
                span,
            )
                .into()),
        }
    }

    pub fn assert_bounds_with_unit(
        &self,
        name: &str,
        min: f64,
        max: f64,
        unit: &Unit,
        span: Span,
    ) -> SassResult<()> {
        if !(self.num <= Number(max) && self.num >= Number(min)) {
            return Err((
                format!(
                    "${}: Expected {} to be within {}{} and {}{}.",
                    name,
                    inspect_number(self, &Options::default(), span)?,
                    inspect_float(min, &Options::default(), span),
                    unit,
                    inspect_float(max, &Options::default(), span),
                    unit,
                ),
                span,
            )
                .into());
        }

        Ok(())
    }

    pub fn is_comparable_to(&self, other: &Self) -> bool {
        self.unit.comparable(&other.unit)
    }

    /// For use in calculations
    pub fn has_possibly_compatible_units(&self, other: &Self) -> bool {
        if self.unit.is_complex() || other.unit.is_complex() {
            return false;
        }

        let known_compatibilities = match known_compatibilities_by_unit(&self.unit) {
            Some(known_compatibilities) => known_compatibilities,
            None => return true,
        };

        known_compatibilities.contains(&other.unit)
            || known_compatibilities_by_unit(&other.unit).is_none()
    }

    pub fn unit(&self) -> &Unit {
        &self.unit
    }
}

impl PartialEq for SassNumber {
    fn eq(&self, other: &Self) -> bool {
        if !self.unit.comparable(&other.unit) {
            return false;
        }

        if (other.unit == Unit::None || self.unit == Unit::None) && self.unit != other.unit {
            return false;
        }

        self.num == other.num.convert(&other.unit, &self.unit)
    }
}

impl Eq for SassNumber {}

impl Add<SassNumber> for SassNumber {
    type Output = SassNumber;
    fn add(self, rhs: SassNumber) -> Self::Output {
        if self.unit == rhs.unit {
            SassNumber {
                num: self.num + rhs.num,
                unit: self.unit,
                as_slash: None,
            }
        } else if self.unit == Unit::None {
            SassNumber {
                num: self.num + rhs.num,
                unit: rhs.unit,
                as_slash: None,
            }
        } else if rhs.unit == Unit::None {
            SassNumber {
                num: self.num + rhs.num,
                unit: self.unit,
                as_slash: None,
            }
        } else {
            SassNumber {
                num: self.num + rhs.num.convert(&rhs.unit, &self.unit),
                unit: self.unit,
                as_slash: None,
            }
        }
    }
}

impl Sub<SassNumber> for SassNumber {
    type Output = SassNumber;

    fn sub(self, rhs: SassNumber) -> Self::Output {
        if self.unit == rhs.unit {
            SassNumber {
                num: self.num - rhs.num,
                unit: self.unit,
                as_slash: None,
            }
        } else if self.unit == Unit::None {
            SassNumber {
                num: self.num - rhs.num,
                unit: rhs.unit,
                as_slash: None,
            }
        } else if rhs.unit == Unit::None {
            SassNumber {
                num: self.num - rhs.num,
                unit: self.unit,
                as_slash: None,
            }
        } else {
            SassNumber {
                num: self.num - rhs.num.convert(&rhs.unit, &self.unit),
                unit: self.unit,
                as_slash: None,
            }
        }
    }
}

impl Mul<SassNumber> for SassNumber {
    type Output = SassNumber;
    fn mul(self, rhs: SassNumber) -> Self::Output {
        if rhs.unit == Unit::None {
            return SassNumber {
                num: self.num * rhs.num,
                unit: self.unit,
                as_slash: None,
            };
        }

        self.multiply_units(self.num.0 * rhs.num.0, rhs.unit)
    }
}

impl Div<SassNumber> for SassNumber {
    type Output = SassNumber;
    fn div(self, rhs: SassNumber) -> Self::Output {
        if rhs.unit == Unit::None {
            return SassNumber {
                num: self.num / rhs.num,
                unit: self.unit,
                as_slash: None,
            };
        }

        self.multiply_units(self.num.0 / rhs.num.0, rhs.unit.invert())
    }
}
