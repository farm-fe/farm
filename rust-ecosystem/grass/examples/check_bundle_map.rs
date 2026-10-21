//! Acceptance tooling: trace an actual Farm CSS bundle to its authored input.
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let arguments: Vec<_> = std::env::args().skip(1).collect();
  if arguments.len() != 4
    && !(arguments.len() == 5 && arguments[4] == "--first-unmapped")
    && !(arguments.len() == 6 && arguments[4] == "--original-needle")
  {
    return Err("Usage: check_bundle_map <map> <css> <source> <needle> [--first-unmapped | --original-needle <text>]".into());
  }
  let map = sourcemap::SourceMap::from_slice(&std::fs::read(&arguments[0])?)?;
  if arguments.len() == 5 {
    assert!(
      map
        .lookup_token(0, 0)
        .expect("initial mapping")
        .get_source()
        .is_none(),
      "synthetic CSS must remain unmapped"
    );
  }
  let css = std::fs::read_to_string(&arguments[1])?;
  let original = std::fs::read_to_string(&arguments[2])?;
  let locate = |text: &str, needle: &str| {
    let offset = text.find(needle).expect("needle in source");
    let prefix = &text[..offset];
    (
      prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
      prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
  };
  let (line, column) = locate(&css, &arguments[3]);
  let token = map.lookup_token(line, column).ok_or("missing mapping")?;
  let filename = std::path::Path::new(&arguments[2])
    .file_name()
    .unwrap()
    .to_str()
    .unwrap();
  assert!(token.get_source().unwrap().ends_with(filename));
  assert_eq!(
    (token.get_src_line(), token.get_src_col()),
    locate(
      &original,
      if arguments.len() == 6 {
        &arguments[5]
      } else {
        &arguments[3]
      }
    )
  );
  assert!(map
    .source_contents()
    .any(|content| content == Some(original.as_str())));
  println!(
    "Bundle token maps to {}:{}:{} with original sourcesContent",
    token.get_source().unwrap(),
    token.get_src_line() + 1,
    token.get_src_col() + 1
  );
  Ok(())
}
