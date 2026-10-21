//! Reparse raw CommonMark HTML with HTML5 rules while retaining structured
//! Markdown nodes and their source positions. No Markdown is rendered twice.
use markdown::mdast;
use mdxjs::hast::{self, Node};
use swc_common::{FileName, SourceMap, Span, DUMMY_SP};
use swc_html_ast::{Child, DocumentMode, Element, Namespace};

const RAW: &str = "farm-docs-private-raw";

struct Origin {
  start: usize,
  end: usize,
  source: usize,
}
struct Positions<'a> {
  origins: &'a [Origin],
  base: u32,
  location: markdown::Location,
}
impl Positions<'_> {
  fn position(&self, span: Span) -> Option<markdown::unist::Position> {
    let mapped = |offset: u32| {
      let offset = offset.checked_sub(self.base)? as usize;
      self
        .origins
        .iter()
        .find(|origin| offset >= origin.start && offset <= origin.end)
        .map(|origin| origin.source + offset - origin.start)
    };
    let start = mapped(span.lo.0)?;
    let end = mapped(span.hi.0).unwrap_or(start).max(start);
    Some(markdown::unist::Position {
      start: self.location.to_point(start)?,
      end: self.location.to_point(end)?,
    })
  }
}

fn escape(value: &str) -> String {
  value
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
}

fn mark_raw(node: &mut mdast::Node, raw: &mut Vec<String>) {
  if let mdast::Node::Html(html) = node {
    let index = raw.len();
    raw.push(html.value.clone());
    *node = mdast::Node::MdxJsxTextElement(mdast::MdxJsxTextElement {
      name: Some(RAW.into()),
      children: vec![],
      position: html.position.clone(),
      attributes: vec![mdast::AttributeContent::Property(mdast::MdxJsxAttribute {
        name: "index".into(),
        value: Some(mdast::AttributeValue::Literal(index.to_string())),
      })],
    });
    return;
  }
  if let Some(children) = node.children_mut() {
    for child in children {
      mark_raw(child, raw);
    }
  }
}

fn void(name: &str) -> bool {
  matches!(
    name,
    "area"
      | "base"
      | "br"
      | "col"
      | "embed"
      | "hr"
      | "img"
      | "input"
      | "link"
      | "meta"
      | "param"
      | "source"
      | "track"
      | "wbr"
  )
}

fn serialize(
  node: &Node,
  raw: &[String],
  marker: &str,
  preserved: &mut Vec<hast::Element>,
  output: &mut String,
  origins: &mut Vec<Origin>,
) -> Result<(), String> {
  match node {
    Node::Root(root) => {
      for child in &root.children {
        serialize(child, raw, marker, preserved, output, origins)?;
      }
    }
    Node::Text(text) => output.push_str(&escape(&text.value)),
    Node::Element(element) => {
      let index = preserved.len();
      preserved.push(element.clone());
      output.push_str(&format!("<{} {marker}=\"{index}\">", element.tag_name));
      for child in &element.children {
        serialize(child, raw, marker, preserved, output, origins)?;
      }
      if !void(&element.tag_name) {
        output.push_str(&format!("</{}>", element.tag_name));
      }
    }
    Node::MdxJsxElement(element) if element.name.as_deref() == Some(RAW) => {
      let index = element
        .attributes
        .first()
        .and_then(|attribute| match attribute {
          mdast::AttributeContent::Property(attribute) => match &attribute.value {
            Some(mdast::AttributeValue::Literal(value)) => value.parse::<usize>().ok(),
            _ => None,
          },
          _ => None,
        })
        .and_then(|index| raw.get(index))
        .ok_or("[DOCS_HTML] invalid internal raw node")?;
      let start = output.len();
      output.push_str(index);
      if let Some(position) = &element.position {
        origins.push(Origin {
          start,
          end: output.len(),
          source: position.start.offset,
        });
      }
    }
    Node::Comment(_) | Node::Doctype(_) => {}
    _ => return Err("[DOCS_HTML] unexpected MDX node in Markdown tree".into()),
  }
  Ok(())
}

fn style(value: &str) -> Result<String, String> {
  let mut input = cssparser::ParserInput::new(value);
  let mut parser = cssparser::Parser::new(&mut input);
  let mut properties = serde_json::Map::new();
  while !parser.is_exhausted() {
    let declaration: Result<(String, String), cssparser::ParseError<'_, ()>> = parser
      .parse_until_before(cssparser::Delimiter::Semicolon, |parser| {
        let name = parser.expect_ident_cloned()?.to_string();
        parser.expect_colon()?;
        let start = parser.position();
        while !parser.is_exhausted() {
          parser.next_including_whitespace_and_comments()?;
        }
        Ok((name, parser.slice_from(start).trim().to_string()))
      });
    let (name, value) = declaration.map_err(|error| format!("[DOCS_HTML_STYLE] {error:?}"))?;
    let name = if name.starts_with("--") {
      name
    } else {
      let mut uppercase = false;
      let mut result = String::new();
      for character in name.chars() {
        if character == '-' {
          uppercase = true;
        } else if uppercase {
          result.extend(character.to_uppercase());
          uppercase = false;
        } else {
          result.push(character);
        }
      }
      if let Some(suffix) = result.strip_prefix("Ms") {
        format!("ms{suffix}")
      } else {
        result
      }
    };
    properties.insert(name, value.into());
    let _ = parser.try_parse(|parser| parser.expect_semicolon());
  }
  Ok(serde_json::Value::Object(properties).to_string())
}

