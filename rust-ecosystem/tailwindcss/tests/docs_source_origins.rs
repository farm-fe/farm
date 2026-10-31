use farmfe_ecosystem_tailwindcss::{
  ast::{AstNode, to_css},
  parser::{parse, parse_with_origins},
};

fn collect<'a>(nodes: &'a [AstNode], output: &mut Vec<(&'a str, usize)>) {
  for node in nodes {
    match node {
      AstNode::Context(context) => {
        let offset = context.context["farm:source-offset"].parse().unwrap();
        let label = match &context.nodes[0] {
          AstNode::Rule(rule) => rule.selector.as_str(),
          AstNode::AtRule(rule) => rule.name.as_str(),
          AstNode::Declaration(decl) => decl.property.as_str(),
          AstNode::Comment(comment) => comment.as_str(),
          _ => panic!("origin should wrap a concrete CSS node"),
        };
        output.push((label, offset));
        collect(&context.nodes, output);
      }
      AstNode::Rule(rule) => collect(&rule.nodes, output),
      AstNode::AtRule(rule) => collect(&rule.nodes, output),
      _ => {}
    }
  }
}

#[test]
fn parser_records_original_byte_offsets_without_changing_css() {
  let source = "/*! license */\r\n/* skipped */\r\n@media screen {\n .中文😀 { color: red; color: blue;\n @apply flex; content: '😀'; padding: 0 }\n}\n@tailwind utilities;";
  let nodes = parse_with_origins(source);
  assert_eq!(to_css(&nodes), to_css(&parse(source)));
  let mut origins = Vec::new();
  collect(&nodes, &mut origins);
  let expected = [
    (" license ", 0),
    ("@media", source.find("@media").unwrap()),
    (".中文😀", source.find(".中文😀").unwrap()),
    ("color", source.find("color: red").unwrap()),
    ("color", source.find("color: blue").unwrap()),
    ("@apply", source.find("@apply").unwrap()),
    ("content", source.find("content").unwrap()),
    ("padding", source.find("padding").unwrap()),
    ("@tailwind", source.find("@tailwind").unwrap()),
  ];
  assert_eq!(origins, expected);
}

#[test]
fn comments_and_strings_do_not_create_fake_origins() {
  let source = "/* .fake { color: blue; } */\n.real { content: '} ; /* 😀 */'; color: red }";
  let nodes = parse_with_origins(source);
  let mut origins = Vec::new();
  collect(&nodes, &mut origins);
  assert_eq!(
    origins,
    vec![
      (".real", source.find(".real").unwrap()),
      ("content", source.find("content").unwrap()),
      ("color", source.find("color: red").unwrap()),
    ]
  );
  assert_eq!(to_css(&nodes), to_css(&parse(source)));
}
