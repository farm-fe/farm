//! Swc implementation of the sourcemap generator.

use std::{collections::HashMap, path::Path, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  swc_common::{
    source_map::{Pos, SourceMapGenConfig},
    BytePos, FileName, LineCol, SourceFile, SourceMap,
  },
  swc_ecma_ast::{Ident, Module as SwcModule},
};
use farmfe_utils::diff_paths;
use sourcemap::SourceMapBuilder;
use swc_atoms::JsWord;
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};

/*
Modified by brightwu, the original source code is from the swc project.

Copyright 2023 강동윤 <kdy1997.dev@gmail.com>

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
 */

/// Stores the state of the last conversion between BytePos and CharPos.
#[derive(Debug, Clone, Default)]
pub struct ByteToCharPosState {
  /// The last BytePos to convert.
  pos: BytePos,

  /// The total number of extra chars in the UTF-8 encoding.
  total_extra_bytes: u32,

  /// The index of the last MultiByteChar read to compute the extra bytes of
  /// the last conversion.
  mbc_index: usize,
}

/// Calculates the number of excess chars seen in the UTF-8 encoding of a
/// file compared with the UTF-16 encoding.
fn calc_utf16_offset(file: &SourceFile, bpos: BytePos, state: &mut ByteToCharPosState) -> u32 {
  let mut total_extra_bytes = state.total_extra_bytes;
  let mut index = state.mbc_index;

  if bpos >= state.pos {
    let range = index..file.multibyte_chars.len();
    for i in range {
      let mbc = &file.multibyte_chars[i];
      // debug!("{}-byte char at {:?}", mbc.bytes, mbc.pos);
      if mbc.pos >= bpos {
        break;
      }
      total_extra_bytes += mbc.byte_to_char_diff() as u32;
      // We should never see a byte position in the middle of a
      // character
      debug_assert!(
        bpos.to_u32() >= mbc.pos.to_u32() + mbc.bytes as u32,
        "bpos = {:?}, mbc.pos = {:?}, mbc.bytes = {:?}",
        bpos,
        mbc.pos,
        mbc.bytes
      );
      index += 1;
    }
  } else {
    let range = 0..index;
    for i in range.rev() {
      let mbc = &file.multibyte_chars[i];
      // debug!("{}-byte char at {:?}", mbc.bytes, mbc.pos);
      if mbc.pos < bpos {
        break;
      }
      total_extra_bytes -= mbc.byte_to_char_diff() as u32;
      // We should never see a byte position in the middle of a
      // character
      debug_assert!(
        bpos.to_u32() <= mbc.pos.to_u32(),
        "bpos = {:?}, mbc.pos = {:?}",
        bpos,
        mbc.pos,
      );
      index -= 1;
    }
  }

  state.pos = bpos;
  state.total_extra_bytes = total_extra_bytes;
  state.mbc_index = index;

  total_extra_bytes
}

pub fn build_source_map_with_config(
  cm: Arc<SourceMap>,
  mappings: &[(BytePos, LineCol)],
  orig_map_chains: &[sourcemap::SourceMap],
  config: impl SourceMapGenConfig,
) -> sourcemap::SourceMap {
  let mut builder = SourceMapBuilder::new(None);

  let mut src_id = 0u32;

  for orig in orig_map_chains {
    for src in orig.sources() {
      let id = builder.add_source(src);
      src_id = id + 1;

      builder.set_source_contents(id, orig.get_source_contents(id));
    }
  }

  // // This method is optimized based on the fact that mapping is sorted.
  // mappings.sort_by_key(|v| v.0);

  let mut cur_file: Option<Arc<SourceFile>> = None;

  let mut prev_dst_line = u32::MAX;

  let mut inline_sources_content = false;
  let mut ch_state = ByteToCharPosState::default();
  let mut line_state = ByteToCharPosState::default();

  for (pos, lc) in mappings.iter() {
    let pos = *pos;

    if pos.is_reserved_for_comments() {
      continue;
    }

    let lc = *lc;

    // If pos is same as a DUMMY_SP (eg BytePos(0)) and if line and col are 0;
    // ignore the mapping.
    if lc.line == 0 && lc.col == 0 && pos.is_dummy() {
      continue;
    }

    if pos == BytePos(u32::MAX) {
      builder.add_raw(lc.line, lc.col, 0, 0, Some(src_id), None);
      continue;
    }

    let f;
    let f = match cur_file {
      Some(ref f) if f.start_pos <= pos && pos < f.end_pos => f,
      _ => {
        f = cm.lookup_source_file(pos);
        src_id = builder.add_source(&config.file_name_to_source(&f.name));

        inline_sources_content = config.inline_sources_content(&f.name);

        if inline_sources_content && orig_map_chains.is_empty() {
          builder.set_source_contents(src_id, Some(&f.src));
        }

        ch_state = ByteToCharPosState::default();
        line_state = ByteToCharPosState::default();

        cur_file = Some(f.clone());
        &f
      }
    };
    if config.skip(&f.name) {
      continue;
    }

    let emit_columns = config.emit_columns(&f.name);

    if !emit_columns && lc.line == prev_dst_line {
      continue;
    }

    let mut line = match f.lookup_line(pos) {
      Some(line) => line as u32,
      None => continue,
    };

    let linebpos = f.lines[line as usize];
    debug_assert!(
      pos >= linebpos,
      "{}: bpos = {:?}; linebpos = {:?};",
      f.name,
      pos,
      linebpos,
    );

    let linechpos = linebpos.to_u32() - calc_utf16_offset(f, linebpos, &mut line_state);
    let chpos = pos.to_u32() - calc_utf16_offset(f, pos, &mut ch_state);

    debug_assert!(
      chpos >= linechpos,
      "{}: chpos = {:?}; linechpos = {:?};",
      f.name,
      chpos,
      linechpos,
    );

    let mut col = chpos - linechpos;
    let mut name = None;
    if let Some(orig) = &orig {
      if let Some(token) = orig
        .lookup_token(line, col)
        .filter(|t| t.get_dst_line() == line)
      {
        line = token.get_src_line();
        col = token.get_src_col();
        if token.has_name() {
          name = token.get_name();
        }
        if let Some(src) = token.get_source() {
          src_id = builder.add_source(src);
          if inline_sources_content && !builder.has_source_contents(src_id) {
            if let Some(contents) = token.get_source_view() {
              builder.set_source_contents(src_id, Some(contents.source()));
            }
          }
        }
      } else {
        continue;
      }
    }

    let name_idx = name
      .or_else(|| config.name_for_bytepos(pos))
      .map(|name| builder.add_name(name));

    builder.add_raw(lc.line, lc.col, line, col, Some(src_id), name_idx);
    prev_dst_line = lc.line;
  }

  builder.into_sourcemap()
}

