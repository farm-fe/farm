//! A color is internally represented as either RGBA or HSLA.
//!
//! Colors can be constructed in Sass through names (e.g. red, blue, aqua)
//! or the builtin functions `rgb()`, `rgba()`, `hsl()`, and `hsla()`,
//! all of which can accept 1-4 arguments.
//!
//! It is necessary to retain the original values with which the
//! color was constructed.
//! E.g. `hsla(.999999999999, 100, 100, 1)` should retain its full HSLA
//! values to an arbitrary precision.
//!
//! Color values matching named colors are implicitly converted to named colors
//! E.g. `rgba(255, 0, 0, 1)` => `red`
//!
//! Named colors retain their original casing,
//! so `rEd` should be emitted as `rEd`.

use crate::value::{fuzzy_round, Number};
pub(crate) use name::NAMED_COLORS;

mod name;

// todo: only store alpha once on color
#[derive(Debug, Clone)]
pub struct Color {
    rgba: Rgb,
    hsla: Option<Hsl>,
    alpha: Number,
    pub(crate) format: ColorFormat,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ColorFormat {
    Rgb,
    Hsl,
    /// Literal string from source text. Either a named color like `red` or a hex color
    // todo: make this is a span and lookup text from codemap
    Literal(String),
    /// Use the most appropriate format
    Infer,
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        if self.alpha != other.alpha
            && !(self.alpha >= Number::one() && other.alpha >= Number::one())
        {
            return false;
        }

        self.rgba == other.rgba
    }
}

impl Eq for Color {}

impl Color {
    pub(crate) const fn new_rgba(
        red: Number,
        green: Number,
        blue: Number,
        alpha: Number,
        format: ColorFormat,
    ) -> Color {
        Color {
            rgba: Rgb::new(red, green, blue),
            alpha,
            hsla: None,
            format,
        }
    }

    const fn new_hsla(red: Number, green: Number, blue: Number, alpha: Number, hsla: Hsl) -> Color {
        Color {
            rgba: Rgb::new(red, green, blue),
            alpha,
            hsla: Some(hsla),
            format: ColorFormat::Infer,
        }
    }
}

#[derive(Debug, Clone)]
struct Rgb {
    red: Number,
    green: Number,
    blue: Number,
}

impl PartialEq for Rgb {
    fn eq(&self, other: &Self) -> bool {
        if self.red != other.red && !(self.red >= Number(255.0) && other.red >= Number(255.0)) {
            return false;
        }
        if self.green != other.green
            && !(self.green >= Number(255.0) && other.green >= Number(255.0))
        {
            return false;
        }
        if self.blue != other.blue && !(self.blue >= Number(255.0) && other.blue >= Number(255.0)) {
            return false;
        }
        true
    }
}

impl Eq for Rgb {}

impl Rgb {
    pub const fn new(red: Number, green: Number, blue: Number) -> Self {
        Rgb { red, green, blue }
    }
}

#[derive(Debug, Clone)]
struct Hsl {
    hue: Number,
    saturation: Number,
    luminance: Number,
}

impl Hsl {
    pub const fn new(hue: Number, saturation: Number, luminance: Number) -> Self {
        Hsl {
            hue,
            saturation,
            luminance,
        }
    }

    pub fn hue(&self) -> Number {
        self.hue
    }

    pub fn saturation(&self) -> Number {
        self.saturation
    }

    pub fn luminance(&self) -> Number {
        self.luminance
    }
}

