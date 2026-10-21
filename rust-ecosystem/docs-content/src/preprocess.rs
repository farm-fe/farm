use mdxjs::hast::{self, Node, PropertyValue};
use serde_json::{Map, Value};
use std::borrow::Cow;
use std::collections::HashMap;

pub struct Directive {
  start: usize,
  content_start: usize,
  end: usize,
  finish: usize,
  kind: String,
  title: String,
  implicit_title: bool,
  fence_length: usize,
  position: markdown::unist::Position,
  empty: bool,
}

pub struct Prepared {
  pub source: String,
  pub frontmatter: Value,
  pub metadata_locations: super::metadata::MetadataLocations,
  pub directives: Vec<Directive>,
  pub heading_ids: HashMap<usize, String>,
}

pub(crate) fn mask(bytes: &mut [u8]) {
  for byte in bytes {
    if *byte != b'\n' && *byte != b'\r' {
      *byte = b' ';
    }
  }
}

/// Mask extension delimiters without shifting source offsets or line numbers.
pub fn prepare(
  source: &str,
  allow_html_comments: bool,
  commonmark: bool,
  file: Option<&str>,
) -> Result<Prepared, String> {
  let lines: Vec<_> = source.split_inclusive('\n').collect();
  let mut bytes = source.as_bytes().to_vec();
  let mut frontmatter = Value::Object(Map::new());
  let mut metadata_locations = super::metadata::MetadataLocations::default();
  let mut first = 0;
  let mut offset = 0;
  if lines.first().is_some_and(|line| line.trim_end() == "---") {
    let closing = lines
      .iter()
      .enumerate()
      .skip(1)
      .find(|(_, line)| matches!(line.trim_end(), "---" | "..."))
      .map(|(index, _)| index)
      .ok_or_else(|| super::metadata::diagnostic(file, 1, 1, "missing closing delimiter"))?;
    frontmatter = serde_yaml::from_str::<Value>(&lines[1..closing].concat()).map_err(|error| {
      if let Some(location) = error.location() {
        let line = location.line() + 1;
        let snippet = lines
          .get(line - 1)
          .map(|line| line.trim_end())
          .unwrap_or("");
        let column = snippet
          .chars()
          .take(location.column() - 1)
          .map(char::len_utf16)
          .sum::<usize>()
          + 1;
        super::metadata::diagnostic(file, line, column, format!("{error}\n{snippet}"))
      } else {
        super::metadata::diagnostic(file, 2, 1, error)
      }
    })?;
    if frontmatter.is_null() {
      frontmatter = Value::Object(Map::new());
    }
    if !frontmatter.is_object() {
      return Err(super::metadata::diagnostic(
        file,
        2,
        1,
        "expected a YAML mapping",
      ));
    }
    metadata_locations = super::metadata::locations(&lines[1..closing].concat(), file)?;
    offset = lines[..=closing].iter().map(|line| line.len()).sum();
    mask(&mut bytes[..offset]);
    first = closing + 1;
  }
  let mut stack: Vec<Directive> = vec![];
  let mut directives = vec![];
  let mut directive_markers = vec![];
  let mut heading_ids = HashMap::new();
  let mut fence: Option<(char, usize)> = None;
  let mut comments = super::comments::Comments::default();
  for (index, line) in lines.iter().enumerate().skip(first) {
    let indentation = line
      .chars()
      .take_while(|ch| *ch == ' ' || *ch == '\t')
      .fold(0, |column, ch| {
        if ch == '\t' {
          (column / 4 + 1) * 4
        } else {
          column + 1
        }
      });
    if commonmark && fence.is_none() && !comments.in_comment && indentation >= 4 {
      offset += line.len();
      comments.reset_inline();
      continue;
    }
    let trimmed = line.trim_start();
    let fence_character = trimmed
      .chars()
      .next()
      .filter(|character| *character == '`' || *character == '~');
    if let Some(character) = fence_character {
      let length = trimmed
        .chars()
        .take_while(|candidate| *candidate == character)
        .count();
      if length >= 3 && !comments.in_comment {
        match fence {
          None => fence = Some((character, length)),
          Some((open, count))
            if open == character && length >= count && trimmed[length..].trim().is_empty() =>
          {
            fence = None
          }
          _ => {}
        }
        offset += line.len();
        comments.reset_inline();
        continue;
      }
    }
    if fence.is_none() {
      let visible: Cow<'_, str> = if allow_html_comments
        && comments.mask_line(line, &mut bytes[offset..offset + line.len()])
      {
        Cow::Owned(
          String::from_utf8(bytes[offset..offset + line.len()].to_vec())
            .map_err(|error| error.to_string())?,
        )
      } else {
        Cow::Borrowed(line)
      };
      let trimmed = visible.trim_start();
      let heading = trimmed.trim_end();
      let depth = heading
        .chars()
        .take_while(|character| *character == '#')
        .count();
      if (1..=6).contains(&depth)
        && heading[depth..].starts_with(char::is_whitespace)
        && heading.ends_with('}')
      {
        if let Some(start) = heading.rfind(" {#") {
          let id = &heading[start + 3..heading.len() - 1];
          if id.is_empty()
            || !id.chars().all(|character| {
              character.is_alphanumeric() || matches!(character, '-' | '_' | ':' | '.')
            })
          {
            return Err(format!(
              "[DOCS_HEADING_ID] invalid heading ID at line {}",
              index + 1
            ));
          }
          let indentation = line.len() - trimmed.len();
          heading_ids.insert(offset + indentation, id.to_string());
          mask(&mut bytes[offset + indentation + start..offset + indentation + heading.len()]);
        }
      }
      let delimiter = trimmed.trim_end();
      let fence_length = delimiter.bytes().take_while(|byte| *byte == b':').count();
      if fence_length >= 3 {
        let marker = &delimiter[fence_length..];
        if marker.is_empty() {
          let mut directive = stack.pop().ok_or_else(|| {
            format!(
              "[DOCS_ADMONITION] unexpected closing delimiter at line {}",
              index + 1
            )
          })?;
          if fence_length < directive.fence_length {
            return Err(format!(
              "[DOCS_ADMONITION] closing fence shorter than opening fence at line {}",
              index + 1
            ));
          }
          directive.end = offset;
          directive.finish = offset + line.len();
          directive.position.end = if line.ends_with('\n') {
            markdown::unist::Point {
              line: index + 2,
              column: 1,
              offset: directive.finish,
            }
          } else {
            markdown::unist::Point {
              line: index + 1,
              column: line.chars().count() + 1,
              offset: directive.finish,
            }
          };
          directives.push(directive);
        } else {
          let kind_end = marker
            .find(|character: char| character.is_whitespace() || character == '{')
            .unwrap_or(marker.len());
          let kind = &marker[..kind_end];
          if !matches!(
            kind,
            "note" | "tip" | "info" | "warning" | "danger" | "caution" | "important" | "success"
          ) {
            return Err(format!(
              "[DOCS_ADMONITION] unknown type {kind} at line {}",
              index + 1
            ));
          }
          let rest = marker[kind_end..].trim();
          let title = if rest.is_empty() {
            kind.to_string()
          } else if rest.starts_with('{') {
            let value = rest
              .strip_prefix("{title=")
              .and_then(|value| value.strip_suffix('}'))
              .ok_or_else(|| format!("[DOCS_ADMONITION] malformed title at line {}", index + 1))?;
            if value.starts_with('"') {
              serde_json::from_str::<String>(value)
                .map_err(|error| format!("[DOCS_ADMONITION] {error}"))?
            } else {
              value.trim_matches('\'').to_string()
            }
          } else {
            rest.to_string()
          };
          stack.push(Directive {
            start: offset,
            content_start: offset + line.len(),
            end: 0,
            finish: 0,
            kind: kind.into(),
            title,
            implicit_title: rest.is_empty(),
            fence_length,
            position: markdown::unist::Position {
              start: markdown::unist::Point {
                line: index + 1,
                column: 1,
                offset,
              },
              end: markdown::unist::Point {
                line: index + 1,
                column: 1,
                offset,
              },
            },
            empty: false,
          });
        }
        directive_markers.push(offset + visible.len() - trimmed.len());
        mask(&mut bytes[offset..offset + line.len()]);
      }
    }
    offset += line.len();
  }
  if let Some(open) = stack.last() {
    return Err(format!(
      "[DOCS_ADMONITION] unclosed {} block at byte {}",
      open.kind, open.start
    ));
  }
  directives.sort_by_key(|directive| std::cmp::Reverse(directive.start));
  for directive in &mut directives {
    directive.empty = bytes[directive.content_start..directive.end]
      .iter()
      .all(u8::is_ascii_whitespace);
  }
  // A blank delimiter lets Markdown merge lists and paragraphs across the
  // block boundary. A temporary thematic break interrupts both without moving
  // any source offsets; its HAST node is removed before wrapping the block.
  let mut raw_html_ranges = vec![];
  if commonmark && !directive_markers.is_empty() {
    fn collect(node: &markdown::mdast::Node, ranges: &mut Vec<(usize, usize)>) {
      if let markdown::mdast::Node::Html(html) = node {
        if let Some(position) = &html.position {
          ranges.push((position.start.offset, position.end.offset));
        }
      } else if let Some(children) = node.children() {
        for child in children {
          collect(child, ranges);
        }
      }
    }
    let original = markdown::to_mdast(source, &markdown::ParseOptions::gfm())
      .map_err(|error| error.to_string())?;
    let mut original_raw_ranges = vec![];
    collect(&original, &mut original_raw_ranges);
    for directive in &directives {
      if original_raw_ranges
        .iter()
        .any(|(start, end)| *start <= directive.start && directive.finish.saturating_sub(1) <= *end)
      {
        bytes[directive.start..directive.finish]
          .copy_from_slice(&source.as_bytes()[directive.start..directive.finish]);
        directive_markers.retain(|marker| *marker < directive.start || *marker >= directive.finish);
      }
    }
    let tree = markdown::to_mdast(
      std::str::from_utf8(&bytes).map_err(|error| error.to_string())?,
      &markdown::ParseOptions::gfm(),
    )
    .map_err(|error| error.to_string())?;
    collect(&tree, &mut raw_html_ranges);
  }
  for marker in directive_markers {
    // Raw HTML is terminated by the blank delimiter itself. Replacing that
    // blank with a break would extend the HTML block into following content.
    if raw_html_ranges.iter().any(|(start, end)| {
      (*start <= marker && marker < *end)
        || (*end <= marker && bytes[*end..marker].iter().all(u8::is_ascii_whitespace))
    }) {
      continue;
    }
    bytes[marker..marker + 3].copy_from_slice(b"***");
  }
  Ok(Prepared {
    source: String::from_utf8(bytes).map_err(|error| error.to_string())?,
    frontmatter,
    metadata_locations,
    directives,
    heading_ids,
  })
}

