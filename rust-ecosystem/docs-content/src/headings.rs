use mdxjs::hast::{Node, PropertyValue};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Serialize)]
pub struct Heading {
  pub id: String,
  pub title: String,
  pub depth: u8,
}

pub fn annotate(
  tree: &mut Node,
  explicit: &HashMap<usize, String>,
) -> Result<Vec<Heading>, String> {
  let mut reserved = HashSet::new();
  for id in explicit.values() {
    if !reserved.insert(id.clone()) {
      return Err(format!("[DOCS_HEADING_ID] duplicate explicit ID {id}"));
    }
  }
  let mut headings = vec![];
  visit(
    tree,
    &mut headings,
    &mut HashMap::new(),
    &mut HashSet::new(),
    explicit,
    &reserved,
  );
  Ok(headings)
}

fn visit(
  node: &mut Node,
  headings: &mut Vec<Heading>,
  counters: &mut HashMap<String, usize>,
  used: &mut HashSet<String>,
  explicit: &HashMap<usize, String>,
  reserved: &HashSet<String>,
) {
  if let Node::Element(element) = node {
    if let Some(depth) = element
      .tag_name
      .strip_prefix('h')
      .and_then(|value| value.parse::<u8>().ok())
      .filter(|value| (1..=6).contains(value))
    {
      let title = element
        .children
        .iter()
        .map(ToString::to_string)
        .collect::<String>()
        .trim()
        .to_string();
      let explicit_id = element
        .position
        .as_ref()
        .and_then(|position| explicit.get(&position.start.offset));
      let id = if let Some(value) = explicit_id {
        value.clone()
      } else {
        let mut base = String::new();
        for character in title.to_lowercase().chars() {
          if character.is_alphanumeric() || character == '-' || character == '_' {
            base.push(character);
          } else if character.is_whitespace() && !base.ends_with('-') {
            base.push('-');
          }
        }
        base = base.trim_matches('-').to_string();
        if base.is_empty() {
          base = "section".into();
        }
        let count = counters.entry(base.clone()).or_default();
        let mut id = if *count == 0 {
          base.clone()
        } else {
          format!("{base}-{count}")
        };
        while used.contains(&id) || reserved.contains(&id) {
          *count += 1;
          id = format!("{base}-{count}");
        }
        *count += 1;
        id
      };
      used.insert(id.clone());
      element
        .properties
        .push(("id".into(), PropertyValue::String(id.clone())));
      headings.push(Heading { id, title, depth });
    }
  }
  if let Some(children) = node.children_mut() {
    for child in children {
      visit(child, headings, counters, used, explicit, reserved);
    }
  }
}
