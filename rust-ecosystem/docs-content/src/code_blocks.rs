use farmfe_syntax_highlight::{highlight, HighlightOptions};
use markdown::mdast::{
  self, AttributeContent, AttributeValue, AttributeValueExpression, MdxJsxAttribute,
};
use mdxjs::hast::{self, Node, PropertyValue};
use std::collections::{BTreeSet, HashMap};
#[derive(Default, Debug)]
struct Metadata {
  title: Option<String>,
  numbers: bool,
  lines: BTreeSet<usize>,
  unknown: Vec<String>,
}

fn raw_metadata<'a>(code: &mdast::Code, source: &'a str) -> Option<&'a str> {
  code.meta.as_ref()?;
  let start = code.position.as_ref()?.start.offset;
  let line = source
    .get(start..)?
    .split(['\n', '\r'])
    .next()?
    .trim_start_matches([' ', '\t']);
  let marker = *line.as_bytes().first()?;
  if marker != b'`' && marker != b'~' {
    return None;
  }
  let length = line.bytes().take_while(|byte| *byte == marker).count();
  if length < 3 {
    return None;
  }
  let info = line.get(length..)?.trim_start_matches([' ', '\t']);
  let language_end = info.find(char::is_whitespace)?;
  Some(info[language_end..].trim())
}

// Decode once after parsing the title so an entity containing a quote cannot
// change the metadata grammar. Keep JSON escapes independent of Markdown.
fn decode_title(input: &str) -> String {
  let mut result = String::new();
  let mut rest = input;
  while !rest.is_empty() {
    if let Some(value) = rest.strip_prefix('&') {
      if let Some(end) = value.find(';').filter(|end| *end <= 32) {
        let entity = &value[..end];
        let decoded = if let Some(number) = entity.strip_prefix('#') {
          let (digits, radix, max) = number
            .strip_prefix(['x', 'X'])
            .map_or((number, 10, 7), |digits| (digits, 16, 6));
          if !digits.is_empty()
            && digits.len() <= max
            && digits.bytes().all(|byte| match radix {
              16 => byte.is_ascii_hexdigit(),
              _ => byte.is_ascii_digit(),
            })
          {
            Some(markdown::decode_numeric(digits, radix))
          } else {
            None
          }
        } else {
          markdown::decode_named(entity, true)
        };
        if let Some(decoded) = decoded {
          result.push_str(&decoded);
          rest = &value[end + 1..];
          continue;
        }
      }
    }
    let ch = rest.chars().next().expect("nonempty title remainder");
    result.push(ch);
    rest = &rest[ch.len_utf8()..];
  }
  result
}

fn metadata(input: &str, line_count: usize) -> Result<Metadata, String> {
  let mut result = Metadata::default();
  let mut rest = input.trim();
  while !rest.is_empty() {
    if let Some(value) = rest.strip_prefix("title=\"") {
      let mut escaped = false;
      let end = value
        .char_indices()
        .find_map(|(i, ch)| {
          if escaped {
            escaped = false;
            None
          } else if ch == '\\' {
            escaped = true;
            None
          } else if ch == '"' {
            Some(i)
          } else {
            None
          }
        })
        .ok_or("unterminated code title")?;
      let quoted = &rest[6..7 + end + 1];
      result.title =
        Some(serde_json::from_str(quoted).map_err(|e| format!("invalid code title: {e}"))?);
      rest = value[end + 1..].trim_start();
    } else if let Some(value) = rest.strip_prefix("title=") {
      let end = value.find(char::is_whitespace).unwrap_or(value.len());
      let title = &value[..end];
      if title.is_empty() || title.contains(['\'', '"']) {
        return Err("invalid unquoted code title".into());
      }
      result.title = Some(title.into());
      rest = value[end..].trim_start();
    } else if let Some(value) = rest.strip_prefix('{') {
      let end = value.find('}').ok_or("unterminated code line range")?;
      result
        .lines
        .extend(farmfe_syntax_highlight::parse_line_ranges(
          &value[..end],
          line_count,
        )?);
      rest = value[end + 1..].trim_start();
    } else {
      let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
      let token = &rest[..end];
      if token == "showLineNumbers" {
        result.numbers = true;
      } else {
        result.unknown.push(token.into());
      }
      rest = rest[end..].trim_start();
    }
  }
  result.title = result.title.map(|title| decode_title(&title));
  Ok(result)
}

