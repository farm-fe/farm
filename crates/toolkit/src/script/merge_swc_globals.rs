use std::sync::Arc;

use farmfe_core::{
  HashMap,
  context::{CompilationContext, get_swc_sourcemap_filename},
  module::{ModuleId, meta_data::script::CommentsMetaData},
  rayon::iter::{IntoParallelRefMutIterator, ParallelIterator},
  swc_common::{
    BytePos, SourceMap,
    comments::{Comments, SingleThreadedComments},
  },
  swc_ecma_ast::Module as SwcModule,
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
  let mut relocations = HashMap::default();

  for module_id in module_ids {
    let cm = source_maps
      .remove(module_id)
      .unwrap_or(context.meta.get_module_source_map(module_id));
    // for scope hoisted module, the source map may be combined with other modules
    // so we need to merge all source files to the new source map
    let ranges = cm
      .files()
      .iter()
      .map(|source_file| {
        let new_file = new_cm.new_source_file(source_file.name.clone(), source_file.src.clone());
        (
          source_file.start_pos,
          source_file.end_pos,
          new_file.start_pos,
        )
      })
      .collect::<Vec<_>>();
    relocations.insert(module_id.clone(), ranges);
  }

  // update Span in parallel
  module_asts.par_iter_mut().for_each(|(module_id, ast)| {
    ast.visit_mut_with(&mut SpanUpdater {
      ranges: &relocations[module_id],
    });
  });

  new_cm
}

struct SpanUpdater<'a> {
  // A scope-hoisted AST already refers to several source files. Relocate each
  // file from its old interval; shifting everything by the entry file's start
  // corrupts both imported file identity and columns on repeated merges.
  ranges: &'a [(BytePos, BytePos, BytePos)],
}

impl SpanUpdater<'_> {
  fn relocate(&self, position: BytePos) -> BytePos {
    if position.is_dummy() || position.is_reserved_for_comments() {
      return position;
    }
    let index = self
      .ranges
      .partition_point(|(start, _, _)| *start <= position);
    if let Some((start, end, destination)) = index.checked_sub(1).map(|index| self.ranges[index]) {
      if position <= end {
        return BytePos(destination.0 + position.0 - start.0);
      }
    }
    position
  }
}

impl VisitMut for SpanUpdater<'_> {
  fn visit_mut_span(&mut self, node: &mut farmfe_core::swc_common::Span) {
    // Do not update the span if it's auto-generated
    if node.lo.is_dummy() || node.lo.is_reserved_for_comments() {
      return;
    }

    node.lo = self.relocate(node.lo);
    node.hi = self.relocate(node.hi);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::script::parse_module;
  use farmfe_core::{config::Config, swc_common::Spanned, swc_ecma_ast::EsVersion};

  #[test]
  fn nested_merge_preserves_each_original_file_and_column() {
    let context = Arc::new(CompilationContext::new(Config::default(), vec![]).unwrap());
    let mut modules = vec![];
    for (name, content) in [("a.js", "const alpha = 1;"), ("b.js", "const beta = 2;")] {
      let id: ModuleId = name.into();
      let parsed = parse_module(
        &id,
        Arc::new(content.into()),
        Default::default(),
        EsVersion::EsNext,
      )
      .unwrap();
      context.meta.set_module_source_map(&id, parsed.source_map);
      modules.push((id, parsed.ast));
    }
    let first = merge_sourcemap(&mut modules, Default::default(), &context);
    for (id, ast) in &modules {
      let location = first.lookup_char_pos(ast.body[0].span().lo);
      assert_eq!(location.file.name, Arc::new(get_swc_sourcemap_filename(id)));
      assert_eq!(location.col.0, 0, "first merge column of {id:?}");
    }
    let root: ModuleId = "a.js".into();
    let mut combined = modules.remove(0).1;
    combined.body.extend(modules.remove(0).1.body);
    let mut combined = vec![(root.clone(), combined)];
    let second = merge_sourcemap(&mut combined, HashMap::from_iter([(root, first)]), &context);
    for (item, expected) in combined[0].1.body.iter().zip(["a.js", "b.js"]) {
      let location = second.lookup_char_pos(item.span().lo);
      assert_eq!(
        location.file.name,
        Arc::new(get_swc_sourcemap_filename(&expected.into()))
      );
      assert_eq!(location.col.0, 0, "nested merge column of {expected}");
    }
  }
}
