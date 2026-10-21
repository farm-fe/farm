//! Markdown HTML comments, excluding fenced code (handled by the caller) and code spans.
#[derive(Default)]
pub(crate) struct Comments {
  pub in_comment: bool,
  inline_ticks: Option<usize>,
}

impl Comments {
  pub fn reset_inline(&mut self) {
    self.inline_ticks = None;
  }

  pub fn mask_line(&mut self, source: &str, output: &mut [u8]) -> bool {
    if source.trim().is_empty() {
      self.reset_inline();
    }
    let bytes = source.as_bytes();
    let mut cursor = 0;
    let mut changed = false;
    while cursor < bytes.len() {
      if self.in_comment {
        let closing = bytes[cursor..]
          .windows(3)
          .position(|window| window == b"-->");
        let finish = closing.map_or(bytes.len(), |index| cursor + index + 3);
        super::preprocess::mask(&mut output[cursor..finish]);
        changed = true;
        cursor = finish;
        if closing.is_some() {
          self.in_comment = false;
        }
        continue;
      }
      if bytes[cursor] == b'`' {
        let count = bytes[cursor..]
          .iter()
          .take_while(|byte| **byte == b'`')
          .count();
        match self.inline_ticks {
          None => self.inline_ticks = Some(count),
          Some(open) if open == count => self.inline_ticks = None,
          _ => {}
        }
        cursor += count;
        continue;
      }
      if self.inline_ticks.is_none() {
        if bytes[cursor] == b'\\' {
          cursor = (cursor + 2).min(bytes.len());
          continue;
        }
        if bytes[cursor..].starts_with(b"<!--") {
          self.in_comment = true;
          continue;
        }
      }
      cursor += 1;
    }
    changed
  }
}