fn text(value: String) -> Node {
  Node::Text(hast::Text {
    value,
    position: None,
  })
}
fn property(name: &str, value: impl Into<String>) -> (String, PropertyValue) {
  (name.into(), PropertyValue::String(value.into()))
}

pub fn apply(
  mdast: &mdast::Node,
  tree: &mut Node,
  options: &HighlightOptions,
  source: &str,
) -> Result<Vec<String>, String> {
  fn collect<'a>(node: &'a mdast::Node, blocks: &mut HashMap<usize, &'a mdast::Code>) {
    if let mdast::Node::Code(code) = node {
      if let Some(position) = &code.position {
        blocks.insert(position.start.offset, code);
      }
    }
    if let Some(children) = node.children() {
      for child in children {
        collect(child, blocks);
      }
    }
  }
  fn rewrite(
    node: &mut Node,
    blocks: &HashMap<usize, &mdast::Code>,
    options: &HighlightOptions,
    source: &str,
    diagnostics: &mut Vec<String>,
  ) -> Result<(), String> {
    if matches!(node, Node::Element(element) if element.tag_name == "pre") {
      if let Some(code) = node.position().and_then(|p| blocks.get(&p.start.offset)) {
        let language = code.lang.as_deref().unwrap_or("text");
        // Match CommonMark's displayed terminal newline; code.value is the AST
        // semantic code string, never text extracted from generated markup.
        let raw = format!("{}\n", code.value);
        let highlighted = highlight(&raw, language, options)?;
        let position = code.position.as_ref().expect("collected positioned block");
        let location = format!("{}:{}", position.start.line, position.start.column);
        // Markdown's semantic meta has already unescaped JSON quotes. Read
        // this fence's info via its AST position, not by scanning MDX text.
        let metadata = metadata(
          raw_metadata(code, source)
            .or(code.meta.as_deref())
            .unwrap_or(""),
          highlighted.lines.len(),
        )
        .map_err(|e| format!("[DOCS_CODE_META] {location}: {e}"))?;
        if !highlighted.known_language {
          diagnostics.push(format!(
            "[DOCS_UNKNOWN_LANGUAGE] {location}: {language}; rendered as plain text"
          ));
        }
        for field in &metadata.unknown {
          diagnostics.push(format!(
            "[DOCS_CODE_META] {location}: unknown field {field}"
          ));
        }
        let mut lines = vec![];
        for (index, tokens) in highlighted.lines.into_iter().enumerate() {
          let children = tokens
            .into_iter()
            .map(|token| {
              let style =
                serde_json::json!({"--code-light": token.light, "--code-dark": token.dark})
                  .to_string();
              Node::MdxJsxElement(hast::MdxJsxElement {
                name: Some("span".into()),
                attributes: vec![
                  AttributeContent::Property(MdxJsxAttribute {
                    name: "className".into(),
                    value: Some(AttributeValue::Literal("farm-code-token".into())),
                  }),
                  AttributeContent::Property(MdxJsxAttribute {
                    name: "style".into(),
                    value: Some(AttributeValue::Expression(AttributeValueExpression {
                      value: style,
                      stops: vec![],
                    })),
                  }),
                ],
                children: vec![text(token.text)],
                position: None,
              })
            })
            .collect();
          let mut properties = vec![
            property("className", "farm-code-line"),
            property("data-line", (index + 1).to_string()),
          ];
          if metadata.lines.contains(&(index + 1)) {
            properties.push(property("data-highlighted", "true"));
          }
          lines.push(crate::element("span", properties, children));
        }
        let content = crate::element(
          "code",
          vec![property("className", format!("language-{language}"))],
          lines,
        );
        let mut properties = vec![
          property("className", "farm-code-block"),
          property("data-language", language),
          property("data-code", &raw),
        ];
        if let Some(title) = metadata.title {
          properties.push(property("data-title", title));
        }
        if metadata.numbers {
          properties.push(property("data-line-numbers", "true"));
        }
        if !highlighted.known_language {
          properties.push(property("data-unknown-language", "true"));
        }
        let mut replacement = crate::element("pre", properties, vec![content]);
        if let Node::Element(element) = &mut replacement {
          element.position = code.position.clone();
        }
        *node = replacement;
        return Ok(());
      }
    }
    if let Some(children) = node.children_mut() {
      for child in children {
        rewrite(child, blocks, options, source, diagnostics)?;
      }
    }
    Ok(())
  }
  let mut blocks = HashMap::new();
  collect(mdast, &mut blocks);
  let mut diagnostics = vec![];
  rewrite(tree, &blocks, options, source, &mut diagnostics)?;
  Ok(diagnostics)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn titles_decode_entities_once_without_changing_quote_boundaries() {
    let parsed = metadata(
      r#"title="A &amp; &#38; &#x26; &quot;B&quot; &amp;amp; &unknown; &#99999999;""#,
      1,
    )
    .unwrap();
    assert_eq!(
      parsed.title.as_deref(),
      Some("A & & & \"B\" &amp; &unknown; &#99999999;")
    );
    assert_eq!(
      metadata(r#"title="folder\\&name""#, 1)
        .unwrap()
        .title
        .as_deref(),
      Some(r"folder\&name")
    );
  }

  #[test]
  fn indented_literal_fence_has_no_metadata() {
    let source = "    ```ts title=wrong showLineNumbers\n";
    let parsed = markdown::to_mdast(source, &markdown::ParseOptions::gfm()).unwrap();
    let mdast::Node::Code(code) = &parsed.children().unwrap()[0] else {
      panic!("expected indented code")
    };
    assert!(code.meta.is_none());
    assert!(raw_metadata(code, source).is_none());
  }
  #[test]
  fn raw_fence_metadata_preserves_json_escapes_in_nested_blocks() {
    fn check(node: &mdast::Node, source: &str) -> usize {
      let mut found = 0;
      if let mdast::Node::Code(code) = node {
        let value = raw_metadata(code, source).expect("raw fence metadata");
        assert_eq!(
          metadata(value, 1).unwrap().title.as_deref(),
          Some("example \"<&>\".ts")
        );
        found += 1;
      }
      if let Some(children) = node.children() {
        for child in children {
          found += check(child, source);
        }
      }
      found
    }
    for prefix in ["", "> ", "  "] {
      let source =
        format!("{prefix}```ts title=\"example \\\"<&>\\\".ts\"\n{prefix}value\n{prefix}```\n");
      let parsed = markdown::to_mdast(&source, &markdown::ParseOptions::gfm()).unwrap();
      assert_eq!(check(&parsed, &source), 1);
    }
  }
  #[test]
  fn parses_titles_ranges_and_rejects_invalid_metadata() {
    let parsed = metadata("title=\"hello world.ts\" {1,3-4} showLineNumbers", 4).unwrap();
    assert_eq!(parsed.title.as_deref(), Some("hello world.ts"));
    assert_eq!(parsed.lines, BTreeSet::from([1, 3, 4]));
    assert!(parsed.numbers);
    assert_eq!(
      metadata("title=postcss.config.js", 1)
        .unwrap()
        .title
        .as_deref(),
      Some("postcss.config.js")
    );
    for input in ["{0}", "{4-2}", "{5}", "title=\"unterminated", "{1"] {
      assert!(metadata(input, 4).is_err(), "{input}");
    }
  }
}