// RGBA color functions
impl Color {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8, format: String) -> Self {
        Color {
            rgba: Rgb::new(red.into(), green.into(), blue.into()),
            hsla: None,
            alpha: alpha.into(),
            format: ColorFormat::Literal(format),
        }
    }

    /// Create a new `Color` with just RGBA values.
    /// Color representation is created automatically.
    pub fn from_rgba(
        mut red: Number,
        mut green: Number,
        mut blue: Number,
        mut alpha: Number,
    ) -> Self {
        red = red.clamp(0.0, 255.0);
        green = green.clamp(0.0, 255.0);
        blue = blue.clamp(0.0, 255.0);
        alpha = alpha.clamp(0.0, 1.0);

        Color::new_rgba(red, green, blue, alpha, ColorFormat::Infer)
    }

    pub fn from_rgba_fn(
        mut red: Number,
        mut green: Number,
        mut blue: Number,
        mut alpha: Number,
    ) -> Self {
        red = red.clamp(0.0, 255.0);
        green = green.clamp(0.0, 255.0);
        blue = blue.clamp(0.0, 255.0);
        alpha = alpha.clamp(0.0, 1.0);

        Color::new_rgba(red, green, blue, alpha, ColorFormat::Rgb)
    }

    pub fn red(&self) -> Number {
        self.rgba.red.round()
    }

    pub fn blue(&self) -> Number {
        self.rgba.blue.round()
    }

    pub fn green(&self) -> Number {
        self.rgba.green.round()
    }

    /// Mix two colors together with weight
    /// Algorithm adapted from
    /// <https://github.com/sass/dart-sass/blob/0d0270cb12a9ac5cce73a4d0785fecb00735feee/lib/src/functions/color.dart#L718>
    pub fn mix(&self, other: &Color, weight: Number) -> Self {
        let weight = weight.clamp(0.0, 100.0);
        let normalized_weight = weight * Number(2.0) - Number::one();
        let alpha_distance = self.alpha() - other.alpha();

        let combined_weight1 = if normalized_weight * alpha_distance == Number(-1.0) {
            normalized_weight
        } else {
            (normalized_weight + alpha_distance)
                / (Number::one() + normalized_weight * alpha_distance)
        };
        let weight1 = (combined_weight1 + Number::one()) / Number(2.0);
        let weight2 = Number::one() - weight1;

        Color::from_rgba(
            self.red() * weight1 + other.red() * weight2,
            self.green() * weight1 + other.green() * weight2,
            self.blue() * weight1 + other.blue() * weight2,
            self.alpha() * weight + other.alpha() * (Number::one() - weight),
        )
    }
}

/// HSLA color functions
/// Algorithms adapted from <http://www.niwa.nu/2013/05/math-behind-colorspace-conversions-rgb-hsl/>
impl Color {
    /// Calculate hue from RGBA values
    pub fn hue(&self) -> Number {
        if let Some(h) = &self.hsla {
            return h.hue();
        }

        let red = self.red() / Number(255.0);
        let green = self.green() / Number(255.0);
        let blue = self.blue() / Number(255.0);

        let min = red.min(green.min(blue));
        let max = red.max(green.max(blue));

        let delta = max - min;

        let hue = if min == max {
            Number::zero()
        } else if max == red {
            Number(60.0) * (green - blue) / delta
        } else if max == green {
            Number(120.0) + Number(60.0) * (blue - red) / delta
        } else {
            Number(240.0) + Number(60.0) * (red - green) / delta
        };

        hue % Number(360.0)
    }

    /// Calculate saturation from RGBA values
    pub fn saturation(&self) -> Number {
        if let Some(h) = &self.hsla {
            return h.saturation() * Number(100.0);
        }

        let red: Number = self.red() / Number(255.0);
        let green = self.green() / Number(255.0);
        let blue = self.blue() / Number(255.0);

        let min = red.min(green.min(blue));
        let max = red.max(green.max(blue));

        if min == max {
            return Number::zero();
        }

        let delta = max - min;

        let sum = max + min;

        let s = delta
            / if sum > Number::one() {
                Number(2.0) - sum
            } else {
                sum
            };

        s * Number(100.0)
    }

    /// Calculate luminance from RGBA values
    pub fn lightness(&self) -> Number {
        if let Some(h) = &self.hsla {
            return h.luminance() * Number(100.0);
        }

        let red: Number = self.red() / Number(255.0);
        let green = self.green() / Number(255.0);
        let blue = self.blue() / Number(255.0);
        let min = red.min(green.min(blue));
        let max = red.max(green.max(blue));
        (((min + max) / Number(2.0)) * Number(100.0)).round()
    }

