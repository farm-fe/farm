use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{
    meta_data::script::statement::Statement,
    module_graph::{ModuleGraph, ModuleGraphEdge, ModuleGraphEdgeDataItem},
    ModuleId,
  },
  plugin::ResolveKind,
  swc_common::{Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    CallExpr, Callee, Expr, ExprOrSpread, ExprStmt, Ident, ImportDecl, ImportNamedSpecifier,
    ImportSpecifier, ModuleDecl, ModuleExportName, ModuleItem, Stmt,
  },
  HashSet,
};
use farmfe_toolkit::{
  lazy_static::lazy_static,
  runtime::RuntimeFeatureGuardRemover,
  script::{
    analyze_statement::{analyze_statement_info, AnalyzedStatementInfo},
    swc_try_with::try_with,
  },
  swc_ecma_visit::VisitMutWith,
};

use crate::{RUNTIME_INPUT_SCOPE, RUNTIME_PACKAGE};

lazy_static! {
  /// The order of the dynamic inputs is important, the first one is reserved for module system
  /// and the rest will be initialized in order
  static ref DYNAMIC_INPUTS: Vec<String> = vec![
    format!("{RUNTIME_INPUT_SCOPE}_module_system"),
    format!("{RUNTIME_INPUT_SCOPE}_dynamic_import"),
    format!("{RUNTIME_INPUT_SCOPE}_plugin"),
    format!("{RUNTIME_INPUT_SCOPE}_module_system_helper"),
    format!("{RUNTIME_INPUT_SCOPE}_module_helper"),
  ];
}

const MODULE_SYSTEM: &str = "moduleSystem";
const INIT_MODULE_SYSTEM: &str = "initModuleSystem";

fn create_ident(name: &str, mark: Mark, index: Option<usize>) -> Ident {
  Ident::new(
    if let Some(index) = index {
      format!("{name}{index}").as_str().into()
    } else {
      name.into()
    },
    DUMMY_SP,
    SyntaxContext::empty().apply_mark(mark),
  )
}

