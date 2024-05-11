#![allow(unused)]

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
