use farmfe_core::{
  plugin::{PluginAnalyzeDepsHookResultEntry, ResolveKind},
  swc_ecma_ast::{Module, ModuleDecl, ModuleItem},
};

use farmfe_toolkit::swc_ecma_visit::{Visit, VisitWith};

pub struct DepsAnalyzer<'a> {
  ast: &'a Module,
  deps: Option<Vec<PluginAnalyzeDepsHookResultEntry>>,
}

impl<'a> DepsAnalyzer<'a> {
  pub fn new(ast: &'a Module) -> Self {
    Self { ast, deps: None }
  }

  pub fn analyze_deps(&mut self) -> Vec<PluginAnalyzeDepsHookResultEntry> {
    self.ast.visit_with(self);
    self.deps.take().unwrap()
  }

  fn insert_dep(&mut self, dep: PluginAnalyzeDepsHookResultEntry) {
    if let Some(deps) = &mut self.deps {
      deps.push(dep);
    } else {
      self.deps.replace(vec![dep]);
    }
  }
}

impl<'a> Visit for DepsAnalyzer<'a> {
  fn visit_module_item(&mut self, n: &ModuleItem) {
    match n {
      ModuleItem::ModuleDecl(ModuleDecl::Import(import)) => {
        self.insert_dep(PluginAnalyzeDepsHookResultEntry {
          source: import.src.value.to_string(),
          kind: ResolveKind::Import,
        });
      }
      _ => { /* TODO support analyze cjs require and dynamic import */ }
    }
  }
}
