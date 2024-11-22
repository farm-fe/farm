use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::ModuleSystem,
  plugin::{PluginFinalizeModuleHookParam, ResolveKind},
  swc_common::Mark,
  swc_ecma_ast::{Expr, Ident, IdentName, MemberProp, Module as SwcModule, ModuleItem, Stmt},
};
use swc_ecma_visit::{Visit, VisitWith};

use crate::common::{create_swc_source_map, Source};

pub use farmfe_toolkit_plugin_types::swc_ast::ParseScriptModuleResult;

use crate::script::swc_try_with::try_with;

pub fn module_system_from_deps(deps: Vec<ResolveKind>) -> ModuleSystem {
  let mut module_system = ModuleSystem::Custom(String::from("unknown"));

  for resolve_kind in deps {
    if matches!(resolve_kind, ResolveKind::Import)
      || matches!(resolve_kind, ResolveKind::DynamicImport)
      || matches!(resolve_kind, ResolveKind::ExportFrom)
    {
      match module_system {
        ModuleSystem::EsModule => continue,
        ModuleSystem::CommonJs => {
          module_system = ModuleSystem::Hybrid;
          break;
        }
        _ => module_system = ModuleSystem::EsModule,
      }
    } else if matches!(resolve_kind, ResolveKind::Require) {
      match module_system {
        ModuleSystem::CommonJs => continue,
        ModuleSystem::EsModule => {
          module_system = ModuleSystem::Hybrid;
          break;
        }
        _ => module_system = ModuleSystem::CommonJs,
      }
    }
  }

  module_system
}

struct ModuleSystemAnalyzer {
  unresolved_mark: Mark,
  contain_module_exports: bool,
  contain_esm: bool,
}

impl Visit for ModuleSystemAnalyzer {
  fn visit_stmts(&mut self, n: &[Stmt]) {
    if self.contain_module_exports || self.contain_esm {
      return;
    }

    n.visit_children_with(self);
  }

  fn visit_member_expr(&mut self, n: &farmfe_core::swc_ecma_ast::MemberExpr) {
    if self.contain_module_exports {
      return;
    }

    if let box Expr::Ident(Ident { sym, ctxt, .. }) = &n.obj {
      if sym == "module" && ctxt.outer() == self.unresolved_mark {
        if let MemberProp::Ident(IdentName { sym, .. }) = &n.prop {
          if sym == "exports" {
            self.contain_module_exports = true;
          }
        }
      } else if sym == "exports" && ctxt.outer() == self.unresolved_mark {
        self.contain_module_exports = true;
      } else {
        n.visit_children_with(self);
      }
    } else {
      n.visit_children_with(self);
    }
  }

  fn visit_module_decl(&mut self, n: &farmfe_core::swc_ecma_ast::ModuleDecl) {
    if self.contain_esm {
      return;
    }

    self.contain_esm = true;

    n.visit_children_with(self);
  }
}

pub fn module_system_from_ast(ast: &SwcModule, module_system: ModuleSystem) -> ModuleSystem {
  if module_system != ModuleSystem::Hybrid {
    // if the ast contains ModuleDecl, it's a esm module
    for item in ast.body.iter() {
      if let ModuleItem::ModuleDecl(_) = item {
        if module_system == ModuleSystem::CommonJs {
          return ModuleSystem::Hybrid;
        } else {
          return ModuleSystem::EsModule;
        }
      }
    }
  }

  module_system
}

pub fn set_module_system_for_module_meta(
  param: &mut PluginFinalizeModuleHookParam,
  context: &Arc<CompilationContext>,
) {
  // default to commonjs
  let module_system_from_deps_option = if !param.deps.is_empty() {
    module_system_from_deps(param.deps.iter().map(|d| d.kind.clone()).collect())
  } else {
    ModuleSystem::UnInitial
  };

  // param.module.meta.as_script_mut().module_system = module_system.clone();

  let ast = &param.module.meta.as_script().ast;

  let mut module_system_from_ast: ModuleSystem = ModuleSystem::UnInitial;
  {
    // try_with(param.module.meta.as_script().comments.into(), globals, op)

    let (cm, _) = create_swc_source_map(Source {
      path: PathBuf::from(&param.module.id.to_string()),
      content: param.module.content.clone(),
    });

    try_with(cm, &context.meta.script.globals, || {
      let unresolved_mark = Mark::from_u32(param.module.meta.as_script().unresolved_mark);
      let mut analyzer = ModuleSystemAnalyzer {
        unresolved_mark,
        contain_module_exports: false,
        contain_esm: false,
      };

      ast.visit_with(&mut analyzer);

      if analyzer.contain_module_exports {
        module_system_from_ast = module_system_from_ast.merge(ModuleSystem::CommonJs);
      }

      if analyzer.contain_esm {
        module_system_from_ast = module_system_from_ast.merge(ModuleSystem::EsModule);
      }
    })
    .unwrap();
  }

  let mut v = [module_system_from_deps_option, module_system_from_ast]
    .into_iter()
    .reduce(|a, b| a.merge(b))
    .unwrap_or(ModuleSystem::UnInitial);

  if matches!(v, ModuleSystem::UnInitial) {
    v = ModuleSystem::Hybrid;
  }

  param.module.meta.as_script_mut().module_system = v;
}
