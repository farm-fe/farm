use mdxjs::{
  Options,
  hast::{self, Node, PropertyValue},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceKind {
  Link,
  Image,
}

pub fn anchors(
  tree: &Node,
  options: &Options,
  source: &str,
) -> Result<std::collections::HashSet<String>, String> {
  fn visit(
    node: &Node,
    output: &mut std::collections::HashSet<String>,
    options: &Options,
    source: &str,
  ) -> Result<(), String> {
    let ids: Vec<&String> = match node {
      Node::Element(element) => element
        .properties
        .iter()
        .filter_map(|(key, value)| match (key.as_str(), value) {
          ("id", PropertyValue::String(value)) => Some(value),
          _ => None,
        })
        .collect(),
      Node::MdxJsxElement(element)
        if element
          .name
          .as_ref()
          .is_some_and(|name| name.chars().next().is_some_and(char::is_lowercase)) =>
      {
        element
          .attributes
          .iter()
          .filter_map(|attribute| match attribute {
            hast::AttributeContent::Property(attribute) if attribute.name == "id" => {
              match &attribute.value {
                Some(hast::AttributeValue::Literal(value)) => Some(value),
                _ => None,
              }
            }
            _ => None,
          })
          .collect()
      }
      _ => vec![],
    };
    for id in ids {
      if !output.insert(id.clone()) {
        let origin = reference(node, id, ReferenceKind::Link, options, source);
        return Err(format!(
          "[DOCS_ANCHOR_ID] {}:{}:{}: duplicate DOM id {id:?}",
          origin.file, origin.line, origin.column
        ));
      }
    }
    if let Some(children) = node.children() {
      for child in children {
        visit(child, output, options, source)?;
      }
    }
    Ok(())
  }
  let mut output = std::collections::HashSet::new();
  visit(tree, &mut output, options, source)?;
  Ok(output)
}
#[derive(Clone, Debug)]
pub struct ContentReference {
  pub kind: ReferenceKind,
  pub target: String,
  pub file: String,
  pub line: usize,
  pub column: usize,
  pub snippet: String,
}
fn reference(
  node: &Node,
  target: &str,
  kind: ReferenceKind,
  options: &Options,
  source: &str,
) -> ContentReference {
  let (line, column) = node
    .position()
    .map(|position| {
      (
        position.start.line,
        source
          .get(..position.start.offset)
          .map(|prefix| {
            prefix
              .rsplit(['\n', '\r'])
              .next()
              .unwrap_or("")
              .encode_utf16()
              .count()
              + 1
          })
          .unwrap_or(0),
      )
    })
    .unwrap_or((0, 0));
  ContentReference {
    kind,
    target: target.into(),
    file: options
      .filepath
      .clone()
      .unwrap_or_else(|| "document.mdx".into()),
    line,
    column,
    snippet: if line == 0 {
      String::new()
    } else {
      source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .into()
    },
  }
}
fn property(node: &Node) -> Option<(&str, ReferenceKind)> {
  let name = match node {
    Node::Element(element) => Some(element.tag_name.as_str()),
    Node::MdxJsxElement(element) => element.name.as_deref(),
    _ => None,
  }?;
  match name {
    "a" => Some(("href", ReferenceKind::Link)),
    "img" => Some(("src", ReferenceKind::Image)),
    _ => None,
  }
}
pub fn collect(tree: &Node, options: &Options, source: &str) -> Vec<ContentReference> {
  let mut output = vec![];
  fn visit(node: &Node, options: &Options, source: &str, output: &mut Vec<ContentReference>) {
    if let Some((name, kind)) = property(node) {
      match node {
        Node::Element(element) => {
          for (key, value) in &element.properties {
            if key == name {
              if let PropertyValue::String(value) = value {
                output.push(reference(node, value, kind, options, source));
              }
            }
          }
        }
        Node::MdxJsxElement(element) => {
          for attribute in &element.attributes {
            if let hast::AttributeContent::Property(attribute) = attribute {
              if attribute.name == name {
                if let Some(hast::AttributeValue::Literal(value)) = &attribute.value {
                  output.push(reference(node, value, kind, options, source));
                }
              }
            }
          }
        }
        _ => {}
      }
    }
    if let Some(children) = node.children() {
      for child in children {
        visit(child, options, source, output);
      }
    }
  }
  visit(tree, options, source, &mut output);
  output
}
pub fn rewrite(
  tree: &mut Node,
  options: &Options,
  source: &str,
  resolve: &mut impl FnMut(&ContentReference) -> Result<String, String>,
) -> Result<(), String> {
  if let Some((name, kind)) = property(tree) {
    let name = name.to_string();
    // Capture position before taking a mutable borrow of the property's value.
    let mut origin = reference(tree, "", kind, options, source);
    match tree {
      Node::Element(element) => {
        for (key, value) in &mut element.properties {
          if key == &name {
            if let PropertyValue::String(value) = value {
              origin.target = value.clone();
              *value = resolve(&origin)?;
            }
          }
        }
      }
      Node::MdxJsxElement(element) => {
        for attribute in &mut element.attributes {
          if let hast::AttributeContent::Property(attribute) = attribute {
            if attribute.name == name {
              if let Some(hast::AttributeValue::Literal(value)) = &mut attribute.value {
                origin.target = value.clone();
                *value = resolve(&origin)?;
              }
            }
          }
        }
      }
      _ => {}
    }
  }
  if let Some(children) = tree.children_mut() {
    for child in children {
      rewrite(child, options, source, resolve)?;
    }
  }
  Ok(())
}
