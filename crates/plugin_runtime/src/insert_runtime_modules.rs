use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge, ModuleGraphEdgeDataItem},
    ModuleId,
  },
  plugin::ResolveKind,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{ImportDecl, ModuleDecl, ModuleItem},
};

use crate::{RUNTIME_INPUT_SCOPE, RUNTIME_PACKAGE};

fn try_get_dynamic_input_entry(input: String, module_graph: &ModuleGraph) -> Option<ModuleId> {
  module_graph
    .dynamic_entries
    .iter()
    .find(|(_, i)| **i == input)
    .map(|(e, _)| e.clone())
}

fn insert_dynamic_input_import(
  dynamic_entry: ModuleId,
  index: usize,
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) {
  let entry_module_id: ModuleId = RUNTIME_PACKAGE.into();
  let entry_module = module_graph.module_mut(&entry_module_id).unwrap();
  let ast = &mut entry_module.meta.as_script_mut().ast;
  ast.body.insert(
    index,
    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
      span: DUMMY_SP,
      specifiers: vec![],
      src: Box::new(dynamic_entry.id(context.config.mode).into()),
      type_only: false,
      with: None,
      phase: Default::default(),
    })),
  );

  module_graph
    .add_edge(
      &entry_module_id,
      &dynamic_entry,
      ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
        source: dynamic_entry.id(context.config.mode),
        kind: ResolveKind::Import,
        order: index,
      }]),
    )
    .unwrap();

  module_graph.dynamic_entries.remove(&dynamic_entry);
}

/// Example:
/// ```js
/// // pre dynamic inputs than must be executed before module system
/// import '/xxx/runtime/dynamic-import'
/// import '/xxx/runtime/plugin'
///
/// import '/xxx/runtime/module-system'
///
/// // pre dynamic inputs than must be executed after module system
/// import '/xxx/runtime/module-system-helper'
/// import '/xxx/runtime/module-helper'
/// ```
pub fn insert_runtime_modules(module_graph: &mut ModuleGraph, context: &Arc<CompilationContext>) {
  // remove runtime dynamic entry and insert them into runtime entry module
  let dynamic_inputs = vec![
    format!("{RUNTIME_INPUT_SCOPE}_dynamic_import"),
    format!("{RUNTIME_INPUT_SCOPE}_plugin"),
    format!("{RUNTIME_INPUT_SCOPE}_module_system"),
    format!("{RUNTIME_INPUT_SCOPE}_module_system_helper"),
    format!("{RUNTIME_INPUT_SCOPE}_module_helper"),
  ];

  for (index, input) in dynamic_inputs.into_iter().enumerate() {
    if let Some(entry) = try_get_dynamic_input_entry(input, module_graph) {
      // insert import '/xxx/runtime/dynamic-import' at first import statement
      insert_dynamic_input_import(entry, index, module_graph, context);
    }
  }
}