struct SwcSourceMapConfig<'a> {
  source_file_name: Option<&'a str>,
  /// Output path of the `.map` file.
  output_path: Option<&'a Path>,

  names: &'a HashMap<BytePos, JsWord>,

  inline_sources_content: bool,

  emit_columns: bool,
}

impl SourceMapGenConfig for SwcSourceMapConfig<'_> {
  fn file_name_to_source(&self, f: &FileName) -> String {
    if let Some(file_name) = self.source_file_name {
      return file_name.to_string();
    }

    let base_path = match self.output_path {
      Some(v) => v,
      None => return f.to_string(),
    };
    let target = match f {
      FileName::Real(v) => v,
      _ => return f.to_string(),
    };

    let rel = diff_paths(target, base_path);
    match rel {
      Some(v) => {
        let s = v.to_string_lossy().to_string();
        if cfg!(target_os = "windows") {
          s.replace('\\', "/")
        } else {
          s
        }
      }
      None => f.to_string(),
    }
  }

  fn name_for_bytepos(&self, pos: BytePos) -> Option<&str> {
    self.names.get(&pos).map(|v| &**v)
  }

  fn inline_sources_content(&self, _: &FileName) -> bool {
    self.inline_sources_content
  }

  fn emit_columns(&self, _f: &FileName) -> bool {
    self.emit_columns
  }

  fn skip(&self, f: &FileName) -> bool {
    if let FileName::Custom(s) = f {
      s.starts_with('<')
    } else {
      false
    }
  }
}

pub enum AstModule<'a> {
  Script(&'a SwcModule),
  Css(&'a farmfe_core::swc_css_ast::Stylesheet),
}

pub fn build_source_map(
  mappings: &[(BytePos, LineCol)],
  cm: Arc<SourceMap>,
  ast: AstModule,
  context: Arc<CompilationContext>,
) -> Vec<u8> {
  let mut v = IdentCollector {
    names: Default::default(),
  };

  v.visit_ident(ast);

  // let config = SwcSourceMapConfig {
  //   source_file_name: None,
  //   output_path: None,
  //   names: &v.names,
  //   inline_sources_content: true,
  //   emit_columns: true,
  // };
  let mut src_buf = vec![];
  cm.build_source_map_with_config(mappings, None, config)
    .to_writer(&mut src_buf)
    .unwrap();

  src_buf
}

pub struct IdentCollector {
  names: HashMap<BytePos, JsWord>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}

impl swc_css_visit::Visit for IdentCollector {
  fn visit_ident(&mut self, ident: &farmfe_core::swc_css_ast::Ident) {
    self.names.insert(ident.span.lo, ident.value.clone());
  }
}

impl IdentCollector {
  fn visit_ident(&mut self, module: AstModule) {
    match module {
      AstModule::Script(ast) => ast.visit_with(self),
      AstModule::Css(stylesheet) => {
        use swc_css_visit::VisitWith;
        stylesheet.visit_with(self)
      }
    };
  }
}
