use markdown::{Location, message::Message};
use mdxjs::{Options, hast::Node};

/// Compile plain MDX using the upstream stages without Docs semantics.
pub fn compile_mdx(source: &str, options: &Options) -> Result<String, Message> {
  let mdast = mdxjs::mdast_util_from_mdx(source, options)?;
  let mut tree = mdxjs::mdast_util_to_hast(&mdast);
  remove_generated_table_whitespace(&mut tree);
  let location = Location::new(source.as_bytes());
  let mut explicit_jsxs = Default::default();
  let mut program = mdxjs::hast_util_to_swc(&tree, options, Some(&location), &mut explicit_jsxs)?;
  mdxjs::mdx_plugin_recma_document(&mut program, options, Some(&location))?;
  mdxjs::mdx_plugin_recma_jsx_rewrite(&mut program, options, Some(&location), &explicit_jsxs)?;
  Ok(program.serialize())
}

/// mdxjs inserts position-less line feeds between generated GFM table nodes.
/// React must not receive these as text children of table structure elements.
/// Authored JSX/expressions and positioned text are not compiler whitespace.
pub(crate) fn remove_generated_table_whitespace(node: &mut Node) {
  if let Node::Element(element) = node
    && matches!(
      element.tag_name.as_str(),
      "table" | "thead" | "tbody" | "tfoot" | "tr" | "colgroup"
    )
  {
    element.children.retain(|child| !matches!(child, Node::Text(text)
      if text.position.is_none() && text.value.bytes().all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0c))));
  }
  if let Some(children) = node.children_mut() {
    for child in children {
      remove_generated_table_whitespace(child);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use markdown::unist::Position;
  use mdxjs::hast::{Element, Text};

  fn text(value: &str, authored: bool) -> Node {
    Node::Text(Text {
      value: value.into(),
      position: authored.then(|| Position::new(1, 1, 0, 1, 2, 1)),
    })
  }
  fn element(tag: &str, children: Vec<Node>) -> Node {
    Node::Element(Element {
      tag_name: tag.into(),
      properties: vec![],
      children,
      position: None,
    })
  }

  #[test]
  fn only_generated_html_whitespace_under_table_structure_is_removed() {
    for tag in ["table", "thead", "tbody", "tfoot", "tr", "colgroup"] {
      let retained = vec![
        text(" \n", true),
        text("content", false),
        text("\u{a0}", false),
        element("td", vec![text(" \n", false)]),
      ];
      let mut children = vec![text(" \t\r\n\u{c}", false)];
      children.extend(retained.clone());
      let mut tree = element(tag, children);
      remove_generated_table_whitespace(&mut tree);
      assert_eq!(tree.children().unwrap(), &retained, "{tag}");
    }
    for tag in ["td", "th", "span", "code", "pre", "div", "Table"] {
      let mut tree = element(tag, vec![text(" \n", false)]);
      let before = tree.clone();
      remove_generated_table_whitespace(&mut tree);
      assert_eq!(tree, before, "{tag}");
    }
  }

  #[test]
  fn gfm_tables_compile_without_structural_text_in_plain_and_docs_modes() {
    for source in [
      "| Header |\n| --- |\n",
      "| Left | Right |\n| :--- | ---: |\n| **bold** | ` a b ` |\n| | last |\n",
    ] {
      for jsx in [false, true] {
        for development in [false, true] {
          let options = Options {
            jsx,
            development,
            filepath: Some("table.mdx".into()),
            ..Default::default()
          };
          let code = compile_mdx(source, &options).unwrap();
          for content in if source.contains("bold") {
            vec!["Left", "Right", "bold", "a b", "last"]
          } else {
            vec!["Header"]
          } {
            assert!(code.contains(content), "lost {content}: {code}");
          }
          assert!(
            !code.contains("\"\\n\""),
            "plain {jsx}/{development}: {code}"
          );
          for format in [crate::ContentFormat::Mdx, crate::ContentFormat::Markdown] {
            let docs =
              crate::compile_content_with_format(source, &options, true, None, format).unwrap();
            assert!(
              !docs.code.contains("\"\\n\""),
              "docs {jsx}/{development}: {}",
              docs.code
            );
            let map: serde_json::Value =
              serde_json::from_str(docs.source_map.as_ref().unwrap()).unwrap();
            assert!(
              map["sourcesContent"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str() == Some(source))
            );
          }
        }
      }
    }
  }

  #[test]
  fn authored_jsx_expressions_and_non_table_content_keep_upstream_semantics() {
    for source in [
      "<table><tbody><tr><td>{'  '}<code> x y </code></td></tr></tbody></table>",
      "<table>{'\\n'}<caption>Caption</caption></table>",
      "# Title\n\nA *word* and ` x y `.\n\n```text\n  preserved\n```\n",
      "export const value = 1;\n\n<Widget value={value} />",
    ] {
      for jsx in [false, true] {
        for development in [false, true] {
          let options = Options {
            jsx,
            development,
            provider_import_source: Some("@mdx-js/react".into()),
            filepath: Some("authored.mdx".into()),
            ..Default::default()
          };
          assert_eq!(
            compile_mdx(source, &options).unwrap(),
            mdxjs::compile(source, &options).unwrap()
          );
        }
      }
    }
  }

  #[test]
  fn classic_runtime_and_disabled_gfm_keep_the_requested_configuration() {
    let source = "| Value |\n| --- |\n| one |\n";
    let mut options = Options {
      jsx_runtime: Some(mdxjs::JsxRuntime::Classic),
      ..Default::default()
    };
    let code = compile_mdx(source, &options).unwrap();
    assert!(code.contains("React.createElement"));
    assert!(code.contains("one"));
    assert!(!code.contains("\"\\n\""));
    options.parse.constructs.gfm_table = false;
    assert_eq!(
      compile_mdx(source, &options).unwrap(),
      mdxjs::compile(source, &options).unwrap()
    );
  }
}