fn react_name(name: &str) -> &str {
  match name {
    "class" => "className",
    "for" => "htmlFor",
    "tabindex" => "tabIndex",
    "readonly" => "readOnly",
    "colspan" => "colSpan",
    "rowspan" => "rowSpan",
    "srcset" => "srcSet",
    "crossorigin" => "crossOrigin",
    "maxlength" => "maxLength",
    "autofocus" => "autoFocus",
    "contenteditable" => "contentEditable",
    "checked" => "defaultChecked",
    other => other,
  }
}

fn convert(
  child: Child,
  marker: &str,
  preserved: &[hast::Element],
  positions: &Positions<'_>,
) -> Result<Option<Node>, String> {
  match child {
    Child::Text(text) => Ok(Some(Node::Text(hast::Text {
      value: text.data.to_string(),
      position: positions.position(text.span),
    }))),
    Child::Element(element) => {
      let original = element
        .attributes
        .iter()
        .find(|attribute| attribute.name.as_ref() == marker)
        .and_then(|attribute| attribute.value.as_ref())
        .and_then(|value| value.parse::<usize>().ok())
        .and_then(|index| preserved.get(index));
      let mut children = vec![];
      let source_children = element
        .content
        .map(|content| content.children)
        .unwrap_or(element.children);
      for child in source_children {
        if let Some(child) = convert(child, marker, preserved, positions)? {
          children.push(child);
        }
      }
      if let Some(original) = original {
        let mut original = original.clone();
        original.children = children;
        return Ok(Some(Node::Element(original)));
      }
      let mut attributes = vec![];
      for attribute in element.attributes {
        let name = attribute.name.to_string();
        if name.starts_with("on") {
          return Err(format!(
            "[DOCS_HTML_ATTRIBUTE] inline event attribute {name} is not supported in React content"
          ));
        }
        let value = if name == "style" {
          mdast::AttributeValue::Expression(mdast::AttributeValueExpression {
            value: style(attribute.value.as_deref().unwrap_or(""))?,
            stops: vec![],
          })
        } else if matches!(
          name.as_str(),
          "disabled"
            | "checked"
            | "selected"
            | "multiple"
            | "readonly"
            | "required"
            | "autofocus"
            | "hidden"
            | "open"
            | "controls"
            | "loop"
            | "muted"
            | "autoplay"
        ) {
          mdast::AttributeValue::Expression(mdast::AttributeValueExpression {
            value: "true".into(),
            stops: vec![],
          })
        } else {
          mdast::AttributeValue::Literal(
            attribute
              .value
              .map(|value| value.to_string())
              .unwrap_or_default(),
          )
        };
        attributes.push(mdast::AttributeContent::Property(mdast::MdxJsxAttribute {
          name: react_name(&name).into(),
          value: Some(value),
        }));
      }
      Ok(Some(Node::MdxJsxElement(hast::MdxJsxElement {
        name: Some(element.tag_name.to_string()),
        attributes,
        children,
        position: positions.position(element.span),
      })))
    }
    Child::Comment(_) | Child::DocumentType(_) => Ok(None),
  }
}

pub fn to_hast(mdast: &mdast::Node, source: &str) -> Result<Node, String> {
  let mut mdast = mdast.clone();
  let mut raw = vec![];
  mark_raw(&mut mdast, &mut raw);
  let tree = mdxjs::mdast_util_to_hast(&mdast);
  if raw.is_empty() {
    return Ok(tree);
  }
  // The marker cannot collide with authored HTML attributes.
  let mut marker = "data-farm-docs-preserved".to_string();
  while source.to_ascii_lowercase().contains(&marker) {
    marker.push('x');
  }
  let mut preserved = vec![];
  let mut html = String::new();
  let mut origins = vec![];
  serialize(
    &tree,
    &raw,
    &marker,
    &mut preserved,
    &mut html,
    &mut origins,
  )?;
  let cm = SourceMap::default();
  let file = cm.new_source_file(FileName::Custom("markdown-html".into()).into(), html);
  let context = Element {
    span: DUMMY_SP,
    tag_name: "body".into(),
    namespace: Namespace::HTML,
    attributes: vec![],
    children: vec![],
    content: None,
    is_self_closing: false,
  };
  let fragment = swc_html_parser::parse_file_as_document_fragment(
    &file,
    &context,
    DocumentMode::NoQuirks,
    None,
    Default::default(),
    &mut vec![],
  )
  .map_err(|error| format!("[DOCS_HTML] {error:?}"))?;
  let mut children = vec![];
  let positions = Positions {
    origins: &origins,
    base: file.start_pos.0,
    location: markdown::Location::new(source.as_bytes()),
  };
  for child in fragment.children {
    if let Some(child) = convert(child, &marker, &preserved, &positions)? {
      children.push(child);
    }
  }
  Ok(Node::Root(hast::Root {
    children,
    position: tree.position().cloned(),
  }))
}
