//! Repair positions after known Rust preprocessing, retaining authored sources.
use similar::{ChangeTag, TextDiff};
use sourcemap::{SourceMap, SourceMapBuilder};
use std::{collections::HashMap, path::Path};

#[derive(Clone, Debug)]
pub(crate) struct SourceRecord {
  pub original: String,
  pub compiled: String,
  pub synthetic_prefix: usize,
}

pub(crate) fn filename(name: &str) -> String {
  Path::new(name)
    .canonicalize()
    .unwrap_or_else(|_| name.into())
    .to_string_lossy()
    .replace('\\', "/")
}

/// Character boundaries in transformed text -> corresponding original bytes.
struct Offsets(Vec<(usize, usize)>);
impl Offsets {
  fn new(original: &str, transformed: &str) -> Self {
    let (mut old, mut new) = (0, 0);
    let mut points = vec![(0, 0)];
    for change in TextDiff::from_chars(original, transformed).iter_all_changes() {
      for character in change.value().chars() {
        match change.tag() {
          ChangeTag::Equal => {
            points.push((new, old));
            old += character.len_utf8();
            new += character.len_utf8();
          }
          ChangeTag::Delete => {
            old += character.len_utf8();
          }
          ChangeTag::Insert => {
            points.push((new, old));
            new += character.len_utf8();
          }
        }
      }
      points.push((new, old));
    }
    Self(points)
  }
  fn original(&self, transformed: usize) -> usize {
    let index = self
      .0
      .partition_point(|(position, _)| *position <= transformed);
    self.0[index.saturating_sub(1)].1
  }
}

fn byte_position(text: &str, line: u32, column: u32) -> Result<usize, String> {
  let start = if line == 0 {
    0
  } else {
    text
      .match_indices('\n')
      .nth(line as usize - 1)
      .map(|(offset, _)| offset + 1)
      .ok_or("source map line outside source")?
  };
  let mut utf16 = 0;
  for (offset, character) in text[start..].char_indices() {
    if utf16 == column {
      return Ok(start + offset);
    }
    if character == '\n' {
      break;
    }
    utf16 += character.len_utf16() as u32;
  }
  if utf16 == column {
    Ok(text.len())
  } else {
    Err("source map column outside source".into())
  }
}
fn position(text: &str, offset: usize) -> Result<(u32, u32), String> {
  let prefix = text
    .get(..offset)
    .ok_or("source map offset outside source")?;
  Ok((
    prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
    prefix
      .rsplit('\n')
      .next()
      .unwrap_or("")
      .encode_utf16()
      .count() as u32,
  ))
}

pub(crate) fn repair(
  map: &str,
  before_css: &str,
  after_css: &str,
  records: &HashMap<String, SourceRecord>,
) -> Result<String, String> {
  let map = SourceMap::from_slice(map.as_bytes()).map_err(|error| error.to_string())?;
  // Invert the transformation: original=final CSS, transformed=old CSS.
  let generated = Offsets::new(after_css, before_css);
  let mut source_offsets = HashMap::new();
  for (name, source) in records {
    source_offsets.insert(
      name,
      Offsets::new(
        &source.original,
        &source.compiled[source.synthetic_prefix..],
      ),
    );
  }
  let mut builder = SourceMapBuilder::new(None);
  for token in map.tokens() {
    let old_offset = byte_position(before_css, token.get_dst_line(), token.get_dst_col())?;
    let (line, column) = position(after_css, generated.original(old_offset))?;
    let Some(source_name) = token.get_source() else {
      builder.add(line, column, 0, 0, None, None, false);
      continue;
    };
    if let Some(record) = records.get(&filename(source_name)) {
      let offset = byte_position(&record.compiled, token.get_src_line(), token.get_src_col())?;
      if offset < record.synthetic_prefix {
        builder.add(line, column, 0, 0, None, None, false);
        continue;
      }
      let source_offset =
        source_offsets[&filename(source_name)].original(offset - record.synthetic_prefix);
      let (source_line, source_column) = position(&record.original, source_offset)?;
      let added = builder.add(
        line,
        column,
        source_line,
        source_column,
        Some(source_name),
        token.get_name(),
        false,
      );
      builder.set_source_contents(added.src_id, Some(&record.original));
    } else {
      let added = builder.add(
        line,
        column,
        token.get_src_line(),
        token.get_src_col(),
        Some(source_name),
        token.get_name(),
        false,
      );
      builder.set_source_contents(added.src_id, map.get_source_contents(token.get_src_id()));
    }
  }
  let mut bytes = vec![];
  builder
    .into_sourcemap()
    .to_writer(&mut bytes)
    .map_err(|error| error.to_string())?;
  String::from_utf8(bytes).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn repairs_prelude_url_edits_and_unicode_in_both_directions() {
    let original = ".🌱 { background: url('x'); color: red; }\n";
    let transformed = original.replace("'x'", "'../long-path/x'");
    let prelude = "$injected: 1;\n";
    let compiled = format!("{prelude}{transformed}");
    let before_css = ".🌱 { background: url('../long-path/x'); color: red; }\n";
    let after_css = before_css.replace("'../long-path/x'", "'/assets/x.hash.svg'");
    let source = filename("test.scss");
    let mut builder = SourceMapBuilder::new(None);
    builder.add(0, 0, 0, 0, Some(&source), None, false);
    let (dst_line, dst_col) = position(before_css, before_css.find("color").unwrap()).unwrap();
    let (src_line, src_col) = position(&compiled, compiled.find("color").unwrap()).unwrap();
    builder.add(
      dst_line,
      dst_col,
      src_line,
      src_col,
      Some(&source),
      None,
      false,
    );
    let mut bytes = vec![];
    builder.into_sourcemap().to_writer(&mut bytes).unwrap();
    let records = HashMap::from([(
      source,
      SourceRecord {
        original: original.into(),
        compiled,
        synthetic_prefix: prelude.len(),
      },
    )]);
    let repaired = repair(
      std::str::from_utf8(&bytes).unwrap(),
      before_css,
      &after_css,
      &records,
    )
    .unwrap();
    let map = SourceMap::from_slice(repaired.as_bytes()).unwrap();
    let (line, col) = position(&after_css, after_css.find("color").unwrap()).unwrap();
    let token = map.lookup_token(line, col).unwrap();
    assert!(map.lookup_token(0, 0).unwrap().get_source().is_none());
    assert_eq!(
      (token.get_src_line(), token.get_src_col()),
      position(original, original.find("color").unwrap()).unwrap()
    );
    assert_eq!(map.get_source_contents(0), Some(original));
  }
}