    pub fn as_hsla(&self) -> (Number, Number, Number, Number) {
        if let Some(h) = &self.hsla {
            return (h.hue(), h.saturation(), h.luminance(), self.alpha());
        }

        let red = self.red() / Number(255.0);
        let green = self.green() / Number(255.0);
        let blue = self.blue() / Number(255.0);
        let min = red.min(green.min(blue));
        let max = red.max(green.max(blue));

        let lightness = (min + max) / Number(2.0);

        let saturation = if min == max {
            Number::zero()
        } else {
            let d = max - min;
            let mm = max + min;
            d / if mm > Number::one() {
                Number(2.0) - mm
            } else {
                mm
            }
        };

        let mut hue = if min == max {
            Number::zero()
        } else if blue == max {
            Number(4.0) + (red - green) / (max - min)
        } else if green == max {
            Number(2.0) + (blue - red) / (max - min)
        } else {
            (green - blue) / (max - min)
        };

        if hue.is_negative() {
            hue += Number(360.0);
        }

        hue *= Number(60.0);

        (hue % Number(360.0), saturation, lightness, self.alpha())
    }

    pub fn adjust_hue(&self, degrees: Number) -> Self {
        let (hue, saturation, luminance, alpha) = self.as_hsla();
        Color::from_hsla(hue + degrees, saturation, luminance, alpha)
    }

    pub fn lighten(&self, amount: Number) -> Self {
        let (hue, saturation, luminance, alpha) = self.as_hsla();
        Color::from_hsla(hue, saturation, luminance + amount, alpha)
    }

    pub fn darken(&self, amount: Number) -> Self {
        let (hue, saturation, luminance, alpha) = self.as_hsla();
        Color::from_hsla(hue, saturation, luminance - amount, alpha)
    }

    pub fn saturate(&self, amount: Number) -> Self {
        let (hue, saturation, luminance, alpha) = self.as_hsla();
        Color::from_hsla(hue, (saturation + amount).clamp(0.0, 1.0), luminance, alpha)
    }

    pub fn desaturate(&self, amount: Number) -> Self {
        let (hue, saturation, luminance, alpha) = self.as_hsla();
        Color::from_hsla(hue, (saturation - amount).clamp(0.0, 1.0), luminance, alpha)
    }

    pub fn from_hsla_fn(hue: Number, saturation: Number, luminance: Number, alpha: Number) -> Self {
        let mut color = Self::from_hsla(hue, saturation, luminance, alpha);
        color.format = ColorFormat::Hsl;
        color
    }

    /// Create RGBA representation from HSLA values
    pub fn from_hsla(hue: Number, saturation: Number, lightness: Number, alpha: Number) -> Self {
        let hue = hue % Number(360.0);
        let hsla = Hsl::new(hue, saturation.clamp(0.0, 1.0), lightness.clamp(0.0, 1.0));

        let scaled_hue = hue.0 / 360.0;
        let scaled_saturation = saturation.0.clamp(0.0, 1.0);
        let scaled_lightness = lightness.0.clamp(0.0, 1.0);

        let m2 = if scaled_lightness <= 0.5 {
            scaled_lightness * (scaled_saturation + 1.0)
        } else {
            scaled_lightness.mul_add(-scaled_saturation, scaled_lightness + scaled_saturation)
        };

        let m1 = scaled_lightness.mul_add(2.0, -m2);

        let red = fuzzy_round(Self::hue_to_rgb(m1, m2, scaled_hue + 1.0 / 3.0) * 255.0);
        let green = fuzzy_round(Self::hue_to_rgb(m1, m2, scaled_hue) * 255.0);
        let blue = fuzzy_round(Self::hue_to_rgb(m1, m2, scaled_hue - 1.0 / 3.0) * 255.0);

        Color::new_hsla(Number(red), Number(green), Number(blue), alpha, hsla)
    }

