//! Original character provenance through Rust import inlining and URL edits.
use similar::{ChangeTag, TextDiff};
use std::{collections::HashSet, ops::Range, sync::Arc};

#[derive(Clone)]
struct Source {
  name: String,
  text: String,
  lines: Vec<usize>,
}

impl Source {
  fn new(name: &str, text: &str) -> Self {
    Self {
      name: name.replace('\\', "/"),
      text: text.into(),
      lines: lines(text),
    }
  }

  fn position(&self, offset: usize) -> (u32, u32) {
    position(&self.text, &self.lines, offset)
  }
}

#[derive(Clone)]
struct Marker {
  generated: usize,
  original: Option<(Arc<Source>, usize)>,
}

pub(crate) struct MappedCss {
  pub text: String,
  markers: Option<Vec<Marker>>,
}

impl MappedCss {
  pub fn new(text: &str, name: Option<&str>) -> Self {
    let source = name.map(|name| Arc::new(Source::new(name, text)));
    let markers = source.map(|source| {
      text
        .char_indices()
        .map(|(offset, _)| Marker {
          generated: offset,
          original: Some((source.clone(), offset)),
        })
        .collect()
    });
    Self {
      text: text.into(),
      markers,
    }
  }

  pub fn empty(&self) -> Self {
    Self {
      text: String::new(),
      markers: self.markers.as_ref().map(|_| Vec::new()),
    }
  }

  pub fn is_mapped(&self) -> bool {
    self.markers.is_some()
  }

  pub fn append_slice(&mut self, source: &Self, range: Range<usize>) {
    let start = self.text.len();
    self.text.push_str(&source.text[range.clone()]);
    if let Some(markers) = &mut self.markers {
      if let Some(input) = &source.markers {
        let first = input.partition_point(|marker| marker.generated < range.start);
        let end = input.partition_point(|marker| marker.generated < range.end);
        markers.extend(input[first..end].iter().cloned().map(|mut marker| {
          marker.generated = start + marker.generated - range.start;
          marker
        }));
      } else {
        markers.extend(source.text[range].char_indices().map(|(offset, _)| Marker {
          generated: start + offset,
          original: None,
        }));
      }
    }
  }

  pub fn append(&mut self, source: &Self) {
    self.append_slice(source, 0..source.text.len());
  }

  /// Equal characters retain their actual source; inserted characters are
  /// explicitly unmapped instead of pretending to have authored positions.
  pub fn rewritten(self, text: String) -> Self {
    let Some(markers) = self.markers else {
      return Self {
        text,
        markers: None,
      };
    };
    let mut result = Vec::new();
    let (mut old, mut new) = (0, 0);
    for change in TextDiff::from_chars(&self.text, &text).iter_all_changes() {
      for character in change.value().chars() {
        match change.tag() {
          ChangeTag::Equal => {
            let index = markers.partition_point(|marker| marker.generated < old);
            let original = markers
              .get(index)
              .filter(|marker| marker.generated == old)
              .and_then(|marker| marker.original.clone());
            result.push(Marker {
              generated: new,
              original,
            });
            old += character.len_utf8();
            new += character.len_utf8();
          }
          ChangeTag::Delete => old += character.len_utf8(),
          ChangeTag::Insert => {
            result.push(Marker {
              generated: new,
              original: None,
            });
            new += character.len_utf8();
          }
        }
      }
    }
    Self {
      text,
      markers: Some(result),
    }
  }

  pub fn source_map(&self) -> Option<sourcemap::SourceMap> {
    let markers = self.markers.as_ref()?;
    let mut builder = sourcemap::SourceMapBuilder::new(None);
    let generated_lines = lines(&self.text);
    let mut registered = HashSet::new();
    for marker in markers {
      let (line, col) = position(&self.text, &generated_lines, marker.generated);
      if let Some((source, offset)) = &marker.original {
        if registered.insert(source.name.clone()) {
          let id = builder.add_source(&source.name);
          builder.set_source_contents(id, Some(&source.text));
        }
        let (source_line, source_col) = source.position(*offset);
        builder.add(
          line,
          col,
          source_line,
          source_col,
          Some(&source.name),
          None,
          false,
        );
      } else {
        builder.add(line, col, 0, 0, None, None, false);
      }
    }
    Some(builder.into_sourcemap())
  }
}

fn lines(text: &str) -> Vec<usize> {
  std::iter::once(0)
    .chain(
      text
        .bytes()
        .enumerate()
        .filter_map(|(offset, byte)| (byte == b'\n').then_some(offset + 1)),
    )
    .collect()
}

fn position(text: &str, lines: &[usize], offset: usize) -> (u32, u32) {
  let line = lines.partition_point(|start| *start <= offset) - 1;
  (
    line as u32,
    text[lines[line]..offset].encode_utf16().count() as u32,
  )
}
