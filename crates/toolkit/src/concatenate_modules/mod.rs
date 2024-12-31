use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{meta_data::script::statement::ExportInfo, module_graph::ModuleGraph, ModuleId},
  plugin::ResolveKind,
  swc_ecma_ast::Module as SwcModule,
  HashSet,
};

pub struct ConcatenateModulesAstResult {
  /// The concatenated AST of the modules
  pub ast: SwcModule,
  /// The module IDs of the modules that are concatenated in order
  pub module_ids: Vec<ModuleId>,
  /// The external modules that are imported by the modules
  pub external_modules: HashSet<String>,
}

/// Concatenate the ASTs of the modules in the module graph starting from the entry module by DFS
/// for example, if the input files are:
/// ```js
/// // a.js
/// import b from './b.js';
/// console.log(b + 1);
///
/// // b.js
/// export default 1;
/// ```
///
/// The output should be:
/// ```js
/// // b.js
/// var b = 1;
/// // a.js
/// console.log(b + 1);
/// ```
pub fn concatenate_modules_ast(
  entry_module_id: ModuleId,
  module_ids: &HashSet<ModuleId>,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> ConcatenateModulesAstResult {
  let mut visited = HashSet::default();

  traverse_modules_dfs(
    entry_module_id,
    module_ids,
    module_graph,
    context,
    &mut visited,
  )
}

fn traverse_modules_dfs(
  entry_module_id: ModuleId,
  module_ids: &HashSet<ModuleId>,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
  visited: &mut HashSet<ModuleId>,
) -> ConcatenateModulesAstResult {
  let entry_module = module_graph.module(&entry_module_id).unwrap();
  let entry_module_meta = entry_module.meta.as_script();
  let mut concatenated_ast = entry_module_meta.ast.clone();
  let mut concatenated_module_ids = vec![];
  let mut external_modules: HashSet<String> = HashSet::default();

  visited.insert(entry_module_id.clone());

  // traverse the module starting from the entry module by DFS
  for statement in &entry_module_meta.statements {
    // deal with import/export from statement
    if let Some(import_info) = &statement.import_info {
      let dep_module_id = module_graph.get_dep_by_source(
        &entry_module_id,
        &import_info.source,
        Some(ResolveKind::Import),
      );

      if !visited.contains(&dep_module_id) {
        let result =
          traverse_modules_dfs(dep_module_id, module_ids, module_graph, context, visited);

        concatenated_ast
          .body
          .splice(statement.id..(statement.id + 1), result.ast.body);
        concatenated_module_ids.extend(result.module_ids);
      }
    } else if let Some(ExportInfo {
      source: Some(source),
      specifiers,
      ..
    }) = &statement.export_info
    {
    }
  }

  concatenated_module_ids.push(entry_module_id);

  ConcatenateModulesAstResult {
    ast: concatenated_ast,
    module_ids: concatenated_module_ids,
    external_modules,
  }
}
