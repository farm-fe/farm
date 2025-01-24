use std::{collections::VecDeque, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::{ModuleId, ModuleMetaData},
  HashSet,
};
// use farmfe_toolkit::swc_ecma_utils::contains_top_level_await;

pub fn find_async_modules(context: &Arc<CompilationContext>) -> HashSet<ModuleId> {
  let module_graph = context.module_graph.read();
  let mut init_async_modules = HashSet::default();

  for module in module_graph.modules() {
    if let ModuleMetaData::Script(script_meta) = module.meta.as_ref() {
      // if contains_top_level_await(&script_meta.ast) {
      if script_meta.is_async {
        init_async_modules.insert(module.id.clone());
      }
    }
  }

  let mut queue = VecDeque::from(init_async_modules.into_iter().collect::<Vec<_>>());
  let mut async_modules = HashSet::default();

  while !queue.is_empty() {
    let module_id = queue.pop_front().unwrap();
    async_modules.insert(module_id.clone());

    for (dept, edge) in module_graph.dependents(&module_id) {
      if !async_modules.contains(&dept) && !edge.is_dynamic_import() && !edge.is_dynamic_entry() {
        queue.push_back(dept);
      }
    }
  }

  async_modules
}

pub fn update_async_modules(
  param: &farmfe_core::plugin::PluginModuleGraphUpdatedHookParam,
  context: &Arc<CompilationContext>,
) {
  // detect async module like top level await when module graph updated
  // module graph updated is called during compiler.update
  let module_graph = context.module_graph.read();
  let mut added_async_modules = vec![];
  // find added modules that contains top level await
  let mut analyze_top_level_await = |module_id: &ModuleId| {
    let module = module_graph.module(module_id).unwrap();

    if module.module_type.is_script() {
      let is_async = module.meta.as_script().is_async;
      let dependencies = module_graph.dependencies(module_id);
      let is_deps_async = dependencies.iter().any(|(dep_id, edge)| {
        let dep = module_graph.module(dep_id).unwrap();
        dep.meta.as_script().is_async && !edge.is_dynamic_import() && !edge.is_dynamic_entry()
      });
      if is_deps_async || is_async {
        added_async_modules.push(module_id.clone());
      }
    }
  };
  for added in &param.added_modules_ids {
    analyze_top_level_await(added);
  }
  for updated in &param.updated_modules_ids {
    analyze_top_level_await(updated);
  }

  let mut queue = VecDeque::from(added_async_modules.into_iter().collect::<Vec<_>>());
  let mut async_modules = HashSet::default();

  while !queue.is_empty() {
    let module_id = queue.pop_front().unwrap();
    async_modules.insert(module_id.clone());

    for (dept, edge) in module_graph.dependents(&module_id) {
      if !async_modules.contains(&dept) && !edge.is_dynamic_import() && !edge.is_dynamic_entry() {
        queue.push_back(dept);
      }
    }
  }

  drop(module_graph);

  // update async modules
  let mut module_graph = context.module_graph.write();

  for module_id in async_modules {
    let module = module_graph.module_mut(&module_id).unwrap();
    module.meta.as_script_mut().is_async = true;
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use farmfe_core::{
    context::CompilationContext,
    module::{meta_data::script::ScriptModuleMetaData, ModuleMetaData, ModuleType},
    parking_lot::RwLock,
    swc_common::DUMMY_SP,
    swc_ecma_ast::{AwaitExpr, Expr, ExprStmt, Lit, Module, ModuleItem, Stmt},
    HashSet,
  };
  use farmfe_testing_helpers::construct_test_module_graph;

  #[test]
  fn test_find_async_modules() {
    let mut module_graph = construct_test_module_graph();
    module_graph.modules_mut().into_iter().for_each(|module| {
      module.module_type = ModuleType::Js;
      module.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData::default()));
      module.meta.as_script_mut().ast = Module {
        body: vec![],
        span: DUMMY_SP,
        shebang: None,
      };
    });
    let module_c = module_graph.module_mut(&"C".into()).unwrap();
    module_c.meta.as_script_mut().ast = Module {
      body: vec![ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        expr: Box::new(Expr::Await(AwaitExpr {
          arg: Box::new(Expr::Lit(Lit::Str("any".into()))),
          span: DUMMY_SP,
        })),
        span: DUMMY_SP,
      }))],
      span: DUMMY_SP,
      shebang: None,
    };

    let mut context = CompilationContext::new(Default::default(), vec![]).unwrap();
    context.module_graph = Box::new(RwLock::new(module_graph));

    let async_modules = super::find_async_modules(&Arc::new(context));
    println!("{:#?}", async_modules);
    assert_eq!(
      async_modules,
      HashSet::from_iter(vec!["C".into(), "F".into(), "A".into()])
    );
  }
}
