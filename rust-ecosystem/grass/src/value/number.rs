use std::{
    convert::From,
    fmt, mem,
    ops::{
        Add, AddAssign, Deref, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
    },
};

use crate::{
    error::SassResult,
    unit::{Unit, UNIT_CONVERSION_TABLE},
};

use codemap::Span;

const PRECISION: i32 = 10;

fn epsilon() -> f64 {
    10.0_f64.powi(-PRECISION - 1)
}

fn inverse_epsilon() -> f64 {
    10.0_f64.powi(PRECISION + 1)
}

/// Thin wrapper around `f64` providing utility functions and more accurate
/// operations -- namely a Sass-compatible modulo
#[derive(Clone, Copy, PartialOrd)]
#[repr(transparent)]
pub struct Number(pub f64);

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        fuzzy_equals(self.0, other.0)
    }
}

impl Eq for Number {}

pub(crate) fn fuzzy_equals(a: f64, b: f64) -> bool {
    if a == b {
        return true;
    }

    (a - b).abs() <= epsilon() && (a * inverse_epsilon()).round() == (b * inverse_epsilon()).round()
}

pub(crate) fn fuzzy_as_int(num: f64) -> Option<i64> {
    if !num.is_finite() {
        return None;
    }

    let rounded = num.round();

    if fuzzy_equals(num, rounded) {
        // todo: this can oveflow
        Some(rounded as i64)
    } else {
        None
    }
}

pub(crate) fn fuzzy_round(number: f64) -> f64 {
    // If the number is within epsilon of X.5, round up (or down for negative
    // numbers).
    if number > 0.0 {
        if fuzzy_less_than(number % 1.0, 0.5) {
            number.floor()
        } else {
            number.ceil()
        }
    } else if fuzzy_less_than_or_equals(number % 1.0, 0.5) {
        number.floor()
    } else {
        number.ceil()
    }
}

pub(crate) fn fuzzy_less_than(number1: f64, number2: f64) -> bool {
    number1 < number2 && !fuzzy_equals(number1, number2)
}

pub(crate) fn fuzzy_less_than_or_equals(number1: f64, number2: f64) -> bool {
    number1 < number2 || fuzzy_equals(number1, number2)
}

impl Number {
    /// This differs from `std::cmp::min` when either value is NaN
    pub fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    /// This differs from `std::cmp::max` when either value is NaN
    pub fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }

    pub fn is_positive(self) -> bool {
        self.0.is_sign_positive() && !self.is_zero()
    }

    pub fn is_negative(self) -> bool {
        self.0.is_sign_negative() && !self.is_zero()
    }

    pub fn assert_int(self, span: Span) -> SassResult<i64> {
        match fuzzy_as_int(self.0) {
            Some(i) => Ok(i),
            None => Err((format!("{} is not an int.", self.0), span).into()),
        }
    }

    pub fn round(self) -> Self {
        Self(self.0.round())
    }

    pub fn ceil(self) -> Self {
        Self(self.0.ceil())
    }

    pub fn floor(self) -> Self {
        Self(self.0.floor())
    }

    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    pub fn clamp(self, min: f64, max: f64) -> Self {
        Number(min.max(self.0.min(max)))
    }

    pub fn sqrt(self) -> Self {
        Self(self.0.sqrt())
    }

    pub fn ln(self) -> Self {
        Self(self.0.ln())
    }

    pub fn log(self, base: Number) -> Self {
        Self(self.0.log(base.0))
    }

    pub fn pow(self, exponent: Self) -> Self {
        Self(self.0.powf(exponent.0))
    }

    /// Invariants: `from.comparable(&to)` must be true
    pub fn convert(self, from: &Unit, to: &Unit) -> Self {
        if from == &Unit::None || to == &Unit::None || from == to {
            return self;
        }

        debug_assert!(from.comparable(to), "from: {:?}, to: {:?}", from, to);

        Number(self.0 * UNIT_CONVERSION_TABLE[to][from])
    }
}

macro_rules! inverse_trig_fn(
    ($name:ident) => {
        pub fn $name(self) -> Self {
            Self(self.0.$name().to_degrees())
        }
    }
);

