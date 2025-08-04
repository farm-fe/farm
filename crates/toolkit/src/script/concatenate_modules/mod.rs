use std::{cell::RefCell, rc::Rc, sync::Arc};

use dynamic_import::DynamicImportVisitor;
use farmfe_core::{
  config::comments::CommentsConfig,
  context::CompilationContext,
  module::{
    meta_data::script::{
      statement::SwcId, CommentsMetaData, CommentsMetaDataItem, ModuleExportIdent,
      ModuleReExportIdentType, ScriptModuleMetaData,
    },
    module_graph::ModuleGraph,
    ModuleId, ModuleSystem,
  },
  parking_lot::Mutex,
  plugin::ResolveKind,
  rayon::iter::{IntoParallelRefMutIterator, ParallelIterator},
  swc_common::{
    comments::{Comment, CommentKind},
    Globals, Mark, SourceMap, Span, DUMMY_SP, GLOBALS,
  },
  swc_ecma_ast::{EmptyStmt, Module as SwcModule, ModuleItem, Stmt},
  HashMap, HashSet,
};

use strip_module_decl::{strip_module_decl, PreservedImportDeclItem, StripModuleDeclResult};
use swc_ecma_visit::VisitMutWith;
use unique_idents::TopLevelIdentsRenameHandler;
pub use unique_idents::EXPORT_NAMESPACE;
use utils::{create_var_namespace_item, generate_export_decl_item};

use crate::script::concatenate_modules::{
  strip_module_decl::{
    handle_external_module_idents, is_ident_reexported_from_external_module,
    StripModuleDeclStatementParams,
  },
  utils::should_add_namespace_ident,
};

use super::{
  merge_swc_globals::{merge_comments, merge_sourcemap},
  swc_try_with::{resolve_module_mark, try_with},
};

mod dynamic_import;
mod handle_external_modules;
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

#[derive(Debug, Default, Clone)]
pub struct ConcatenateModulesAstOptions {
  /// Whether to check if the module is esm
  pub check_esm: bool,
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
  options: ConcatenateModulesAstOptions,
  context: &Arc<CompilationContext>,
) -> Result<ConcatenateModulesAstResult, String> {
  let StripModulesAstsResult {
    strip_module_results,
    dynamic_external_modules,
    strip_context,
  } = strip_modules_asts(
    entry_module_id,
    module_ids,
    module_graph,
    options.check_esm,
    context,
  )?;

  let mut comments = vec![];
  let mut module_asts = vec![];
  let mut pre_post_items = vec![];

  let mut sorted_modules = vec![];
  let merged_globals = Globals::new();

  for (module_id, mut stripped_module) in strip_module_results {
    if matches!(context.config.comments, box CommentsConfig::Bool(true)) {
      // insert comment to the top of the module: // module_id: xxxx
      GLOBALS.set(&merged_globals, || {
        let span = Span::dummy_with_cmt();
        stripped_module.comments.trailing.insert(
          0,
          CommentsMetaDataItem {
            byte_pos: span.hi,
            comment: vec![Comment {
              kind: CommentKind::Line,
              span: DUMMY_SP,
              text: format!(" module_id: {}", module_id.id(context.config.mode)).into(),
            }],
          },
        );
        stripped_module
          .items_to_prepend
          .insert(0, ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span })));
      });
    }

    comments.push((module_id.clone(), stripped_module.comments));
    module_asts.push((module_id.clone(), stripped_module.ast));
    pre_post_items.push((
      module_id.clone(),
      (
        stripped_module.items_to_prepend,
        stripped_module.items_to_append,
      ),
    ));

    sorted_modules.push(module_id);
  }

  let merged_source_map = merge_sourcemap(&mut module_asts, Default::default(), context);
  let merged_comments = merge_comments(&mut comments, merged_source_map.clone()).into();

  // merge comments, module_asts and preserved_module_decls back to strip_module_results
  let stripped_results = comments
    .into_iter()
    .zip(module_asts.into_iter())
    .zip(pre_post_items)
    .map(
      |(((_, comments), (_, ast)), (_, (items_to_prepend, items_to_append)))| {
        StripModuleDeclResult {
          comments,
          ast,
          items_to_prepend,
          items_to_append,
        }
      },
    )
    .collect::<Vec<_>>();

  let (mut concatenated_ast, mut external_modules) =
    merge_stripped_module_asts(stripped_results, module_graph, strip_context);
  // extend dynamic external modules
  external_modules.extend(dynamic_external_modules);

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

struct StripModulesAstsResult {
  strip_module_results: Vec<(ModuleId, StripModuleDeclResult)>,
  dynamic_external_modules: HashMap<(String, ResolveKind), ModuleId>,
  strip_context: StripModuleContext,
}

