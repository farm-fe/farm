use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{
    meta_data::script::{statement::SwcId, CommentsMetaData, ModuleExportIdent},
    module_graph::ModuleGraph,
    ModuleId,
  },
  plugin::ResolveKind,
  rayon::iter::{IntoParallelRefMutIterator, ParallelIterator},
  swc_common::{Globals, Mark, SourceMap, DUMMY_SP},
  swc_ecma_ast::Module as SwcModule,
  HashMap, HashSet,
};
use strip_module_decl::{strip_module_decl, StripModuleDeclResult};
use swc_ecma_visit::VisitMutWith;
pub use unique_idents::EXPORT_NAMESPACE;
use utils::create_var_namespace_item;

use super::{
  merge_swc_globals::{merge_comments, merge_sourcemap},
  swc_try_with::{resolve_module_mark, try_with},
};

mod strip_module_decl;
mod unique_idents;
pub(crate) mod utils;

pub struct ConcatenateModulesAstResult {
  /// The concatenated AST of the modules
  pub ast: SwcModule,
  /// The module IDs of the modules that are concatenated in order
  pub module_ids: Vec<ModuleId>,
  /// The external modules that are imported by the modules
  pub external_modules: HashMap<(String, ResolveKind), ModuleId>,
  /// The source map of the concatenated AST
  pub source_map: Arc<SourceMap>,
  /// Swc Globals that represent the syntax context of the concatenated AST
  pub globals: Globals,
  /// The comments of the concatenated AST
  pub comments: CommentsMetaData,
  pub unresolved_mark: Mark,
  pub top_level_mark: Mark,
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
/// var b_default = 1;
/// // a.js
/// console.log(b_default + 1);
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
  entry_module_id: &ModuleId,
  module_ids: &HashSet<ModuleId>,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> Result<ConcatenateModulesAstResult, &'static str> {
  let strip_module_results =
    strip_modules_asts(entry_module_id, module_ids, module_graph, context)?;

  let mut comments = vec![];
  let mut module_asts = vec![];
  let mut preserved_import_decls = vec![];
  let mut preserved_export_decls = vec![];

  let mut sorted_modules = vec![];

  for (module_id, stripped_module) in strip_module_results {
    comments.push((module_id.clone(), stripped_module.comments));
    module_asts.push((module_id.clone(), stripped_module.ast));

    sorted_modules.push(module_id);
    preserved_import_decls.push(stripped_module.preserved_import_decls);
    preserved_export_decls.push(stripped_module.preserved_export_decls);
  }

  let merged_source_map = merge_sourcemap(&mut module_asts, Default::default(), context);
  let merged_comments = merge_comments(&mut comments, merged_source_map.clone()).into();

  // merge comments, module_asts and preserved_module_decls back to strip_module_results
  let stripped_results = comments
    .into_iter()
    .zip(module_asts.into_iter())
    .zip(preserved_import_decls.into_iter())
    .zip(preserved_export_decls.into_iter())
    .map(
      |((((_, comments), (_, ast)), import_decls), export_decls)| StripModuleDeclResult {
        comments,
        ast,
        preserved_import_decls: import_decls,
        preserved_export_decls: export_decls,
      },
    );

  let (mut concatenated_ast, external_modules) =
    merge_stripped_module_asts(stripped_results.collect());

  let merged_globals = Globals::new();
  let (unresolved_mark, top_level_mark) =
    resolve_module_mark(&mut concatenated_ast, false, &merged_globals);

  Ok(ConcatenateModulesAstResult {
    ast: concatenated_ast,
    module_ids: sorted_modules,
    external_modules,
    source_map: merged_source_map,
    comments: merged_comments,
    globals: merged_globals,
    unresolved_mark,
    top_level_mark,
  })
}

