use std::{collections::VecDeque, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::meta_data::script::{
    statement::{Statement, StatementId, SwcId},
    FARM_RUNTIME_MODULE_HELPER_ID,
  },
  plugin::{GeneratedResource, PluginHookContext},
  resource::resource_pot::ResourcePot,
  swc_ecma_ast::{ImportSpecifier, Module, ModuleDecl, ModuleItem, Stmt},
  HashSet,
};
use farmfe_toolkit::{
  script::analyze_statement::analyze_statements,
  swc_ecma_visit::{Visit, VisitWith},
};

pub fn emit_resource_pot(
  resource_pot: &mut ResourcePot,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<Vec<GeneratedResource>> {
  context
    .plugin_driver
    .generate_resources(resource_pot, context, hook_context).map(|v| v.unwrap_or_else(|| {
      panic!("generate_resources does not emit valid resource for {:?}, please check you plugin config", resource_pot.id)
    }).resources)
}

pub fn inject_farm_runtime_helpers(
  runtime_module_helper_ast: &Module,
  used_helper_idents: &HashSet<String>,
) -> Vec<ModuleItem> {
  let statements = analyze_statements(runtime_module_helper_ast);
  let preserved_statements = find_used_statements(
    statements,
    &runtime_module_helper_ast.body,
    used_helper_idents,
  );

  runtime_module_helper_ast
    .clone()
    .body
    .into_iter()
    .enumerate()
    .filter_map(|(idx, item)| {
      if preserved_statements.contains(&idx) {
        Some(match item {
          ModuleItem::ModuleDecl(module_decl) => match module_decl {
            farmfe_core::swc_ecma_ast::ModuleDecl::ExportDecl(export_decl) => {
              ModuleItem::Stmt(Stmt::Decl(export_decl.decl))
            }
            _ => unreachable!(),
          },
          ModuleItem::Stmt(stmt) => ModuleItem::Stmt(stmt),
        })
      } else {
        None
      }
    })
    .collect()
}

pub fn strip_runtime_module_helper_import(module: &mut Module) -> HashSet<String> {
  let mut imports_to_remove = vec![];
  let mut used_helper_idents = HashSet::default();

  for (i, item) in module.body.iter().enumerate() {
    if let ModuleItem::ModuleDecl(module_decl) = item {
      if let ModuleDecl::Import(import_decl) = module_decl {
        if import_decl.src.value == FARM_RUNTIME_MODULE_HELPER_ID {
          for specifier in &import_decl.specifiers {
            if let ImportSpecifier::Named(named_specifier) = specifier {
              used_helper_idents.insert(named_specifier.local.sym.to_string());
            }
          }

          imports_to_remove.push(i);
        }
      }
    }
  }

  imports_to_remove.reverse();

  for i in imports_to_remove {
    module.body.remove(i);
  }

  used_helper_idents
}

fn find_used_statements(
  mut statements: Vec<Statement>,
  items: &Vec<ModuleItem>,
  used_helper_idents: &HashSet<String>,
) -> Vec<StatementId> {
  let mut queue = VecDeque::new();

  for stmt in &mut statements {
    if stmt
      .defined_idents
      .iter()
      .any(|i| used_helper_idents.contains(i.sym.as_str()))
    {
      queue.push_back(stmt.id)
    }

    let mut analyzer = UsedDefinedIdentsAnalyzer {
      user_defined_idents: HashSet::default(),
    };
    items[stmt.id].visit_with(&mut analyzer);
    stmt.used_defined_idents = analyzer.user_defined_idents;
  }

  let mut result = vec![];

  while !queue.is_empty() {
    let idx = queue.pop_front().unwrap();

    if result.contains(&idx) {
      continue;
    }

    result.push(idx);

    let stmt = &statements[idx];
    let dep_stmts = statements.iter().filter(|s| {
      s.defined_idents
        .iter()
        .any(|d| stmt.used_defined_idents.contains(d))
    });

    for dep_stmt in dep_stmts {
      queue.push_back(dep_stmt.id);
    }
  }

  result
}

struct UsedDefinedIdentsAnalyzer {
  user_defined_idents: HashSet<SwcId>,
}

impl Visit for UsedDefinedIdentsAnalyzer {
  fn visit_ident(&mut self, node: &farmfe_core::swc_ecma_ast::Ident) {
    self.user_defined_idents.insert(node.to_id().into());
  }
}

pub fn add_format_to_generated_resources(resources: &mut Vec<GeneratedResource>, format: &str) {
  resources.iter_mut().for_each(|resource| {
    resource
      .resource
      .special_placeholders
      .insert("format".to_string(), format.to_string());
    resource.source_map.as_mut().map(|r| {
      r.special_placeholders
        .insert("format".to_string(), format.to_string())
    });
  });
}