struct PreservedExportDeclItem {
  pub export_item: ModuleItem,
  pub source_module_id: Option<ModuleId>,
}

impl PreservedExportDeclItem {
  pub fn new(export_item: ModuleItem, source_module_id: Option<ModuleId>) -> Self {
    Self {
      export_item,
      source_module_id,
    }
  }
}

struct StripModuleContext {
  pub should_add_namespace_ident: HashSet<ModuleId>,
  pub rename_handler: Rc<RefCell<unique_idents::TopLevelIdentsRenameHandler>>,

  /// the preserved import or export from statements that are external modules
  pub preserved_import_decls: Vec<PreservedImportDeclItem>,
  /// the preserved export statements traced from the entry module
  pub preserved_export_decls: Vec<PreservedExportDeclItem>,
  /// extra var decls that should be appended after import decls
  /// (source_module_id, export_str, var_decl)
  pub extra_var_decls: Vec<(SwcId, SwcId, ModuleItem)>,
  /// `import { foo }` where foo can not be found
  pub unresolved_imported_ident_map: HashMap<ModuleId, HashSet<(Option<SwcId>, ModuleExportIdent)>>,
}

fn strip_modules_asts(
  entry_module_id: &ModuleId,
  module_ids: &HashSet<ModuleId>,
  module_graph: &ModuleGraph,
  check_esm: bool,
  context: &Arc<CompilationContext>,
) -> Result<StripModulesAstsResult, String> {
  // 1. Sort the modules by execution order
  let mut sorted_modules: Vec<_> = module_ids.iter().cloned().collect();
  sorted_modules.sort_by_key(|module_id| module_graph.module(module_id).unwrap().execution_order);
  // cyclic_idents should be accessed by get method, we should collect cyclic idents from cyclic modules first
  let mut cyclic_idents =
    HashMap::<ModuleId, HashSet<(Option<SwcId>, ModuleExportIdent)>>::default();

  // 2. Check if the module is esm, panic if it is not
  for module_id in &sorted_modules {
    let module = module_graph.module(module_id).unwrap();
    // error if it is no script module
    if !module.module_type.is_script() {
      return Err(format!(
        "Module {} is not script module. Only script module is supported when concatenating modules",
        module_id.to_string()
      ));
    }

    // error if it is not ESM
    if check_esm && module.meta.as_script().module_system != ModuleSystem::EsModule {
      return Err(format!(
        "Module {} is not ESM module. Only ESM modules are supported when concatenating modules",
        module_id.to_string()
      ));
    }

    if module_graph.circle_record.is_in_circle(module_id) {
      cyclic_idents.entry(module_id.clone()).or_default().extend(
        module
          .meta
          .as_script()
          .export_ident_map
          .values()
          .cloned()
          .into_iter()
          .map(|e| (None, e)),
      );
    }
  }

  // 3. Visit sorted modules and process them
  let rename_handler = unique_idents::init_rename_handler(&sorted_modules, module_graph);
  let mut strip_context = StripModuleContext {
    should_add_namespace_ident: HashSet::default(),
    rename_handler: Rc::new(RefCell::new(rename_handler)),
    preserved_import_decls: vec![],
    preserved_export_decls: vec![],
    extra_var_decls: vec![],
    unresolved_imported_ident_map: HashMap::default(),
  };

  let mut strip_module_results = vec![];

  for module_id in sorted_modules {
    let module = module_graph.module(&module_id).unwrap();
    let cm = context.meta.get_module_source_map(&module_id);

    try_with(cm, context.meta.get_globals(&module_id).value(), || {
      let export_ident_map = &module.meta.as_script().export_ident_map;
      // rename module_namespace if there are conflicts
      if let Some(module_export_ident) = export_ident_map.get(EXPORT_NAMESPACE) {
        let module_export_ident = module_export_ident.as_internal();
        let mut rename_handler = strip_context.rename_handler.borrow_mut();
        rename_handler
          .rename_ident_if_conflict(&module_export_ident.module_id, &module_export_ident.ident);
      }

      let should_add_export_namespace_item =
        should_add_namespace_ident(&module_id, export_ident_map);

      if should_add_export_namespace_item {
        strip_context
          .should_add_namespace_ident
          .insert(module_id.clone());
      }

      let result = strip_module_decl(
        &module_id,
        module_ids,
        module_id == *entry_module_id,
        module_graph,
        &mut strip_context,
      );

      strip_module_results.push((module_id, result));
    })
    .unwrap();
  }

  let mut rename_handler = strip_context
    .rename_handler
    .replace(TopLevelIdentsRenameHandler::default());

  let unresolved_imported_ident_map =
    std::mem::take(&mut strip_context.unresolved_imported_ident_map);
  cyclic_idents.extend(unresolved_imported_ident_map);

  // handle delayed rename
  for (module_id, module_export_idents) in &cyclic_idents {
    for (ident, module_export_ident) in module_export_idents {
      let module_export_ident = module_export_ident.as_internal();

      // for export ident rename, should only rename the ident defined in current module
      if ident.is_none() && *module_id != module_export_ident.module_id {
        continue;
      }

      let final_ident = rename_handler
        .get_renamed_ident(&module_export_ident.module_id, &module_export_ident.ident)
        .unwrap_or_else(|| {
            if ident.is_some() {
              println!(
                "[Farm warn] rename imported ident failed (module_id: {:?}), please make sure export {:?} is defined in {:?}",
                module_id.to_string(),
                module_export_ident.ident,
                module_export_ident.module_id
              );
            }
            module_export_ident.ident.clone()
        });

      let ident_to_rename = ident.as_ref().unwrap_or(&module_export_ident.ident);

      if *ident_to_rename != final_ident {
        // rename local to final_ident
        rename_handler.rename_ident(module_id.clone(), ident_to_rename.clone(), final_ident);
      }
    }
  }

  // append var namespace item
  for (module_id, result) in &mut strip_module_results {
    // handle reexport ident map for namespace ident or entry module
    if strip_context.should_add_namespace_ident.contains(module_id) || module_id == entry_module_id
    {
      let module = module_graph.module(module_id).unwrap();
      let module_meta = module.meta.as_script();

      handle_external_reexport_idents(
        entry_module_id,
        module_id,
        module_ids,
        module_meta,
        result,
        &mut strip_context,
        module_graph,
        &rename_handler,
      );
    }

    if strip_context.should_add_namespace_ident.contains(module_id) {
      let module = module_graph.module(module_id).unwrap();
      let module_meta = module.meta.as_script();
      let export_ident_map = &module_meta.export_ident_map;

      // append:
      // ```js
      // var module_namespace = {
      //    default: a,
      //    ns: e_js_namespace_farm_internal_
      // }
      // ```
      // if module is used by export * as or import * as or import('...')
      result.ast.body.push(create_var_namespace_item(
        &module_id,
        export_ident_map,
        cyclic_idents.get(&module_id).unwrap_or(&HashSet::default()),
        &rename_handler,
      ));
    }
  }

  let dynamic_external_modules = Mutex::new(HashMap::default());

  // for entry module, add re-export
  // get export info from the entry module
  let entry_module = module_graph.module(entry_module_id).unwrap();
  let entry_module_export_ident_map = entry_module.meta.as_script().get_export_idents();

  if entry_module_export_ident_map.len() > 0 {
    let item = generate_export_decl_item(entry_module_export_ident_map, &rename_handler);
    strip_context
      .preserved_export_decls
      .push(PreservedExportDeclItem::new(item, None));
  }

  // handle dynamic import in parallel
  strip_module_results
    .par_iter_mut()
    .for_each(|(module_id, result)| {
      let cm = context.meta.get_module_source_map(&module_id);

      try_with(cm, context.meta.get_globals(module_id).value(), || {
        let mut dynamic_import_visitor =
          DynamicImportVisitor::new(module_id, &module_graph, &module_ids, &rename_handler);
        result.ast.visit_mut_with(&mut dynamic_import_visitor);

        dynamic_external_modules
          .lock()
          .extend(dynamic_import_visitor.external_modules);
      })
      .unwrap();
    });

  let dynamic_external_modules = dynamic_external_modules.into_inner();

  // rename all the identifiers in the AST
  strip_module_results
    .par_iter_mut()
    .for_each(|(module_id, result)| {
      let mut rename_visitor = unique_idents::RenameVisitor::new(module_id, &rename_handler);

      result.items_to_prepend.visit_mut_with(&mut rename_visitor);
      result.ast.visit_mut_with(&mut rename_visitor);
      result.items_to_append.visit_mut_with(&mut rename_visitor);
    });

  Ok(StripModulesAstsResult {
    strip_module_results,
    dynamic_external_modules,
    strip_context,
  })
}

