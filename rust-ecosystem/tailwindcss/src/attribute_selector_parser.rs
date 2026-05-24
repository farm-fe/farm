//! Attribute-selector parser, ported from
//! `packages/tailwindcss/src/attribute-selector-parser.ts`.

const TAB: u8 = 9;
const LINE_BREAK: u8 = 10;
const CARRIAGE_RETURN: u8 = 13;
const SPACE: u8 = 32;
const DOUBLE_QUOTE: u8 = 34;
const DOLLAR: u8 = 36;
const SINGLE_QUOTE: u8 = 39;
const ASTERISK: u8 = 42;
const EQUALS: u8 = 61;
const UPPER_I: u8 = 73;
const UPPER_S: u8 = 83;
const BACKSLASH: u8 = 92;
const CARET: u8 = 94;
const LOWER_I: u8 = 105;
const LOWER_S: u8 = 115;
const PIPE: u8 = 124;
const TILDE: u8 = 126;
const LOWER_A: u8 = 97;
const LOWER_Z: u8 = 122;
const UPPER_A: u8 = 65;
const UPPER_Z: u8 = 90;
const ZERO: u8 = 48;
const NINE: u8 = 57;
const DASH: u8 = 45;
const UNDERSCORE: u8 = 95;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeOperator {
  Equals,     // =
  Includes,   // ~=
  DashMatch,  // |=
  StartsWith, // ^=
  EndsWith,   // $=
  Contains,   // *=
}

impl AttributeOperator {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Equals => "=",
      Self::Includes => "~=",
      Self::DashMatch => "|=",
      Self::StartsWith => "^=",
      Self::EndsWith => "$=",
      Self::Contains => "*=",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeQuote {
  Double, // "
  Single, // '
}

impl AttributeQuote {
  pub fn as_char(&self) -> char {
    match self {
      Self::Double => '"',
      Self::Single => '\'',
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeSensitivity {
  Insensitive, // i
  Sensitive,   // s
}

impl AttributeSensitivity {
  pub fn as_char(&self) -> char {
    match self {
      Self::Insensitive => 'i',
      Self::Sensitive => 's',
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSelector {
  pub attribute: String,
  pub operator: Option<AttributeOperator>,
  pub quote: Option<AttributeQuote>,
  pub value: Option<String>,
  pub sensitivity: Option<AttributeSensitivity>,
}

fn is_ws(b: u8) -> bool {
  matches!(b, SPACE | TAB | LINE_BREAK | CARRIAGE_RETURN)
}

/// Parse a CSS attribute selector. Returns `None` for invalid input.
pub fn parse(input: &str) -> Option<AttributeSelector> {
  let bytes = input.as_bytes();
  let len = bytes.len();
  if len < 2 || bytes[0] != b'[' || bytes[len - 1] != b']' {
    return None;
  }

  let mut i = 1usize;
  let end = len - 1;

  while i < end && is_ws(bytes[i]) {
    i += 1;
  }

  // Attribute name
  let start = i;
  while i < end {
    let c = bytes[i];
    if c == BACKSLASH {
      i += 2;
      continue;
    }
    if (UPPER_A..=UPPER_Z).contains(&c) {
      i += 1;
      continue;
    }
    if (LOWER_A..=LOWER_Z).contains(&c) {
      i += 1;
      continue;
    }
    if (ZERO..=NINE).contains(&c) {
      i += 1;
      continue;
    }
    if c == DASH || c == UNDERSCORE {
      i += 1;
      continue;
    }
    break;
  }
  if start == i {
    return None;
  }
  let attribute = std::str::from_utf8(&bytes[start..i]).ok()?.to_string();

  while i < end && is_ws(bytes[i]) {
    i += 1;
  }

  if i == end {
    return Some(AttributeSelector {
      attribute,
      operator: None,
      quote: None,
      value: None,
      sensitivity: None,
    });
  }

  // Operator
  let current = bytes[i];
  let operator;
  if current == EQUALS {
    operator = AttributeOperator::Equals;
    i += 1;
  } else if matches!(current, TILDE | PIPE | CARET | DOLLAR | ASTERISK)
    && bytes.get(i + 1).copied() == Some(EQUALS)
  {
    operator = match current {
      TILDE => AttributeOperator::Includes,
      PIPE => AttributeOperator::DashMatch,
      CARET => AttributeOperator::StartsWith,
      DOLLAR => AttributeOperator::EndsWith,
      ASTERISK => AttributeOperator::Contains,
      _ => unreachable!(),
    };
    i += 2;
  } else {
    return None;
  }

  while i < end && is_ws(bytes[i]) {
    i += 1;
  }

  if i == end {
    return None;
  }

  let mut quote: Option<AttributeQuote> = None;
  let value: String;
  let current = bytes[i];
  if current == SINGLE_QUOTE || current == DOUBLE_QUOTE {
    quote = Some(if current == DOUBLE_QUOTE {
      AttributeQuote::Double
    } else {
      AttributeQuote::Single
    });
    i += 1;
    let vstart = i;
    let mut j = i;
    while j < end {
      let cj = bytes[j];
      if cj == current {
        i = j + 1;
      } else if cj == BACKSLASH {
        j += 2;
        continue;
      }
      j += 1;
    }
    // i was advanced to one past the last matching closing quote; if none,
    // i remains at vstart and slice is empty.
    value = if i > vstart {
      std::str::from_utf8(&bytes[vstart..i - 1]).ok()?.to_string()
    } else {
      String::new()
    };
  } else {
    let vstart = i;
    while i < end && !is_ws(bytes[i]) {
      i += 1;
    }
    value = std::str::from_utf8(&bytes[vstart..i]).ok()?.to_string();
  }

  while i < end && is_ws(bytes[i]) {
    i += 1;
  }

  if i == end {
    return Some(AttributeSelector {
      attribute,
      operator: Some(operator),
      quote,
      value: Some(value),
      sensitivity: None,
    });
  }

  let sensitivity = match bytes[i] {
    LOWER_I | UPPER_I => {
      i += 1;
      AttributeSensitivity::Insensitive
    }
    LOWER_S | UPPER_S => {
      i += 1;
      AttributeSensitivity::Sensitive
    }
    _ => return None,
  };

  while i < end && is_ws(bytes[i]) {
    i += 1;
  }

  if i != end {
    return None;
  }

  Some(AttributeSelector {
    attribute,
    operator: Some(operator),
    quote,
    value: Some(value),
    sensitivity: Some(sensitivity),
  })
}
