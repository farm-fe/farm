use crate::constants::{CHAR_BACKWARD_SLASH, CHAR_FORWARD_SLASH};

pub fn is_path_separator(ch: char) -> bool {
  ch == CHAR_FORWARD_SLASH || ch == CHAR_BACKWARD_SLASH
}

pub fn remove_backslashes(input: &str) -> String {
  let chars: Vec<char> = input.chars().collect();
  let mut result = String::with_capacity(input.len());
  let mut index = 0;

  while index < chars.len() {
    let ch = chars[index];

    if ch == '[' {
      result.push(ch);
      index += 1;

      while index < chars.len() {
        let current = chars[index];
        result.push(current);

        if current == CHAR_BACKWARD_SLASH && index + 1 < chars.len() {
          index += 1;
          result.push(chars[index]);
        } else if current == ']' {
          index += 1;
          break;
        }

        index += 1;
      }

      continue;
    }

    if ch == CHAR_BACKWARD_SLASH && index + 1 < chars.len() {
      if chars[index + 1] == '[' {
        index += 1;
        continue;
      }

      index += 1;
      result.push(chars[index]);
      index += 1;
      continue;
    }

    result.push(ch);
    index += 1;
  }

  result
}
