use std::sync::Arc;

use farmfe_core::{
  config::{ModuleFormat, TargetEnv, FARM_GLOBAL_THIS, FARM_MODULE_SYSTEM, FARM_NAMESPACE},
  context::CompilationContext,
  hashbrown::{HashMap, HashSet},
  module::{
    module_graph::ModuleGraph, module_group::ModuleGroupGraph, Module, ModuleId, ModuleSystem,
  },
  resource::{Resource, ResourceType},
  swc_ecma_ast::{self, Decl, ModuleDecl, ModuleItem, Pat},
};
use farmfe_toolkit::get_dynamic_resources_map::{
  get_dynamic_resources_code, get_dynamic_resources_map,
};

use crate::FARM_NODE_MODULE;

pub enum ExportInfoOfEntryModule {
  Default,
  Named {
    name: String,
    import_as: Option<String>,
  },
  CJS,
}

pub fn get_export_info_of_entry_module(
  entry_module_id: &ModuleId,
  module_graph: &ModuleGraph,
  visited: &mut HashSet<ModuleId>,
) -> Vec<ExportInfoOfEntryModule> {
  if visited.contains(entry_module_id) {
    return vec![];
  }

  visited.insert(entry_module_id.clone());

  let entry_module = module_graph
    .module(entry_module_id)
    .expect("entry module is not found");

  if entry_module.external {
    return vec![];
  }

  if matches!(
    entry_module.meta.as_script().module_system,
    ModuleSystem::CommonJs
  ) {
    return vec![ExportInfoOfEntryModule::CJS];
  }

  let ast = &entry_module.meta.as_script().ast;
  let mut export_info = vec![];

  for item in ast.body.iter() {
    match item {
      ModuleItem::ModuleDecl(module_decl) => match module_decl {
        ModuleDecl::ExportDecl(export_decl) => match &export_decl.decl {
          Decl::Class(class) => {
            export_info.push(ExportInfoOfEntryModule::Named {
              name: class.ident.sym.to_string(),
              import_as: None,
            });
          }
          Decl::Fn(func) => {
            export_info.push(ExportInfoOfEntryModule::Named {
              name: func.ident.sym.to_string(),
              import_as: None,
            });
          }
          Decl::Var(var) => {
            for decl in var.decls.iter() {
              match &decl.name {
                Pat::Ident(ident) => {
                  export_info.push(ExportInfoOfEntryModule::Named {
                    name: ident.sym.to_string(),
                    import_as: None,
                  });
                }
                _ => {}
              }
            }
          }
          Decl::Using(_)
          | Decl::TsInterface(_)
          | Decl::TsTypeAlias(_)
          | Decl::TsEnum(_)
          | Decl::TsModule(_) => {
            panic!("export type is not supported")
          }
        },
        ModuleDecl::ExportNamed(named_export) => {
          for spec in named_export.specifiers.iter() {
            match spec {
              swc_ecma_ast::ExportSpecifier::Named(named_spec) => {
                export_info.push(ExportInfoOfEntryModule::Named {
                  name: match &named_spec.orig {
                    swc_ecma_ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
                    swc_ecma_ast::ModuleExportName::Str(str) => str.value.to_string(),
                  },
                  import_as: named_spec.exported.as_ref().map(|exported| match exported {
                    swc_ecma_ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
                    swc_ecma_ast::ModuleExportName::Str(str) => str.value.to_string(),
                  }),
                });
              }
              swc_ecma_ast::ExportSpecifier::Default(default) => {
                export_info.push(ExportInfoOfEntryModule::Named {
                  name: default.exported.sym.to_string(),
                  import_as: None,
                });
              }
              swc_ecma_ast::ExportSpecifier::Namespace(ns) => {
                export_info.push(ExportInfoOfEntryModule::Named {
                  name: match &ns.name {
                    swc_ecma_ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
                    swc_ecma_ast::ModuleExportName::Str(str) => str.value.to_string(),
                  },
                  import_as: None,
                });
              }
            }
          }
        }
        ModuleDecl::ExportAll(export_all) => {
          let source = export_all.src.value.to_string();
          let dep_module = module_graph.get_dep_by_source(entry_module_id, &source);
          let mut dep_export_info =
            get_export_info_of_entry_module(&dep_module, module_graph, visited)
              .into_iter()
              .filter(|e| !matches!(e, ExportInfoOfEntryModule::Default))
              .collect();

          export_info.append(&mut dep_export_info);
        }
        ModuleDecl::ExportDefaultDecl(_) | ModuleDecl::ExportDefaultExpr(_) => {
          export_info.push(ExportInfoOfEntryModule::Default);
        }
        _ => {}
      },
      _ => {}
    }
  }

  export_info
}

