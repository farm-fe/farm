pub mod swc_gen;
// TODO: implement FarmSourceMapBuilder when support original sourcemap

// use std::sync::Arc;

// use farmfe_core::swc_common::{source_map::ByteToCharPosState, BytePos, LineCol, SourceFile};
// use sourcemap::SourceMapBuilder;

// pub struct FarmSourceMapBuilderOptions {
//   src_map_buf: Vec<(BytePos, LineCol)>,
//   original_sourcemap: Option<sourcemap::SourceMap>,
// }

// pub struct FarmSourceMapBuilder {
//   src_map_buf: Vec<(BytePos, LineCol)>,
//   original_sourcemap: Option<sourcemap::SourceMap>,
// }

// impl FarmSourceMapBuilder {
//   pub fn new(options: FarmSourceMapBuilderOptions) -> Self {
//     Self {
//       src_map_buf: options.src_map_buf,
//       original_sourcemap: options.original_sourcemap,
//     }
//   }

//   pub fn build(&self) -> sourcemap::SourceMap {
//     let mut builder = SourceMapBuilder::new(None);

//     let mut src_id = 0u32;
//     // TODO: support original sourcemap

//     let mut cur_file: Option<Arc<SourceFile>> = None;
//     let mut prev_dst_line = u32::MAX;

//     let mut ch_state = ByteToCharPosState::default();
//     let mut line_state = ByteToCharPosState::default();

//     for (pos, lc) in &self.src_map_buf {
//       if pos.is_reserved_for_comments() {
//         continue;
//       }

//       if lc.line == 0 && lc.col == 0 && pos.is_dummy() {
//         continue;
//       }

//       let pos = *pos;
//       let lc = *lc;

//       if pos == BytePos(u32::MAX) {
//         builder.add_raw(lc.line, lc.col, 0, 0, Some(src_id), None);
//         continue;
//       }

//       let file = match cur_file {
//         Some(ref file) if file.start_pos <= pos && pos < file.end_pos => file,
//         _ => {
//           let file = self.
//         }

//       };
//     }

//     builder.into_sourcemap()
//   }

//   fn lookup_source_file(&self, pos: BytePos) -> Lrc<SourceFile> {
//     let files = self.files.borrow();
//     let files = &files.source_files;
//     let fm = Self::lookup_source_file_in(files, pos);
//     match fm {
//         Some(fm) => fm,
//         None => {
//             panic!(
//                 "position {} does not resolve to a source location",
//                 pos.to_usize()
//             );
//         }
//     }
// }
// }