/// Trigonometry methods
impl Number {
    inverse_trig_fn!(acos);
    inverse_trig_fn!(asin);
    inverse_trig_fn!(atan);
}

impl Default for Number {
    fn default() -> Self {
        Self::zero()
    }
}

impl Number {
    pub const fn one() -> Self {
        Self(1.0)
    }

    pub fn is_one(self) -> bool {
        fuzzy_equals(self.0, 1.0)
    }

    pub const fn zero() -> Self {
        Self(0.0)
    }

    pub fn is_zero(self) -> bool {
        fuzzy_equals(self.0, 0.0)
    }
}

impl Deref for Number {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! from_integer {
    ($ty:ty) => {
        impl From<$ty> for Number {
            fn from(b: $ty) -> Self {
                Number(b as f64)
            }
        }
    };
}

macro_rules! from_smaller_integer {
    ($ty:ty) => {
        impl From<$ty> for Number {
            fn from(val: $ty) -> Self {
                Self(f64::from(val))
            }
        }
    };
}

impl From<i64> for Number {
    fn from(val: i64) -> Self {
        Self(val as f64)
    }
}

impl From<f64> for Number {
    fn from(b: f64) -> Self {
        Self(b)
    }
}

from_integer!(usize);
from_integer!(isize);
from_smaller_integer!(i32);
from_smaller_integer!(u32);
from_smaller_integer!(u8);

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Number( {} )", self.to_string(false))
    }
}

impl Number {
    pub(crate) fn inspect(self) -> String {
        self.to_string(false)
    }

    pub(crate) fn to_string(self, is_compressed: bool) -> String {
        if self.0.is_infinite() && self.0.is_sign_negative() {
            return "-Infinity".to_owned();
        } else if self.0.is_infinite() {
            return "Infinity".to_owned();
        }

        let mut buffer = String::with_capacity(3);

        if self.0 < 0.0 {
            buffer.push('-');
        }

        let num = self.0.abs();

        if is_compressed && num < 1.0 {
            buffer.push_str(
                format!("{:.10}", num)[1..]
                    .trim_end_matches('0')
                    .trim_end_matches('.'),
            );
        } else {
            buffer.push_str(
                format!("{:.10}", num)
                    .trim_end_matches('0')
                    .trim_end_matches('.'),
            );
        }

        if buffer.is_empty() || buffer == "-" || buffer == "-0" {
            return "0".to_owned();
        }

        buffer
    }
}

impl Add for Number {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl AddAssign for Number {
    fn add_assign(&mut self, other: Self) {
        let tmp = mem::take(self);
        *self = tmp + other;
    }
}

impl Sub for Number {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl SubAssign for Number {
    fn sub_assign(&mut self, other: Self) {
        let tmp = mem::take(self);
        *self = tmp - other;
    }
}

impl Mul for Number {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}

impl Mul<i64> for Number {
    type Output = Self;

    fn mul(self, other: i64) -> Self {
        Self(self.0 * other as f64)
    }
}

impl MulAssign<i64> for Number {
    fn mul_assign(&mut self, other: i64) {
        let tmp = mem::take(self);
        *self = tmp * other;
    }
}

impl MulAssign for Number {
    fn mul_assign(&mut self, other: Self) {
        let tmp = mem::take(self);
        *self = tmp * other;
    }
}

impl Div for Number {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self(self.0 / other.0)
    }
}

impl DivAssign for Number {
    fn div_assign(&mut self, other: Self) {
        let tmp = mem::take(self);
        *self = tmp / other;
    }
}

fn real_mod(n1: f64, n2: f64) -> f64 {
    n1.rem_euclid(n2)
}

fn modulo(n1: f64, n2: f64) -> f64 {
    if n2 > 0.0 {
        return real_mod(n1, n2);
    }

    if n2 == 0.0 {
        return f64::NAN;
    }

    let result = real_mod(n1, n2);

    if result == 0.0 {
        0.0
    } else {
        result + n2
    }
}

impl Rem for Number {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        Self(modulo(self.0, other.0))
    }
}

impl RemAssign for Number {
    fn rem_assign(&mut self, other: Self) {
        let tmp = mem::take(self);
        *self = tmp % other;
    }
}

impl Neg for Number {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}