/// CommonMark raw HTML retains literal Markdown rather than parsing extensions.
/// Restore authored bytes before the HTML bridge sees temporary boundaries.
pub fn restore_raw_html(
  node: &mut markdown::mdast::Node,
  source: &str,
  directives: &mut Vec<Directive>,
) -> Result<(), String> {
  if let markdown::mdast::Node::Html(html) = node {
    if let Some(position) = &html.position {
      html.value = source
        .get(position.start.offset..position.end.offset)
        .ok_or("[DOCS_HTML] invalid raw HTML source range")?
        .to_string();
      directives.retain(|directive| {
        directive.start < position.start.offset || directive.finish > position.end.offset
      });
    }
  } else if let Some(children) = node.children_mut() {
    for child in children {
      restore_raw_html(child, source, directives)?;
    }
  }
  Ok(())
}

pub(crate) fn default_admonitions(directives: &[Directive]) -> HashMap<usize, String> {
  directives
    .iter()
    .filter(|directive| directive.implicit_title)
    .map(|directive| (directive.position.start.offset, directive.kind.clone()))
    .collect()
}

/// Match compiler-owned wrappers by original directive position, not title text.
/// Author-provided titles (even ones equal to a kind) never enter `defaults`.
pub(crate) fn apply_admonition_labels(
  node: &mut Node,
  defaults: &HashMap<usize, String>,
  labels: &std::collections::BTreeMap<String, String>,
) {
  if let Node::Element(element) = node
    && element.tag_name == "aside"
    && let Some(kind) = element
      .position
      .as_ref()
      .and_then(|position| defaults.get(&position.start.offset))
    && let Some(label) = labels.get(kind)
    && let Some(Node::Element(title)) = element.children.first_mut()
    && title.properties.iter().any(|(key, value)| {
      key == "className"
        && matches!(value, PropertyValue::String(value) if value == "farm-admonition-title")
    })
  {
    if label.is_empty() {
      element.children.remove(0);
    } else {
      title.children = vec![Node::Text(hast::Text {
        value: label.clone(),
        position: None,
      })];
    }
  }
  if let Some(children) = node.children_mut() {
    for child in children {
      apply_admonition_labels(child, defaults, labels);
    }
  }
}

