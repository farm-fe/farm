use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{module_graph::ModuleGraph, ModuleId},
  swc_common::DUMMY_SP,
  swc_ecma_ast::{ImportDecl, ModuleDecl, ModuleItem},
};

use crate::{RUNTIME_INPUT_SCOPE, RUNTIME_PACKAGE};

fn try_remove_dynamic_input_entry(
  input: String,
  module_graph: &mut ModuleGraph,
) -> Option<ModuleId> {
  let entry = module_graph
    .entries
    .iter()
    .find(|(_, i)| **i == input)
    .map(|(e, _)| e.clone());

  entry.map(|e| {
    module_graph.entries.remove(&e);
    e
  })
}

fn insert_dynamic_input_import(
  entry: ModuleId,
  index: usize,
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) {
  let entry_module = module_graph.module_mut(&RUNTIME_PACKAGE.into()).unwrap();
  let ast = &mut entry_module.meta.as_script_mut().ast;
  ast.body.insert(
    index,
    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
      span: DUMMY_SP,
      specifiers: vec![],
      src: Box::new(entry.id(context.config.mode).into()),
      type_only: false,
      with: None,
      phase: Default::default(),
    })),
  );
}

fn insert_post_dynamic_input_import(
  entry: ModuleId,
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) {
  // find last import index
  let entry_module = module_graph.module_mut(&RUNTIME_PACKAGE.into()).unwrap();
  let ast = &mut entry_module.meta.as_script_mut().ast;

  let mut last_import_index = None;

  for (i, item) in ast.body.iter().enumerate() {
    if matches!(item, ModuleItem::ModuleDecl(_)) {
      last_import_index = Some(i);
    }
  }

  insert_dynamic_input_import(
    entry,
    // i + 1 if a module decl statement is found and 0 if there is no module decl
    last_import_index.map(|i| i + 1).unwrap_or(0),
    module_graph,
    context,
  );
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
  let post_dynamic_inputs = vec![
    format!("{RUNTIME_INPUT_SCOPE}_module_system_helper"),
    format!("{RUNTIME_INPUT_SCOPE}_module_helper"),
  ];
  for input in post_dynamic_inputs {
    if let Some(entry) = try_remove_dynamic_input_entry(input, module_graph) {
      // insert import '/xxx/runtime/dynamic-import' at first import statement
      insert_post_dynamic_input_import(entry, module_graph, context);
    }
  }

  let pre_dynamic_inputs = vec![
    format!("{RUNTIME_INPUT_SCOPE}_dynamic_import"),
    format!("{RUNTIME_INPUT_SCOPE}_plugin"),
  ];

  for input in pre_dynamic_inputs {
    if let Some(entry) = try_remove_dynamic_input_entry(input, module_graph) {
      // insert import '/xxx/runtime/dynamic-import' at first import statement
      insert_dynamic_input_import(entry, 0, module_graph, context);
    }
  }
}
