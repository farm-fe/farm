use std::sync::Arc;

use farmfe_core::{
  context::{get_swc_sourcemap_filename, CompilationContext},
  module::{meta_data::script::CommentsMetaData, ModuleId},
  rayon::iter::{IntoParallelRefMutIterator, ParallelIterator},
  swc_common::{
    comments::{Comments, SingleThreadedComments},
    BytePos, SourceMap,
  },
  swc_ecma_ast::Module as SwcModule,
  HashMap,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub fn merge_comments(
  comments: &mut Vec<(ModuleId, CommentsMetaData)>,
  cm: Arc<SourceMap>,
) -> SingleThreadedComments {
  let merged_comments = SingleThreadedComments::default();

  for (module_id, module_comments) in comments {
    let filename = get_swc_sourcemap_filename(module_id);
    let source_file = cm
      .get_source_file(&filename)
      .unwrap_or_else(|| panic!("no source file found for {:?}", module_id));
    let start_pos = source_file.start_pos;
    let comments = std::mem::take(module_comments);

    let get_byte_pos = |byte_pos: BytePos| {
      if byte_pos.is_dummy() || byte_pos.is_reserved_for_comments() {
        byte_pos
      } else {
        start_pos + byte_pos
      }
    };

    for item in comments.leading {
      let byte_pos = get_byte_pos(item.byte_pos);
      // insert comments whose byte_pos is updated to the new position
      for comment in item.comment {
        merged_comments.add_leading(byte_pos, comment);
      }
    }

    for item in comments.trailing {
      let byte_pos = get_byte_pos(item.byte_pos);
      // insert comments whose byte_pos is updated to the new position
      for comment in item.comment {
        merged_comments.add_trailing(byte_pos, comment);
      }
    }
  }

  merged_comments
}

pub fn merge_sourcemap(
  module_asts: &mut Vec<(ModuleId, SwcModule)>,
  mut source_maps: HashMap<ModuleId, Arc<SourceMap>>,
  context: &Arc<CompilationContext>,
) -> Arc<SourceMap> {
  let module_ids: Vec<_> = module_asts.iter().map(|item| &item.0).collect();
  let new_cm = Arc::new(SourceMap::default());

  for module_id in module_ids {
    let cm = source_maps
      .remove(module_id)
      .unwrap_or(context.meta.get_module_source_map(module_id));
    // for scope hoisted module, the source map may be combined with other modules
    // so we need to merge all source files to the new source map
    cm.files().iter().for_each(|source_file| {
      new_cm.new_source_file_from(source_file.name.clone(), source_file.src.clone());
    });
  }

  // update Span in parallel
  module_asts.par_iter_mut().for_each(|(module_id, ast)| {
    let filename = get_swc_sourcemap_filename(module_id);
    let source_file = new_cm
      .get_source_file(&filename)
      .unwrap_or_else(|| panic!("no source file found for {:?}", module_id));
    let start_pos = source_file.start_pos;
    ast.visit_mut_with(&mut SpanUpdater { start_pos });
  });

  new_cm
}

struct SpanUpdater {
  pub start_pos: BytePos,
}

impl VisitMut for SpanUpdater {
  fn visit_mut_span(&mut self, node: &mut farmfe_core::swc_common::Span) {
    // Do not update the span if it's auto-generated
    if node.lo.is_dummy() || node.lo.is_reserved_for_comments() {
      return;
    }

    node.lo = self.start_pos + node.lo;
    node.hi = self.start_pos + node.hi;
  }
}
