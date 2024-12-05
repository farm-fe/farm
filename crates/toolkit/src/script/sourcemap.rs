use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  context::{get_swc_sourcemap_filename, CompilationContext},
  module::{module_graph::ModuleGraph, ModuleId},
  rayon::iter::{IntoParallelRefMutIterator, ParallelIterator},
  resource::resource_pot::ResourcePotId,
  swc_common::{
    comments::{Comments, SingleThreadedComments},
    BytePos, SourceMap,
  },
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use super::generator::RenderModuleResult;

#[inline]
fn try_get_module_id_from_hoisted_map<'a>(
  map: &'a HashMap<ModuleId, Vec<ModuleId>>,
  module_id: &'a ModuleId,
) -> &'a ModuleId {
  if let Some(modules) = map.get(module_id)
    && !modules.is_empty()
  {
    return &modules[0];
  }

  module_id
}

pub fn merge_comments(
  render_module_results: &mut Vec<RenderModuleResult>,
  cm: Arc<SourceMap>,
  hoisted_map: &HashMap<ModuleId, Vec<ModuleId>>,
) -> SingleThreadedComments {
  let merged_comments = SingleThreadedComments::default();

  for RenderModuleResult {
    module_id,
    comments: module_comments,
    ..
  } in render_module_results
  {
    let module_id = try_get_module_id_from_hoisted_map(hoisted_map, module_id);

    let filename = get_swc_sourcemap_filename(module_id);
    let source_file = cm
      .get_source_file(&filename)
      .unwrap_or_else(|| panic!("no source file found for {:?}", module_id));
    let start_pos = source_file.start_pos;
    let comments = std::mem::take(module_comments);

    for item in comments.leading {
      let byte_pos = start_pos + item.byte_pos;
      for comment in item.comment {
        merged_comments.add_leading(byte_pos, comment);
      }
    }

    for item in comments.trailing {
      let byte_pos = start_pos + item.byte_pos;
      for comment in item.comment {
        merged_comments.add_trailing(byte_pos, comment);
      }
    }
  }

  merged_comments
}

pub fn merge_sourcemap(
  resource_pot_id: &ResourcePotId,
  render_module_results: &mut Vec<RenderModuleResult>,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
  hoisted_map: &HashMap<ModuleId, Vec<ModuleId>>,
) -> Arc<SourceMap> {
  let module_ids = render_module_results
    .iter()
    .map(|item| &item.module_id)
    .collect();
  let new_cm = context.meta.script.merge_nested_source_map(
    resource_pot_id,
    &module_ids,
    module_graph,
    hoisted_map,
  );

  // update Span in parallel
  render_module_results.par_iter_mut().for_each(|res| {
    let module_id = try_get_module_id_from_hoisted_map(hoisted_map, &res.module_id);
    let filename = get_swc_sourcemap_filename(module_id);
    let source_file = new_cm
      .get_source_file(&filename)
      .unwrap_or_else(|| panic!("no source file found for {:?}", module_id));
    let start_pos = source_file.start_pos;
    res
      .rendered_ast
      .visit_mut_with(&mut SpanUpdater { start_pos });
  });

  new_cm
}

pub struct SpanUpdater {
  pub start_pos: BytePos,
}

impl VisitMut for SpanUpdater {
  fn visit_mut_span(&mut self, node: &mut farmfe_core::swc_common::Span) {
    node.lo = self.start_pos + node.lo;
    node.hi = self.start_pos + node.hi;
  }
}