    fn hue_to_rgb(m1: f64, m2: f64, mut hue: f64) -> f64 {
        if hue < 0.0 {
            hue += 1.0;
        }
        if hue > 1.0 {
            hue -= 1.0;
        }

        if hue < 1.0 / 6.0 {
            ((m2 - m1) * hue).mul_add(6.0, m1)
        } else if hue < 1.0 / 2.0 {
            m2
        } else if hue < 2.0 / 3.0 {
            ((m2 - m1) * (2.0 / 3.0 - hue)).mul_add(6.0, m1)
        } else {
            m1
        }
    }

    pub fn invert(&self, weight: Number) -> Self {
        if weight.is_zero() {
            return self.clone();
        }

        let red = Number(255.0) - self.red();
        let green = Number(255.0) - self.green();
        let blue = Number(255.0) - self.blue();

        let inverse = Color::new_rgba(red, green, blue, self.alpha(), ColorFormat::Infer);

        inverse.mix(self, weight)
    }

    pub fn complement(&self) -> Self {
        let (hue, saturation, luminance, alpha) = self.as_hsla();

        Color::from_hsla(hue + Number(180.0), saturation, luminance, alpha)
    }
}

/// Opacity color functions
impl Color {
    pub fn alpha(&self) -> Number {
        if self.alpha > Number::one() {
            self.alpha / Number(255.0)
        } else {
            self.alpha
        }
    }

    /// Change `alpha` to value given
    pub fn with_alpha(&self, alpha: Number) -> Self {
        Color::from_rgba(self.red(), self.green(), self.blue(), alpha)
    }

    /// Makes a color more opaque.
    /// Takes a color and a number between 0 and 1,
    /// and returns a color with the opacity increased by that amount.
    pub fn fade_in(&self, amount: Number) -> Self {
        Color::from_rgba(self.red(), self.green(), self.blue(), self.alpha() + amount)
    }

    /// Makes a color more transparent.
    /// Takes a color and a number between 0 and 1,
    /// and returns a color with the opacity decreased by that amount.
    pub fn fade_out(&self, amount: Number) -> Self {
        Color::from_rgba(self.red(), self.green(), self.blue(), self.alpha() - amount)
    }
}

/// Other color functions
impl Color {
    pub fn to_ie_hex_str(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            fuzzy_round(self.alpha().0 * 255.0) as u8,
            self.red().0 as u8,
            self.green().0 as u8,
            self.blue().0 as u8
        )
    }
}

/// HWB color functions
impl Color {
    pub fn from_hwb(hue: Number, white: Number, black: Number, mut alpha: Number) -> Color {
        let hue = Number(hue.rem_euclid(360.0) / 360.0);
        let mut scaled_white = white.0 / 100.0;
        let mut scaled_black = black.0 / 100.0;
        alpha = alpha.clamp(0.0, 1.0);

        let white_black_sum = scaled_white + scaled_black;

        if white_black_sum > 1.0 {
            scaled_white /= white_black_sum;
            scaled_black /= white_black_sum;
        }

        let factor = 1.0 - scaled_white - scaled_black;

        let to_rgb = |hue: f64| -> Number {
            let channel = Self::hue_to_rgb(0.0, 1.0, hue).mul_add(factor, scaled_white);
            Number(fuzzy_round(channel * 255.0))
        };

        let red = to_rgb(hue.0 + 1.0 / 3.0);
        let green = to_rgb(hue.0);
        let blue = to_rgb(hue.0 - 1.0 / 3.0);

        Color::new_rgba(red, green, blue, alpha, ColorFormat::Infer)
    }

    pub fn whiteness(&self) -> Number {
        self.red().min(self.green()).min(self.blue()) / Number(255.0)
    }

    pub fn blackness(&self) -> Number {
        Number(1.0) - (self.red().max(self.green()).max(self.blue()) / Number(255.0))
    }
}
