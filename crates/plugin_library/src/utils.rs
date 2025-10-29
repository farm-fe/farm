use std::{collections::VecDeque, sync::Arc};

use farmfe_core::{
  config::minify::MinifyOptions,
  context::CompilationContext,
  module::meta_data::script::{
    statement::{Statement, StatementId, SwcId},
    FARM_RUNTIME_MODULE_HELPER_ID,
  },
  plugin::{GeneratedResource, PluginHookContext},
  resource::resource_pot::ResourcePot,
  swc_common::{Globals, Mark, GLOBALS},
  swc_ecma_ast::{ImportSpecifier, Module, ModuleDecl, ModuleItem, Stmt},
  HashMap, HashSet,
};
use farmfe_toolkit::{
  fs::EXT,
  resolve::load_package_json,
  script::{
    analyze_statement::analyze_statements,
    minify::minify_js_resource_pot,
    swc_try_with::{resolve_module_mark, ResetSyntaxContextVisitMut},
  },
  swc_ecma_transforms::resolver,
  swc_ecma_visit::{Visit, VisitMutWith, VisitWith},
};

pub fn emit_resource_pot(
  resource_pot: &mut ResourcePot,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<Vec<GeneratedResource>> {
  // For library bundle, if minify is enabled, we should minify resource pot here before emit
  let minify_options = context
    .config
    .minify
    .clone()
    .map(|val| MinifyOptions::from(val))
    .unwrap_or_default();

  if context.config.minify.enabled() {
    let meta = resource_pot.meta.as_js_mut();
    let globals = context.meta.get_resource_pot_globals(&resource_pot.id);
    let (unresolved_mark, top_level_mark) =
      resolve_module_mark(&mut meta.ast, false, globals.value());
    meta.unresolved_mark = unresolved_mark.as_u32();
    meta.top_level_mark = top_level_mark.as_u32();

    minify_js_resource_pot(resource_pot, &minify_options, context)?;
  }

  context
    .plugin_driver
    .generate_resources(resource_pot, context, hook_context).map(|v| v.unwrap_or_else(|| {
      panic!("generate_resources does not emit valid resource for {:?}, please check you plugin config", resource_pot.id)
    }).resources)
}

pub fn inject_farm_runtime_helpers(
  runtime_module_helper_ast: &Module,
  used_helper_idents: &HashSet<String>,
  unresolved_mark: Mark,
  top_level_mark: Mark,
  globals: &Globals,
) -> Vec<ModuleItem> {
  let mut cloned_runtime_module_helper_ast = runtime_module_helper_ast.clone();

  GLOBALS.set(globals, || {
    // clear ctxt
    cloned_runtime_module_helper_ast.visit_mut_with(&mut ResetSyntaxContextVisitMut);

    cloned_runtime_module_helper_ast.visit_mut_with(&mut resolver(
      unresolved_mark,
      top_level_mark,
      false,
    ));
  });

  let statements = analyze_statements(&cloned_runtime_module_helper_ast);
  let preserved_statements = find_used_statements(
    statements,
    &cloned_runtime_module_helper_ast.body,
    used_helper_idents,
  );

  cloned_runtime_module_helper_ast
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
            _ => unreachable!("unexpected module decl {:?}", module_decl),
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
    let both_placeholder_map = HashMap::from_iter([("[format]".to_string(), format.to_string())]);

    resource
      .resource
      .special_placeholders
      .extend(both_placeholder_map.clone());

    // try load current package.json to get ext
    // if type: "module" is specified in the package.json, we should use mjs/cjs by default according to the format
    let ext = std::env::current_dir().ok().and_then(|cwd| {
      load_package_json(cwd.join("package.json"), Default::default())
        .ok()
        .and_then(|pkg_json| {
          pkg_json.raw_map().get("type").and_then(|ty| {
            ty.as_str().and_then(|ty| match ty {
              "module" => match format {
                "cjs" => Some("cjs".to_string()),
                "esm" => Some("mjs".to_string()),
                _ => unimplemented!(
                  "format {} is not supported, current supported format: cjs, esm",
                  format
                ),
              },
              _ => None,
            })
          })
        })
    });

    // override internal ext
    if let Some(ext) = ext {
      resource
        .resource
        .special_placeholders
        .insert(EXT.to_string(), ext);
    }

    // source map
    resource.source_map.as_mut().map(|r| {
      r.special_placeholders.extend(both_placeholder_map);
    });
  });
}
