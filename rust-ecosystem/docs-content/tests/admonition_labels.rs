use farmfe_docs_content::{ContentFormat, emit_content, parse_content};
use std::collections::BTreeMap;

#[test]
fn localized_defaults_preserve_explicit_titles_empty_titles_and_source_maps() {
  let source = "# Labels\n\n:::note\nDefault body.\n\n:::tip\nNested body.\n:::\n:::\n\n:::note note\nExplicit body.\n:::\n\n:::note {title=\"\"}\nEmpty explicit.\n:::\n\n:::warning\nHidden default.\n:::\n\n:::info\nUnmapped default.\n:::\n";
  let labels = BTreeMap::from([
    ("note".into(), "备注".into()),
    ("tip".into(), "提示".into()),
    ("warning".into(), String::new()),
  ]);
  for (file, format) in [
    ("labels.md", ContentFormat::Markdown),
    ("labels.mdx", ContentFormat::Mdx),
  ] {
    let options = mdxjs::Options {
      filepath: Some(file.into()),
      ..Default::default()
    };
    let mut parsed = parse_content(source, &options, None, format).unwrap();
    parsed.apply_admonition_labels(&labels);
    // Defaults are consumed once, preventing a later pass from touching body
    // paragraphs after an empty default title has been removed.
    parsed.apply_admonition_labels(&BTreeMap::from([("warning".into(), "SECOND PASS".into())]));
    let compiled = emit_content(parsed, true).unwrap();
    assert_eq!(compiled.code.matches("farm-admonition-title").count(), 4);
    assert_eq!(compiled.code.matches("备注").count(), 1);
    assert_eq!(compiled.code.matches("提示").count(), 1);
    assert!(!compiled.code.contains("SECOND PASS"));
    for body in [
      "Default body.",
      "Nested body.",
      "Explicit body.",
      "Empty explicit.",
      "Hidden default.",
      "Unmapped default.",
    ] {
      assert!(compiled.code.contains(body), "lost {body}");
    }
    let map: serde_json::Value = serde_json::from_str(&compiled.source_map.unwrap()).unwrap();
    assert_eq!(map["sourcesContent"][0], source);
  }
}

#[test]
fn target_labels_are_applied_to_independent_clones_of_one_parse_tree() {
  let original = parse_content(
    ":::note\nOriginal body.\n:::\n",
    &Default::default(),
    None,
    ContentFormat::Markdown,
  )
  .unwrap();
  for label in ["Source label", "目标标签", "<literal text>"] {
    let mut document = original.clone();
    document.apply_admonition_labels(&BTreeMap::from([("note".into(), label.into())]));
    let compiled = emit_content(document, false).unwrap();
    assert!(compiled.code.contains(label));
    assert!(compiled.code.contains("Original body."));
    assert_eq!(compiled.code.matches("farm-admonition-title").count(), 1);
  }
  let unchanged = emit_content(original, false).unwrap();
  assert!(!unchanged.code.contains("Source label"));
  assert!(!unchanged.code.contains("目标标签"));
}