fn get_export_info_code(
  entry_module_id: &ModuleId,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> String {
  let mut visited = HashSet::new();
  let export_info = get_export_info_of_entry_module(entry_module_id, module_graph, &mut visited);

  // TODO cover it with test
  if !export_info.is_empty() {
    export_info
      .iter()
      .map(|export| match export {
        ExportInfoOfEntryModule::Default => match context.config.output.format {
          ModuleFormat::CommonJs => "module.exports = entry.default || entry;".to_string(),
          ModuleFormat::EsModule => "export default entry.default || entry;".to_string(),
        },
        ExportInfoOfEntryModule::Named { name, import_as } => {
          if let Some(import_as) = import_as {
            match context.config.output.format {
              ModuleFormat::CommonJs => format!("exports.{} = entry.{};", import_as, name),
              ModuleFormat::EsModule => format!(
                "var {name}=entry.{name};export {{ {} as {} }};",
                name, import_as
              ),
            }
          } else {
            match context.config.output.format {
              ModuleFormat::CommonJs => format!("exports.{} = entry.{};", name, name),
              ModuleFormat::EsModule => format!("var {name}=entry.{name};export {{ {} }};", name),
            }
          }
        }
        ExportInfoOfEntryModule::CJS => match context.config.output.format {
          ModuleFormat::CommonJs => "module.exports = entry;".to_string(),
          ModuleFormat::EsModule => "export default entry;".to_string(),
        },
      })
      .collect::<Vec<String>>()
      .join("")
  } else {
    "".to_string()
  }
}

pub fn get_entry_resource_and_dep_resources_name(
  entry: &ModuleId,
  module: &Module,
  module_group_graph: &ModuleGroupGraph,
  resource_map: &HashMap<String, Resource>,
  context: &Arc<CompilationContext>,
) -> (String, Vec<String>, String) {
  let mut entry_js_resource_name = None;
  let mut dep_resources = vec![];

  let module_group_id = entry.clone();
  let module_group = module_group_graph
    .module_group(&module_group_id)
    .expect("module group is not found");
  let resource_pot_map = context.resource_pot_map.read();

  for resource_pot_id in module_group.resource_pots() {
    let resource_pot = resource_pot_map
      .resource_pot(resource_pot_id)
      .expect("resource pot is not found");

    if let Some(entry) = &resource_pot.entry_module {
      if entry != &module.id {
        panic!("entry module is not equal to module id");
      }

      for resource_id in resource_pot.resources() {
        let resource = resource_map
          .get(resource_id)
          .expect("resource is not found");

        if matches!(resource.resource_type, ResourceType::Js) {
          entry_js_resource_name = Some(resource.name.clone());
          break;
        }
      }
    } else {
      for resource_id in resource_pot.resources() {
        let resource = resource_map
          .get(resource_id)
          .expect("resource is not found");

        if matches!(resource.resource_type, ResourceType::Js) {
          dep_resources.push(resource.name.clone());
        }
      }
    }
  }

  let dynamic_resources_map =
    get_dynamic_resources_map(module_group_graph, entry, &resource_pot_map, resource_map);
  let dynamic_resources_code =
    get_dynamic_resources_code(&dynamic_resources_map, context.config.mode.clone());

  (
    entry_js_resource_name.unwrap(),
    dep_resources,
    dynamic_resources_code,
  )
}

pub fn handle_entry_resources(
  resources_map: &mut HashMap<String, Resource>,
  context: &Arc<CompilationContext>,
) {
  let module_graph = context.module_graph.read();
  let module_group_graph = context.module_group_graph.read();

  for (entry, _) in &module_graph.entries {
    let module = module_graph
      .module(entry)
      .expect("module is not found in module graph");

    // find entry resource and other resources that is required by entry resource
    if module.module_type.is_script() {
      let (entry_js_resource_name, mut dep_resources, dynamic_resources_code) =
        get_entry_resource_and_dep_resources_name(
          entry,
          module,
          &module_group_graph,
          resources_map,
          context,
        );
      dep_resources.sort();

      // 1. node specific code.
      // TODO: support async module for node, using dynamic require to load external module instead of createRequire. createRequire does not support load ESM module.
      let node_specific_code = if context.config.output.target_env == TargetEnv::Node {
        match context.config.output.format {
          ModuleFormat::EsModule => {
            format!(
              r#"import {FARM_NODE_MODULE} from 'node:module';var __farmNodeRequire = {FARM_NODE_MODULE}.createRequire(import.meta.url);var __farmNodeBuiltinModules = {FARM_NODE_MODULE}.builtinModules;"#
            )
          }
          ModuleFormat::CommonJs => r#"var __farmNodeRequire = require;var __farmNodeBuiltinModules = require('node:module').builtinModules;"#
            .to_string(), // _ => panic!("node only support cjs and esm format"),
        }
      } else {
        "".to_string()
      };

      // 2. __farm_global_this by namespace
      let farm_namespace = &context.config.runtime.namespace;
      let farm_global_this_code = format!(
        r#"(globalThis || window || global || self).{FARM_NAMESPACE} = '{farm_namespace}';{FARM_GLOBAL_THIS} = {{__FARM_TARGET_ENV__: '{}'}};"#,
        match &context.config.output.target_env {
          TargetEnv::Browser => "browser",
          TargetEnv::Node => "node",
        }
      );

      // 3. find runtime resource
      let runtime_resource_code = String::from_utf8(
        resources_map
          .values()
          .find(|r| matches!(r.resource_type, ResourceType::Runtime))
          .expect("runtime resource is not found")
          .bytes
          .clone(),
      )
      .unwrap();

      // 4. __farmNodeRequire(dep) to entry resource if target env is node
      let dep_resources_require_code = if context.config.output.target_env == TargetEnv::Node {
        dep_resources
          .iter()
          .map(|rn| format!("__farmNodeRequire('./{}');", rn))
          .collect::<Vec<_>>()
          .join("\n")
      } else {
        "".to_string()
      };
      // 5. setInitialLoadedResources and setDynamicModuleResourcesMap
      let set_initial_loaded_resources_code = format!(
        r#"{FARM_GLOBAL_THIS}.{FARM_MODULE_SYSTEM}.setInitialLoadedResources([{initial_loaded_resources}]);"#,
        initial_loaded_resources = dep_resources
          .iter()
          .map(|rn| format!("'{}'", rn))
          .collect::<Vec<_>>()
          .join(",")
      );
      let set_dynamic_resources_map_code = format!(
        r#"{FARM_GLOBAL_THIS}.{FARM_MODULE_SYSTEM}.setDynamicModuleResourcesMap({dynamic_resources_code});"#,
      );

      // 6. append call entry
      let call_entry_code = format!(
        r#"var farmModuleSystem = {}.{};farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("{}");"#,
        FARM_GLOBAL_THIS,
        FARM_MODULE_SYSTEM,
        entry.id(context.config.mode.clone()),
      );

      // 7. append export code
      let export_info_code = get_export_info_code(entry, &module_graph, context);

      let entry_js_resource_code = String::from_utf8(
        resources_map
          .get(&entry_js_resource_name)
          .expect("entry resource is not found")
          .bytes
          .clone(),
      )
      .unwrap();
      // split last line
      let (entry_js_resource_code, entry_js_resource_source_map) =
        if let Some((c, m)) = entry_js_resource_code.rsplit_once('\n') {
          if m.starts_with("//# sourceMappingURL=") {
            (c.to_string(), format!("\n{}", m))
          } else {
            (entry_js_resource_code, "".to_string())
          }
        } else {
          (entry_js_resource_code, "".to_string())
        };

      let entry_js_resource = resources_map
        .get_mut(&entry_js_resource_name)
        .expect("entry resource is not found");

      entry_js_resource.bytes = format!(
        "{}{}{}{}{}{}{}{}{}{}",
        node_specific_code,
        farm_global_this_code,
        runtime_resource_code,
        dep_resources_require_code,
        set_initial_loaded_resources_code,
        set_dynamic_resources_map_code,
        entry_js_resource_code,
        call_entry_code,
        export_info_code,
        entry_js_resource_source_map,
      )
      .into_bytes();
    }
  }
}