fn try_get_dynamic_input_entry(input: &str, module_graph: &ModuleGraph) -> Option<ModuleId> {
  module_graph
    .dynamic_entries
    .iter()
    .find(|(_, i)| *i == input)
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
  let top_level_mark = Mark::from_u32(entry_module.meta.as_script().top_level_mark);
  let ast = &mut entry_module.meta.as_script_mut().ast;

  // the first import statement is reserved for module system
  let is_module_system_runtime = index == 0;
  let imported_ident = if is_module_system_runtime {
    create_ident(MODULE_SYSTEM, top_level_mark, None)
  } else {
    create_ident(INIT_MODULE_SYSTEM, top_level_mark, Some(index))
  };

  ast.body.insert(
    index,
    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
      span: DUMMY_SP,
      specifiers: vec![
        // import { initModuleSystem0 } '/xxx/runtime/dynamic-import'
        ImportSpecifier::Named(ImportNamedSpecifier {
          span: DUMMY_SP,
          local: imported_ident,
          imported: if !is_module_system_runtime {
            Some(ModuleExportName::Ident(create_ident(
              INIT_MODULE_SYSTEM,
              top_level_mark,
              None,
            )))
          } else {
            None
          },
          is_type_only: false,
        }),
      ],
      src: Box::new(dynamic_entry.id(context.config.mode).into()),
      type_only: false,
      with: None,
      phase: Default::default(),
    })),
  );
  // update meta.statements
  let AnalyzedStatementInfo {
    export_info,
    import_info,
    defined_idents,
    top_level_await,
  } = analyze_statement_info(&index, &ast.body[index]);
  let statements = &mut entry_module.meta.as_script_mut().statements;
  statements.insert(
    index,
    Statement::new(
      index,
      export_info,
      import_info,
      defined_idents,
      top_level_await,
    ),
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
/// import { moduleSystem } from '/xxx/runtime/module-system'
/// import { initModuleSystem1 } '/xxx/runtime/dynamic-import'
/// import { initModuleSystem2 } '/xxx/runtime/plugin'
/// /// ...
///
/// initModuleSystem1(moduleSystem)
/// initModuleSystem2(moduleSystem)
/// ```
pub fn insert_runtime_modules(module_graph: &mut ModuleGraph, context: &Arc<CompilationContext>) {
  let entry_module_id: ModuleId = RUNTIME_PACKAGE.into();

  let (cm, top_level_mark) = {
    let entry_module = module_graph.module(&entry_module_id).unwrap();
    let (cm, _) = context
      .meta
      .create_swc_source_map(&entry_module_id, entry_module.content.clone());
    let top_level_mark = Mark::from_u32(entry_module.meta.as_script().top_level_mark);

    (cm, top_level_mark)
  };

  try_with(cm.clone(), &context.meta.script.globals, || {
    // 1. remove runtime dynamic entry
    // 2. insert import { moduleSystem } from '/xxx/runtime/module-system' at first import statement
    //    and import { initModuleSystem1 } '/xxx/runtime/dynamic-import', ... at the rest import statements

    let filtered_dynamic_entries = DYNAMIC_INPUTS
      .iter()
      .filter_map(|input| try_get_dynamic_input_entry(input, module_graph))
      .collect::<Vec<_>>();

    // collect statements before remove runtime dynamic entry
    let mut stmts = vec![];

    // insert initModuleSystem0(moduleSystem) initModuleSystem1(moduleSystem) ...
    for (index, _) in filtered_dynamic_entries.iter().enumerate() {
      // skip module system
      if index == 0 {
        continue;
      }

      // initModuleSystem1(moduleSystem)
      stmts.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Call(CallExpr {
          span: DUMMY_SP,
          callee: Callee::Expr(Box::new(Expr::Ident(create_ident(
            INIT_MODULE_SYSTEM,
            top_level_mark,
            Some(index),
          )))),
          args: vec![ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Ident(create_ident(
              MODULE_SYSTEM,
              top_level_mark,
              None,
            ))),
          }],
          type_args: None,
          ctxt: SyntaxContext::empty(),
        })),
      })));
    }

    for (index, entry) in filtered_dynamic_entries.iter().enumerate() {
      insert_dynamic_input_import(entry.clone(), index, module_graph, context);
    }

    let entry_module = module_graph.module_mut(&entry_module_id).unwrap();
    let statements = &entry_module.meta.as_script_mut().statements;

    let mut last_import_index = 0;

    for (index, statement) in statements.iter().enumerate() {
      if statement.import_info.is_some() {
        last_import_index = index + 1;
      }
    }

    let ast = &mut entry_module.meta.as_script_mut().ast;
    ast.body.splice(last_import_index..last_import_index, stmts);
  })
  .unwrap();
}

/// Remove unused runtime features that controlled by feature guard like `if (__FARM_TARGET_ENV__)`
pub fn remove_unused_runtime_features(
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) {
  let mut all_features_flags = HashSet::default();

  for module in module_graph.modules() {
    if !module.module_type.is_script() {
      continue;
    }

    let module = module_graph.module(&module.id).unwrap();
    let meta = module.meta.as_script();
    all_features_flags.extend(meta.feature_flags.iter().cloned());
  }

  // traverse all dynamic entries
  for input in DYNAMIC_INPUTS.iter() {
    if let Some(module_id) = try_get_dynamic_input_entry(input, module_graph) {
      let module = module_graph.module_mut(&module_id).unwrap();
      // get runtime entry module meta
      let meta = module.meta.as_script_mut();
      // init runtime feature guard remover
      let mut remover = RuntimeFeatureGuardRemover::new(&all_features_flags, context);
      let (cm, _) = context
        .meta
        .create_swc_source_map(&module_id, module.content.clone());

      try_with(cm, &context.meta.script.globals, || {
        meta.ast.visit_mut_with(&mut remover);
      })
      .unwrap();
    }
  }
}