fn merge_stripped_module_asts(
  strip_module_results: Vec<strip_module_decl::StripModuleDeclResult>,
  module_graph: &ModuleGraph,
  strip_context: StripModuleContext,
) -> (SwcModule, HashMap<(String, ResolveKind), ModuleId>) {
  let mut concatenated_ast = SwcModule {
    span: DUMMY_SP,
    body: vec![],
    shebang: None,
  };

  // get external modules from strip_module_decl_result
  let mut external_modules = HashMap::default();
  let preserved_import_decls = strip_context.preserved_import_decls;

  let mut new_body = vec![];

  // add external `import/export from` first
  for result in strip_module_results {
    new_body.extend(result.items_to_prepend);
    new_body.extend(result.ast.body);
    new_body.extend(result.items_to_append);
  }

  // extract external modules
  for item in preserved_import_decls.iter() {
    let source_kind = if let Some(import) = item
      .import_item
      .as_module_decl()
      .and_then(|decl| decl.as_import())
    {
      (import.src.value.to_string(), ResolveKind::Import)
    } else if let Some(src) = item
      .import_item
      .as_module_decl()
      .and_then(|decl| decl.as_export_named())
      .and_then(|export| export.src.as_ref())
    {
      (src.value.to_string(), ResolveKind::ExportFrom)
    } else if let Some(export_all) = item
      .import_item
      .as_module_decl()
      .and_then(|decl| decl.as_export_all())
    {
      (export_all.src.value.to_string(), ResolveKind::ExportFrom)
    } else {
      continue;
    };

    external_modules.insert(source_kind, item.source_module_id.clone());
    // concatenated_ast.body.push(item.import_item);
  }

  let preserved_export_decls = strip_context.preserved_export_decls;

  let mut preserved_decls = vec![];
  preserved_decls.extend(
    preserved_import_decls
      .into_iter()
      .map(|i| (i.import_item, i.source_module_id)),
  );
  // export { xx } that does not contain from clause
  let mut separate_export_decls = vec![];

  for preserved_export in preserved_export_decls {
    if let Some(source_module_id) = preserved_export.source_module_id {
      preserved_decls.push((preserved_export.export_item, source_module_id));
    } else {
      separate_export_decls.push(preserved_export.export_item);
    }
  }

  // order of external import item should be the same as execution order
  preserved_decls.sort_by(|(_, a), (_, b)| {
    let module_a_order = module_graph
      .module(a)
      .map(|m| m.execution_order)
      .unwrap_or(0);
    let module_b_order = module_graph
      .module(b)
      .map(|m| m.execution_order)
      .unwrap_or(0);

    module_a_order.cmp(&module_b_order)
  });

  concatenated_ast
    .body
    .extend(preserved_decls.into_iter().map(|(i, _)| i));
  concatenated_ast
    .body
    .extend(strip_context.extra_var_decls.into_iter().map(|(_, _, i)| i));

  concatenated_ast.body.extend(new_body);
  concatenated_ast.body.extend(separate_export_decls);

  // 4. Return the concatenated result
  (concatenated_ast, external_modules)
}

