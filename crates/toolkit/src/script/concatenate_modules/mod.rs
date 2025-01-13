use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{
    meta_data::script::statement::ExportInfo, module_graph::ModuleGraph, ModuleId, ModuleSystem,
  },
  plugin::ResolveKind,
  swc_common::DUMMY_SP,
  swc_ecma_ast::Module as SwcModule,
  HashSet,
};

mod expand_exports;
mod ident_generator;
mod strip_module;

pub struct ConcatenateModulesAstResult {
  /// The concatenated AST of the modules
  pub ast: SwcModule,
  /// The module IDs of the modules that are concatenated in order
  pub module_ids: Vec<ModuleId>,
  /// The external modules that are imported by the modules
  pub external_modules: HashSet<String>,
}

/// Concatenate the ASTs of the modules in the module graph starting from the entry module
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
///
/// The concatenation algorithm is as follows:
/// 1. Sort the modules by execution order
/// 2. Check if the module is esm, error if it is not
/// 3. Visit the sorted modules in order, for each module:
///   - Visit the module's AST and rewrite the import/export statements to use the module's variable name
///   - collect external modules that are not in module_ids
///   - for dynamic import, if the dynamic imported is in module_ids, replace dynamic import with module_ns, otherwise, replace it with the other resource path
/// 4. Concatenate the ASTs of the modules in order and return the result
/// ```
pub fn concatenate_modules_ast(
  module_ids: &HashSet<ModuleId>,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> Result<ConcatenateModulesAstResult, &'static str> {
  // 1. Sort the modules by execution order
  let mut sorted_modules: Vec<_> = module_ids.iter().collect();
  sorted_modules.sort_by_key(|module_id| module_graph.module(module_id).unwrap().execution_order);

  // 2. Check if the module is esm, panic if it is not
  for module_id in sorted_modules.iter() {
    let module = module_graph.module(module_id).unwrap();
    // error if it is no script module
    if !module.module_type.is_script() {
      return Err("Only script modules are supported");
    }

    // error if it is not ESM
    if module.meta.as_script().module_system != ModuleSystem::EsModule {
      return Err("Only ESM modules are supported");
    }
  }

  // 3. Visit sorted modules and process them
  let mut concatenated_ast = SwcModule {
    span: DUMMY_SP,
    body: vec![],
    shebang: None,
  };

  let mut external_modules = HashSet::default();

  for module_id in sorted_modules {
    let module = module_graph.module(module_id).unwrap();
    let script_meta = module.meta.as_script();

    // Visit and rewrite import/export statements
    let mut ast = script_meta.ast.clone();
    // rewrite_imports_exports(&mut ast, module_id);

    // // Collect external modules
    // collect_external_modules(&ast, module_ids, &mut external_modules);

    // Add to concatenated AST
    concatenated_ast.body.extend(ast.body);
  }

  // 4. Return the concatenated result
  Ok(ConcatenateModulesAstResult {
    ast: concatenated_ast,
    // module_ids: sorted_modules.into_iter().cloned().collect(),
    module_ids: vec![],
    external_modules,
  })
}
