use std::sync::Arc;

use farmfe_core::{
  context::create_swc_source_map,
  module::ModuleId,
  swc_common::{BytePos, LineCol},
};
use farmfe_toolkit::sourcemap::build_sourcemap;

#[test]
fn script_map_keeps_distinct_columns_on_one_generated_line() {
  let source = "世界(); second();";
  let offset = source.find("second").unwrap();
  let (cm, file) = create_swc_source_map(&ModuleId::from("unicode.js"), Arc::new(source.into()));
  let map = build_sourcemap(
    cm,
    &[
      (file.start_pos, LineCol { line: 0, col: 0 }),
      (
        file.start_pos + BytePos(offset as u32),
        LineCol { line: 0, col: 30 },
      ),
    ],
  );
  let first = map.lookup_token(0, 0).unwrap();
  let second = map.lookup_token(0, 30).unwrap();
  assert_eq!(first.get_dst_col(), 0);
  assert_eq!(first.get_src_col(), 0);
  assert_eq!(second.get_dst_col(), 30);
  assert_eq!(
    second.get_src_col(),
    source[..offset].encode_utf16().count() as u32
  );
  assert_eq!(
    second.get_source().map(|value| value.as_str()),
    Some("unicode.js")
  );
  assert_eq!(
    map.get_source_contents(0).map(|value| value.as_str()),
    Some(source)
  );
}
