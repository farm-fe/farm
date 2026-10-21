//! Preserve YAML node locations separately from its resolved JSON values.
//! serde_yaml remains the value authority; marked parser events never coerce values.
use std::collections::HashMap;
use yaml_rust::{
  parser::{Event, Parser},
  scanner::Marker,
};

#[derive(Clone, Copy, Debug)]
pub struct MetadataPosition {
  pub line: usize,
  pub column: usize,
}

#[derive(Clone, Default)]
pub struct MetadataLocations(HashMap<Vec<String>, MetadataPosition>);

impl MetadataLocations {
  /// Field paths are compiler-owned names such as authors[0].url, not YAML text searches.
  pub fn position(&self, field: &str) -> Option<MetadataPosition> {
    let mut path: Vec<String> = field
      .split(['.', '[', ']'])
      .filter(|part| !part.is_empty())
      .map(str::to_owned)
      .collect();
    loop {
      if let Some(position) = self.0.get(&path) {
        return Some(*position);
      }
      path.pop()?;
    }
  }
}

struct Anchor {
  scalar: Option<String>,
  descendants: Vec<Vec<String>>,
}
struct Reader<'a> {
  parser: Parser<std::str::Chars<'a>>,
  lines: Vec<&'a str>,
  file: Option<&'a str>,
  locations: MetadataLocations,
  anchors: HashMap<usize, Anchor>,
}
impl Reader<'_> {
  fn point(&self, mark: Marker) -> MetadataPosition {
    let column = self
      .lines
      .get(mark.line() - 1)
      .copied()
      .unwrap_or("")
      .chars()
      .take(mark.col())
      .map(char::len_utf16)
      .sum::<usize>()
      + 1;
    // The YAML payload starts after the opening frontmatter delimiter.
    MetadataPosition {
      line: mark.line() + 1,
      column,
    }
  }
  fn next(&mut self) -> Result<(Event, Marker), String> {
    self.parser.next().map_err(|error| {
      let position = self.point(*error.marker());
      diagnostic(self.file, position.line, position.column, error)
    })
  }
  fn node(
    &mut self,
    event: Event,
    mark: Marker,
    path: Vec<String>,
    depth: usize,
  ) -> Result<Option<String>, String> {
    if depth > 128 {
      return Err("[DOCS_FRONTMATTER] metadata nesting exceeds 128".into());
    }
    let point = self.point(mark);
    self.locations.0.insert(path.clone(), point);
    let (anchor, scalar) = match event {
      Event::Scalar(value, _, anchor, _) => (anchor, Some(value)),
      Event::Alias(id) => {
        let anchor = self
          .anchors
          .get(&id)
          .ok_or("[DOCS_FRONTMATTER] unresolved metadata alias")?;
        for suffix in &anchor.descendants {
          let mut child = path.clone();
          child.extend(suffix.iter().cloned());
          // Expanded alias values point to the use, not to an unrelated matching string.
          self.locations.0.insert(child, point);
        }
        return Ok(anchor.scalar.clone());
      }
      Event::SequenceStart(anchor) => {
        let mut index = 0;
        loop {
          let (event, mark) = self.next()?;
          if event == Event::SequenceEnd {
            break;
          }
          let mut child = path.clone();
          child.push(index.to_string());
          self.node(event, mark, child, depth + 1)?;
          index += 1;
        }
        (anchor, None)
      }
      Event::MappingStart(anchor) => {
        loop {
          let (event, key_mark) = self.next()?;
          if event == Event::MappingEnd {
            break;
          }
          let key = match event {
            Event::Scalar(key, _, id, _) => {
              if id != 0 {
                self.anchors.insert(
                  id,
                  Anchor {
                    scalar: Some(key.clone()),
                    descendants: vec![],
                  },
                );
              }
              key
            }
            Event::Alias(id) => self
              .anchors
              .get(&id)
              .and_then(|anchor| anchor.scalar.clone())
              .ok_or("[DOCS_FRONTMATTER] metadata keys must be strings")?,
            _ => {
              return Err(format!(
                "[DOCS_FRONTMATTER] non-scalar metadata key at line {}",
                key_mark.line() + 1
              ));
            }
          };
          let (event, mark) = self.next()?;
          let mut child = path.clone();
          child.push(key);
          self.node(event, mark, child, depth + 1)?;
        }
        (anchor, None)
      }
      _ => return Err("[DOCS_FRONTMATTER] unexpected metadata event".into()),
    };
    if anchor != 0 {
      let descendants = self
        .locations
        .0
        .keys()
        .filter_map(|child| child.strip_prefix(path.as_slice()).map(<[String]>::to_vec))
        .collect();
      self.anchors.insert(
        anchor,
        Anchor {
          scalar: scalar.clone(),
          descendants,
        },
      );
    }
    Ok(scalar)
  }
}

pub(crate) fn diagnostic(
  file: Option<&str>,
  line: usize,
  column: usize,
  message: impl std::fmt::Display,
) -> String {
  let file = file.unwrap_or("<content>");
  format!("[DOCS_FRONTMATTER] {file}:{line}:{column}: {message}")
}

pub(crate) fn locations<'a>(
  source: &'a str,
  file: Option<&'a str>,
) -> Result<MetadataLocations, String> {
  let mut reader = Reader {
    parser: Parser::new(source.chars()),
    lines: source.split('\n').collect(),
    file,
    locations: MetadataLocations::default(),
    anchors: HashMap::new(),
  };
  loop {
    let (event, mark) = reader.next()?;
    match event {
      Event::StreamStart | Event::DocumentStart => continue,
      Event::StreamEnd => break,
      _ => {
        reader.node(event, mark, vec![], 0)?;
        break;
      }
    }
  }
  Ok(reader.locations)
}
