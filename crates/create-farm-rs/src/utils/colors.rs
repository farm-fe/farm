#![allow(unused)]
use std::fmt;

pub const BLACK: &str = "\x1b[30m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const WHITE: &str = "\x1b[37m";
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const ITALIC: &str = "\x1b[3m";
pub const DIM: &str = "\x1b[2m";
pub const DIMRESET: &str = "\x1b[22m";

pub fn remove_colors(s: &str) -> String {
  s.replace(BLACK, "")
    .replace(RED, "")
    .replace(GREEN, "")
    .replace(YELLOW, "")
    .replace(BLUE, "")
    .replace(WHITE, "")
    .replace(RESET, "")
    .replace(BOLD, "")
    .replace(ITALIC, "")
    .replace(DIM, "")
    .replace(DIMRESET, "")
}

type StylerEnabled = fn(&str) -> String;
type ColorFunction = fn(&str) -> String;

// brand gradient colors
const GRADIENT_PURPLE_COLOR: [u8; 3] = [176, 106, 179];
const GRADIENT_PINK_COLOR: [u8; 3] = [198, 66, 110];
const BRAND_GRADIENT_COLORS: [u8; 3] = [255, 182, 193];
const BRAND_GRADIENT_COLORS2: [u8; 3] = [128, 0, 128];

pub fn is_color_enabled() -> bool {
  true
}

pub fn create_formatter<'a>(open: &'a str, close: &'a str, replace: Option<&'a str>) -> impl Fn(&'a str) -> String + 'a {
  move |input| {
    let string = input.to_string();
    let index = string.find(close).unwrap_or(open.len());
    if let Some(replace_str) = replace {
      let mut result = String::new();
      result.push_str(open);
      result.push_str(&replace_close(&string, close, replace_str, index));
      result.push_str(close);
      result
    } else {
      format!("{open}{string}{close}")
    }
  }
}

pub fn replace_close(string: &str, close: &str, replace: &str, index: usize) -> String {
  let (start, end) = string.split_at(index);
  let next_index_option = end.find(close).map(|i| i + close.len());
  let next_index = next_index_option.unwrap_or(0);
  if next_index != 0 {
    format!(
      "{}{}{}",
      start,
      replace,
      replace_close(&end[next_index..], close, replace, next_index)
    )
  } else {
    format!("{start}{replace}")
  }
}

pub fn reset(s: &str) -> String {
  if is_color_enabled() {
    format!("\x1b[0m{s}")
  } else {
    s.to_string()
  }
}

pub fn bold(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[1m", "\x1b[22m", Some("\x1b[22m\x1b[1m"))(s)
  } else {
    s.to_string()
  }
}

pub fn dim(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[2m", "\x1b[22m", Some("\x1b[22m\x1b[2m"))(s)
  } else {
    s.to_string()
  }
}

pub fn italic(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[3m", "\x1b[23m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn underline(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[4m", "\x1b[24m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn inverse(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[7m", "\x1b[27m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn hidden(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[8m", "\x1b[28m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn strikethrough(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[9m", "\x1b[29m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn debug_color(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[38;2;255;140;0m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn brand_color(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[38;2;113;26;95m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

// black
pub fn black(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[38;2;0;0;0m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn red(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[38;2;219;90;107m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn green(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[32m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn yellow(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[33m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn blue(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[38;2;68;206;246m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn magenta(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[38;2;180;0;100m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn purple(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[38;2;140;67;86m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn orange(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[38;2;255;137;54m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn cyan(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[36m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn white(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[37m", "\x1b[39m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn bg_black(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[40m", "\x1b[49m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn bg_red(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[41m", "\x1b[49m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn bg_green(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[42m", "\x1b[49m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn bg_yellow(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[43m", "\x1b[49m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn bg_blue(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[44m", "\x1b[49m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn bg_magenta(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[45m", "\x1b[49m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn bg_cyan(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[46m", "\x1b[49m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn bg_white(s: &str) -> String {
  if is_color_enabled() {
    create_formatter("\x1b[47m", "\x1b[49m", None)(s)
  } else {
    s.to_string()
  }
}

pub fn gradient_string(text: &str, colors: &[[u8; 3]]) -> String {
  let steps = text.len();
  let gradient = colors
    .iter()
    .map(|color| format!("\x1b[38;2;{};{};{}m", color[0], color[1], color[2]))
    .collect::<Vec<_>>();

  let mut output = String::new();

  for (i, c) in text.chars().enumerate() {
    let color_index = ((i as f64) / (steps as f64) * (colors.len() as f64 - 1.0)).floor() as usize;
    output += &format!("{}{}", gradient[color_index], c);
  }

  output += "\x1b[0m";
  output
}

pub fn interpolate_color(color1: &[u8; 3], color2: &[u8; 3], factor: f64) -> [u8; 3] {
  [
    (color1[0] as f64 + (color2[0] as f64 - color1[0] as f64) * factor).round() as u8,
    (color1[1] as f64 + (color2[1] as f64 - color1[1] as f64) * factor).round() as u8,
    (color1[2] as f64 + (color2[2] as f64 - color1[2] as f64) * factor).round() as u8,
  ]
}

pub fn persistent_cache_brand() -> String {
  let gradient_string = gradient_string(
    "FULL EXTREME!",
    &[
      GRADIENT_PURPLE_COLOR,
      interpolate_color(&GRADIENT_PURPLE_COLOR, &GRADIENT_PINK_COLOR, 0.1),
      interpolate_color(&GRADIENT_PURPLE_COLOR, &GRADIENT_PINK_COLOR, 0.2),
      interpolate_color(&GRADIENT_PURPLE_COLOR, &GRADIENT_PINK_COLOR, 0.3),
      interpolate_color(&GRADIENT_PURPLE_COLOR, &GRADIENT_PINK_COLOR, 0.4),
      interpolate_color(&GRADIENT_PURPLE_COLOR, &GRADIENT_PINK_COLOR, 0.5),
      interpolate_color(&GRADIENT_PURPLE_COLOR, &GRADIENT_PINK_COLOR, 0.6),
      interpolate_color(&GRADIENT_PURPLE_COLOR, &GRADIENT_PINK_COLOR, 0.7),
      interpolate_color(&GRADIENT_PURPLE_COLOR, &GRADIENT_PINK_COLOR, 0.8),
      interpolate_color(&GRADIENT_PURPLE_COLOR, &GRADIENT_PINK_COLOR, 0.9),
      GRADIENT_PINK_COLOR,
    ],
  );
  format!("{}{}{}", brand_color("⚡️"), gradient_string, reset(""))
}

pub fn handle_brand_text(text: &str) {
  let gradient_string = gradient_string(
    text,
    &[
      BRAND_GRADIENT_COLORS,
      interpolate_color(&BRAND_GRADIENT_COLORS, &BRAND_GRADIENT_COLORS2, 0.2),
      interpolate_color(&BRAND_GRADIENT_COLORS, &BRAND_GRADIENT_COLORS2, 0.4),
      interpolate_color(&BRAND_GRADIENT_COLORS, &BRAND_GRADIENT_COLORS2, 0.6),
      interpolate_color(&BRAND_GRADIENT_COLORS, &BRAND_GRADIENT_COLORS2, 0.8),
      BRAND_GRADIENT_COLORS2,
    ],
  );
  println!("{gradient_string}");
}

pub fn brand_text(text: &str) -> String {
  let gradient_string = gradient_string(
    &format!("\n{text} \n"),
    &[
      BRAND_GRADIENT_COLORS,
      interpolate_color(&BRAND_GRADIENT_COLORS, &BRAND_GRADIENT_COLORS2, 0.2),
      interpolate_color(&BRAND_GRADIENT_COLORS, &BRAND_GRADIENT_COLORS2, 0.4),
      interpolate_color(&BRAND_GRADIENT_COLORS, &BRAND_GRADIENT_COLORS2, 0.6),
      interpolate_color(&BRAND_GRADIENT_COLORS, &BRAND_GRADIENT_COLORS2, 0.8),
      BRAND_GRADIENT_COLORS2,
    ],
  );
  gradient_string
}

pub struct Colors {
  pub reset: StylerEnabled,
  pub bold: StylerEnabled,
  pub dim: StylerEnabled,
  pub italic: StylerEnabled,
  pub underline: StylerEnabled,
  pub inverse: StylerEnabled,
  pub hidden: StylerEnabled,
  pub strikethrough: StylerEnabled,
  pub black: StylerEnabled,
  pub red: StylerEnabled,
  pub green: StylerEnabled,
  pub yellow: StylerEnabled,
  pub blue: StylerEnabled,
  pub magenta: StylerEnabled,
  pub purple: StylerEnabled,
  pub orange: StylerEnabled,
  pub cyan: StylerEnabled,
  pub white: StylerEnabled,
  pub bg_black: StylerEnabled,
  pub bg_red: StylerEnabled,
  pub bg_green: StylerEnabled,
  pub bg_yellow: StylerEnabled,
  pub bg_blue: StylerEnabled,
  pub bg_magenta: StylerEnabled,
  pub bg_cyan: StylerEnabled,
  pub bg_white: StylerEnabled,
  pub debug_color: StylerEnabled,
  pub brand_color: StylerEnabled,
  pub handle_brand_text: fn(&str),
  pub brand_text: fn(&str) -> String,
}