fn strip_modules_asts(
  entry_module_id: &ModuleId,
  module_ids: &HashSet<ModuleId>,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> Result<Vec<(ModuleId, StripModuleDeclResult)>, &'static str> {
  // 1. Sort the modules by execution order
  let mut sorted_modules: Vec<_> = module_ids.iter().cloned().collect();
  sorted_modules.sort_by_key(|module_id| module_graph.module(module_id).unwrap().execution_order);
  // cyclic_idents should be accessed by get method, we should collect cyclic idents from cyclic modules first
  let mut cyclic_idents = HashMap::<ModuleId, HashSet<ModuleExportIdent>>::default();

  // 2. Check if the module is esm, panic if it is not
  for module_id in &sorted_modules {
    let module = module_graph.module(module_id).unwrap();
    // error if it is no script module
    if !module.module_type.is_script() {
      return Err("Only script modules are supported when concatenating modules");
    }

    // // error if it is not ESM
    // if module.meta.as_script().module_system != ModuleSystem::EsModule {
    //   return Err("Only ESM modules are supported when concatenating modules");
    // }

    if module_graph.circle_record.is_in_circle(module_id) {
      cyclic_idents
        .entry(module_id.clone())
        .or_default()
        .extend(module.meta.as_script().export_ident_map.values().cloned());
    }
  }

  // get export info from the entry module
  let entry_module = module_graph.module(entry_module_id).unwrap();
  let entry_module_export_ident_map = entry_module.meta.as_script().get_export_idents();
  let mut module_export_ident_map = entry_module_export_ident_map.into_iter().fold(
    HashMap::<ModuleId, HashMap<String, SwcId>>::default(),
    |mut acc, (k, v)| {
      acc.entry(v.module_id).or_default().insert(k, v.ident);
      acc
    },
  );

  // delayed rename for cyclic reexport of `export * as ns`. See test case: crates/compiler/tests/fixtures/library/reexport/basic
  let mut delayed_rename = HashMap::default();

  // 3. Visit sorted modules and process them
  let mut rename_handler = unique_idents::init_rename_handler(&sorted_modules, module_graph);
  let mut strip_module_results = vec![];

  for module_id in sorted_modules {
    let module = module_graph.module(&module_id).unwrap();
    let cm = context.meta.get_module_source_map(&module_id);

    try_with(cm, context.meta.get_globals(&module_id).value(), || {
      let export_ident_map = &module.meta.as_script().export_ident_map;
      // rename module_namespace if there are conflicts
      if let Some(module_export_ident) = export_ident_map.get(EXPORT_NAMESPACE) {
        rename_handler
          .rename_ident_if_conflict(&module_export_ident.module_id, &module_export_ident.ident);
      }

      let mut result = strip_module_decl(
        &module_id,
        module_ids,
        module_graph,
        &mut rename_handler,
        &mut module_export_ident_map,
      );

      // append:
      // ```js
      // var module_namespace = {
      //    default: a,
      //    ns: e_js_namespace_farm_internal_
      // }
      // ```
      // if module is used by export * as or import * as
      if export_ident_map.contains_key(EXPORT_NAMESPACE) {
        let top_level_mark = Mark::from_u32(module.meta.as_script().top_level_mark);
        result.ast.body.push(create_var_namespace_item(
          &module_id,
          top_level_mark,
          export_ident_map,
          cyclic_idents.get(&module_id).unwrap_or(&HashSet::default()),
          &mut delayed_rename,
        ));
      }
      strip_module_results.push((module_id, result));
    })
    .unwrap();
  }

  // handle delayed rename
  for (module_id, module_export_idents) in delayed_rename {
    for module_export_ident in module_export_idents {
      let final_ident = rename_handler
        .get_renamed_ident(&module_export_ident.module_id, &module_export_ident.ident)
        .unwrap_or(module_export_ident.ident.clone());

      if module_export_ident.ident != final_ident {
        // rename local to final_ident
        rename_handler.rename_ident(module_id.clone(), module_export_ident.ident, final_ident);
      }
    }
  }

  // rename all the identifiers in the AST
  strip_module_results
    .par_iter_mut()
    .for_each(|(module_id, result)| {
      let mut rename_visitor = unique_idents::RenameVisitor::new(module_id, &rename_handler);

      result
        .preserved_import_decls
        .iter_mut()
        .for_each(|(item, _)| {
          item.visit_mut_with(&mut rename_visitor);
        });

      result.ast.visit_mut_with(&mut rename_visitor);

      result
        .preserved_export_decls
        .iter_mut()
        .for_each(|item| item.visit_mut_with(&mut rename_visitor));
    });

  Ok(strip_module_results)
}

fn merge_stripped_module_asts(
  strip_module_results: Vec<strip_module_decl::StripModuleDeclResult>,
) -> (SwcModule, HashMap<(String, ResolveKind), ModuleId>) {
  let mut concatenated_ast = SwcModule {
    span: DUMMY_SP,
    body: vec![],
    shebang: None,
  };

  // get external modules from strip_module_decl_result
  let mut external_modules = HashMap::default();
  let mut preserved_import_decls = vec![];
  let mut preserved_export_decls = vec![];
  let mut new_body = vec![];

  // add external `import/export from` first
  for result in strip_module_results {
    new_body.extend(result.ast.body);
    preserved_import_decls.push(result.preserved_import_decls);
    preserved_export_decls.push(result.preserved_export_decls);
  }
  // external order should be reverse of the topo order
  preserved_import_decls.reverse();

  // extract external modules
  for (module_decl_item, module_id) in preserved_import_decls.into_iter().flatten() {
    let source_kind = if let Some(import) = module_decl_item
      .as_module_decl()
      .and_then(|decl| decl.as_import())
    {
      (import.src.value.to_string(), ResolveKind::Import)
    } else if let Some(src) = module_decl_item
      .as_module_decl()
      .and_then(|decl| decl.as_export_named())
      .and_then(|export| export.src.as_ref())
    {
      (src.value.to_string(), ResolveKind::ExportFrom)
    } else {
      continue;
    };

    external_modules.insert(source_kind, module_id);
    concatenated_ast.body.push(module_decl_item);
  }

  concatenated_ast.body.extend(new_body);
  concatenated_ast
    .body
    .extend(preserved_export_decls.into_iter().flatten());

  // 4. Return the concatenated result
  (concatenated_ast, external_modules)
}