/// For `export * from` and `export { foo } from`, the export ident may be declared in an external module(because of resources split instead of configure the module as external)
/// For this case, we should find the boundary module of the external ident and treat it the same as external module
fn handle_external_reexport_idents(
  entry_module_id: &ModuleId,
  module_id: &ModuleId,
  module_ids: &HashSet<ModuleId>,
  module_meta: &ScriptModuleMetaData,
  result: &mut StripModuleDeclResult,
  strip_context: &mut StripModuleContext,
  module_graph: &ModuleGraph,
  rename_handler: &TopLevelIdentsRenameHandler,
) {
  if module_meta.reexport_ident_map.is_empty() {
    return;
  }

  let mut external_module_idents_map = HashMap::default();

  for (export_str, reexport_type) in &module_meta.reexport_ident_map {
    let source_module_id = match reexport_type {
      ModuleReExportIdentType::FromExportAll(from_module_id) => from_module_id,
      ModuleReExportIdentType::FromExportNamed { from_module_id, .. } => from_module_id,
    };
    let ident: SwcId = export_str.as_str().into();
    if let Some(first_external_module_id) = is_ident_reexported_from_external_module(
      module_ids,
      source_module_id,
      &ident,
      export_str,
      module_graph,
      &rename_handler,
      &mut HashSet::default(),
    ) {
      external_module_idents_map
        .entry(first_external_module_id)
        .or_insert(vec![])
        .push((ident, export_str.to_string()));
    }
  }

  let mut params = StripModuleDeclStatementParams {
    module_id,
    module_ids,
    script_meta: module_meta,
    result,
    strip_context,
    is_entry_module: module_id == entry_module_id,
    module_graph,
  };

  let mut statements_to_remove = vec![];

  // handle external module idents
  for (source_module_id, idents) in external_module_idents_map {
    statements_to_remove.extend(handle_external_module_idents(
      &mut params,
      &source_module_id,
      idents,
    ));
  }

  statements_to_remove.reverse();
  statements_to_remove.into_iter().for_each(|stmt_id| {
    result.ast.body.remove(stmt_id);
  });
}
