use crate::transform::Diagnostic;
use farmfe_core::{context::create_swc_source_map, module::ModuleId};
use farmfe_toolkit::sourcemap::SourceMap;
use std::sync::Arc;

/// Trace an existing AST position. This does not parse or modify authored code.
pub(crate) fn format(
  diagnostic: &Diagnostic,
  id: &ModuleId,
  content: Arc<String>,
  chain: &[Arc<String>],
) -> String {
  let fallback = || format!("{}: {}", id.to_string(), diagnostic.message);
  let Some(span) = diagnostic.span else {
    return fallback();
  };
  // Script parsing creates a fresh SourceMap per module using this same helper.
  let (_, file) = create_swc_source_map(id, content.clone());
  let Some(offset) = span.lo.0.checked_sub(file.start_pos.0) else {
    return fallback();
  };
  let Some(prefix) = content.get(..offset as usize) else {
    return fallback();
  };
  let mut line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
  let mut column = prefix
    .rsplit('\n')
    .next()
    .map_or(0, |line| line.encode_utf16().count()) as u32;
  let mut source = id.to_string();
  for serialized in chain.iter().rev() {
    let Ok(map) = SourceMap::from_slice(serialized.as_bytes()) else {
      break;
    };
    if map.get_token_count() == 0 {
      continue;
    }
    let Some(token) = map
      .lookup_token(line, column)
      .filter(|token| token.get_dst_line() == line)
    else {
      break;
    };
    let Some(name) = token.get_source() else {
      break;
    };
    source = name.to_string();
    line = token.get_src_line();
    column = token.get_src_col();
  }
  format!(
    "{source}:{}:{}: {}",
    line + 1,
    column + 1,
    diagnostic.message
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::swc_common::{BytePos, Span};

  #[test]
  fn plain_script_columns_are_utf16_and_bad_maps_do_not_invent_author_locations() {
    let id = ModuleId::from("fixture.tsx");
    let source = Arc::new("// 中文\r\nconst label='🌱'; <CodeBlock/>".to_owned());
    let offset = source.find("CodeBlock").unwrap();
    let (_, file) = create_swc_source_map(&id, source.clone());
    let point = file.start_pos + BytePos(offset as u32);
    let diagnostic = Diagnostic::at(
      "[DOCS_SNIPPET_STATIC] invalid input",
      Span::new(point, point),
    );
    let expected = format!(
      "fixture.tsx:2:{}: [DOCS_SNIPPET_STATIC] invalid input",
      source[..offset]
        .rsplit('\n')
        .next()
        .unwrap()
        .encode_utf16()
        .count()
        + 1
    );
    for chain in [vec![],vec![Arc::new("not a source map".into())],vec![Arc::new(r#"{"version":3,"sources":["wrong.mdx"],"names":[],"mappings":"AAAA","sourcesContent":["wrong"]}"#.into())]] {
      assert_eq!(format(&diagnostic,&id,source.clone(),&chain),expected);
    }
  }
}