pub fn wrap_admonitions(node: &mut Node, directives: &[Directive]) {
  let root = matches!(node, Node::Root(_));
  if let Some(children) = node.children_mut() {
    children.retain(|child| {
      !matches!(child, Node::Element(element) if element.tag_name == "hr"
      && element.position.as_ref().is_some_and(|position| directives.iter().any(|directive| {
        (position.start.offset >= directive.start && position.end.offset <= directive.content_start)
          || (position.start.offset >= directive.end && position.end.offset <= directive.finish)
      })))
    });
    for child in children {
      wrap_admonitions(child, directives);
    }
  }
  let container = matches!(node, Node::Root(_))
    || matches!(node, Node::Element(element) if matches!(element.tag_name.as_str(), "li" | "blockquote" | "div"));
  if !container {
    return;
  }
  let container_position = node.position().cloned();
  let Some(children) = node.children_mut() else {
    return;
  };
  for directive in directives {
    // A container wholly inside this directive is content of the outer block,
    // not another owner of it. Explicit inner directives still have their own
    // ranges and are wrapped independently before their enclosing block.
    // Markdown list positions may include the masked closing fence's trailing
    // spaces. Those bytes still belong to this block, through `finish`.
    if container_position.as_ref().is_some_and(|position| {
      position.start.offset >= directive.content_start && position.end.offset <= directive.finish
    }) {
      continue;
    }
    let indices: Vec<_> = children
      .iter()
      .enumerate()
      .filter_map(|(index, child)| {
        child
          .position()
          .filter(|position| {
            position.start.offset >= directive.content_start
              && position.end.offset <= directive.finish
          })
          .map(|_| index)
      })
      .collect();
    let (first, mut content) = match (indices.first(), indices.last()) {
      (Some(first), Some(last)) => (*first, children.drain(*first..=*last).collect::<Vec<_>>()),
      _ if root && directive.empty => {
        let first = children
          .iter()
          .position(|child| {
            child
              .position()
              .is_some_and(|p| p.start.offset >= directive.finish)
          })
          .unwrap_or(children.len());
        (first, vec![])
      }
      _ => continue,
    };
    let position = Some(directive.position.clone());
    if !directive.title.is_empty() {
      content.insert(
        0,
        crate::element(
          "p",
          vec![(
            "className".into(),
            PropertyValue::String("farm-admonition-title".into()),
          )],
          vec![Node::Text(hast::Text {
            value: directive.title.clone(),
            position: None,
          })],
        ),
      );
    }
    let mut block = crate::element(
      "aside",
      vec![
        (
          "className".into(),
          PropertyValue::String(format!(
            "farm-admonition farm-admonition-{}",
            directive.kind
          )),
        ),
        (
          "data-type".into(),
          PropertyValue::String(directive.kind.clone()),
        ),
      ],
      content,
    );
    if let Node::Element(element) = &mut block {
      element.position = position;
    }
    children.insert(first, block);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn block_structure(source: &str) -> Vec<(String, String, usize)> {
    let parsed = crate::parse_content(
      source,
      &Default::default(),
      None,
      crate::ContentFormat::Markdown,
    )
    .unwrap();
    fn walk(
      node: &Node,
      parent: &str,
      enclosing: usize,
      result: &mut Vec<(String, String, usize)>,
    ) {
      let mut enclosing = enclosing;
      let tag = if let Node::Element(element) = node {
        if element.tag_name == "aside" {
          let kind = element
            .properties
            .iter()
            .find_map(|(name, value)| {
              if name == "data-type" {
                if let PropertyValue::String(value) = value {
                  return Some(value.clone());
                }
              }
              None
            })
            .unwrap();
          result.push((kind, parent.to_string(), enclosing));
          enclosing += 1;
        }
        element.tag_name.as_str()
      } else {
        "root"
      };
      if let Some(children) = node.children() {
        for child in children {
          walk(child, tag, enclosing, result);
        }
      }
    }
    let mut result = vec![];
    walk(&parsed.tree, "", 0, &mut result);
    result
  }

  #[test]
  fn explicit_list_directive_stays_inside_its_item_and_outer_admonition() {
    let source = "::::warning Outer\n\n- First item.\n\n  :::note Inner\n  Nested body.\n  :::\n\n- Second item.\n\n::::\n";
    assert_eq!(
      block_structure(source),
      vec![
        ("warning".into(), "root".into(), 0),
        ("note".into(), "li".into(), 1),
      ]
    );
  }

  #[test]
  fn a_list_after_the_closing_fence_stays_outside_the_admonition() {
    let parsed = crate::parse_content(
      ":::note Title\n- inside\n:::\n- outside\n",
      &Default::default(),
      None,
      crate::ContentFormat::Markdown,
    )
    .unwrap();
    let Node::Root(root) = &parsed.tree else {
      panic!("expected root");
    };
    let elements: Vec<_> = root
      .children
      .iter()
      .filter_map(|node| match node {
        Node::Element(element) => Some(element),
        _ => None,
      })
      .collect();
    assert_eq!(
      elements
        .iter()
        .map(|element| element.tag_name.as_str())
        .collect::<Vec<_>>(),
      vec!["aside", "ul"],
      "closing fence separates inner and following lists"
    );
    assert!(format!("{:?}", elements[0]).contains("inside"));
    assert!(!format!("{:?}", elements[0]).contains("outside"));
    assert!(format!("{:?}", elements[1]).contains("outside"));
    assert!(!format!("{:?}", elements[1]).contains("inside"));
  }

  #[test]
  fn directive_boundaries_interrupt_blocks_in_markdown_and_mdx() {
    for format in [crate::ContentFormat::Markdown, crate::ContentFormat::Mdx] {
      for (source, expected) in [
        (
          "before\n:::note Title\ninside\n:::\nafter\n",
          vec!["p", "aside", "p"],
        ),
        (
          "- before\n:::note Title\n- inside\n:::\n- after\n",
          vec!["ul", "aside", "ul"],
        ),
        (":::note Title\ninside\n\n***\n\nmore\n:::\n", vec!["aside"]),
      ] {
        let parsed = crate::parse_content(source, &Default::default(), None, format).unwrap();
        let Node::Root(root) = &parsed.tree else {
          panic!("expected root")
        };
        let elements: Vec<_> = root
          .children
          .iter()
          .filter_map(|node| match node {
            Node::Element(element) => Some(element),
            _ => None,
          })
          .collect();
        assert_eq!(
          elements
            .iter()
            .map(|element| element.tag_name.as_str())
            .collect::<Vec<_>>(),
          expected,
          "{source}"
        );
        let aside = elements
          .iter()
          .find(|element| element.tag_name == "aside")
          .unwrap();
        let body = format!("{aside:?}");
        assert!(body.contains("inside"));
        assert!(!body.contains("before") && !body.contains("after"));
        assert_eq!(
          body.matches("tag_name: \"hr\"").count(),
          usize::from(source.contains("***"))
        );
      }
    }
  }

  #[test]
  fn markdown_raw_html_keeps_literal_directives_without_internal_markers() {
    for tag in ["div", "pre"] {
      let source = format!("<{tag}>\n:::note Title\ninside\n:::\n</{tag}>\n");
      let parsed = crate::parse_content(
        &source,
        &Default::default(),
        None,
        crate::ContentFormat::Markdown,
      )
      .unwrap();
      let tree = format!("{:?}", parsed.tree);
      assert!(!tree.contains("***"), "temporary marker leaked: {tree}");
      assert!(
        tree.contains(":::note Title"),
        "raw HTML retains literal Markdown: {tree}"
      );
      assert!(!tree.contains("tag_name: \"aside\""));
    }
  }

  #[test]
  fn markdown_admonition_around_raw_html_keeps_following_text_outside() {
    let source = ":::note Title\n<div>\ninside\n</div>\n:::\nafter\n";
    let parsed = crate::parse_content(
      source,
      &Default::default(),
      None,
      crate::ContentFormat::Markdown,
    )
    .unwrap();
    let Node::Root(root) = &parsed.tree else {
      panic!("expected root")
    };
    let elements: Vec<_> = root
      .children
      .iter()
      .filter_map(|node| match node {
        Node::Element(element) => Some(element),
        _ => None,
      })
      .collect();
    assert_eq!(
      elements
        .iter()
        .map(|element| element.tag_name.as_str())
        .collect::<Vec<_>>(),
      vec!["aside", "p"]
    );
    let aside = format!("{:?}", elements[0]);
    assert!(aside.contains("inside"));
    assert!(!aside.contains("after") && !aside.contains(":::"));
    assert!(format!("{:?}", elements[1]).contains("after"));
  }

  #[test]
  fn adjacent_empty_blocks_do_not_become_nested_or_swallow_each_other() {
    assert_eq!(
      block_structure(":::note First\n:::\n\n:::tip Second\n:::"),
      vec![
        ("note".into(), "root".into(), 0),
        ("tip".into(), "root".into(), 0),
      ]
    );
  }
}
