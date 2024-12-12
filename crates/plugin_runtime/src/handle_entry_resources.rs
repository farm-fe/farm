use std::sync::Arc;

use farmfe_core::enhanced_magic_string::collapse_sourcemap::collapse_sourcemap_chain;
use farmfe_core::enhanced_magic_string::magic_string::MagicString;
use farmfe_core::enhanced_magic_string::types::SourceMapOptions;
use farmfe_core::plugin::ResolveKind;
use farmfe_core::resource::ResourceOrigin;
use farmfe_core::HashMap;
use farmfe_core::HashSet;
use farmfe_core::{
  config::{ModuleFormat, TargetEnv, FARM_MODULE_SYSTEM},
  context::CompilationContext,
  module::{
    module_graph::ModuleGraph, module_group::ModuleGroupGraph, Module, ModuleId, ModuleSystem,
  },
  resource::{Resource, ResourceType},
  swc_ecma_ast::{self, Decl, ModuleDecl, ModuleItem, Pat},
};
use farmfe_toolkit::common::{append_source_map_comment, generate_source_map_resource};
use farmfe_toolkit::fs::transform_output_entry_filename;
use farmfe_toolkit::get_dynamic_resources_map::{
  get_dynamic_resources_code, get_dynamic_resources_map,
};
use farmfe_toolkit::html::get_farm_global_this;
use farmfe_toolkit::sourcemap::SourceMap;
use farmfe_utils::transform_string_to_static_str;

const FARM_NODE_MODULE: &str = "__farmNodeModule";

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
                  name: named_spec
                    .exported
                    .as_ref()
                    .map(|exported| match exported {
                      swc_ecma_ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
                      swc_ecma_ast::ModuleExportName::Str(str) => str.value.to_string(),
                    })
                    .unwrap_or(match &named_spec.orig {
                      swc_ecma_ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
                      swc_ecma_ast::ModuleExportName::Str(str) => str.value.to_string(),
                    }),
                  import_as: None,
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
          let dep_module =
            module_graph.get_dep_by_source(entry_module_id, &source, Some(ResolveKind::ExportFrom));
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
  let mut visited = HashSet::default();
  let export_info = get_export_info_of_entry_module(entry_module_id, module_graph, &mut visited);

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
              ModuleFormat::CommonJs => format!("module.exports.{import_as} = entry.{name};"),
              ModuleFormat::EsModule => {
                format!("var {name}=entry.{name};export {{ {name} as {import_as} }};")
              }
            }
          } else {
            match context.config.output.format {
              ModuleFormat::CommonJs => format!("module.exports.{name} = entry.{name};"),
              ModuleFormat::EsModule => format!("var {name}=entry.{name};export {{ {name} }};"),
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

#[derive(Debug, Default)]
struct EntryResourceAndDepResources {
  pub entry_js_resource_name: String,
  pub entry_js_resource_source_map: Option<Resource>,
  pub entry_js_resource_source_map_name: String,
  pub entry_js_resource_code: Arc<String>,
  pub dep_resources: Vec<String>,
  pub dynamic_resources: String,
  pub dynamic_module_resources_map: String,
}

fn get_entry_resource_and_dep_resources_name(
  entry: &ModuleId,
  module: &Module,
  module_group_graph: &ModuleGroupGraph,
  resource_map: &HashMap<String, Resource>,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> EntryResourceAndDepResources {
  let mut result = EntryResourceAndDepResources::default();

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
        panic!("entry module is not equal to module id, maybe entry imports another entry");
      }

      for resource_id in resource_pot.resources() {
        let resource = resource_map
          .get(resource_id)
          .expect("resource is not found");

        if matches!(resource.resource_type, ResourceType::Js) {
          result.entry_js_resource_name = resource.name.clone();
        } else if matches!(resource.resource_type, ResourceType::SourceMap(_)) {
          result.entry_js_resource_source_map_name = resource.name.clone();
        }
      }
      result.entry_js_resource_code = resource_pot.meta.rendered_content.clone();

      if !resource_pot.meta.rendered_map_chain.is_empty() {
        result.entry_js_resource_source_map = Some(generate_source_map_resource(resource_pot));
      }
    } else {
      for resource_id in resource_pot.resources() {
        let resource = resource_map
          .get(resource_id)
          .expect("resource is not found");

        if matches!(resource.resource_type, ResourceType::Js) {
          result.dep_resources.push(resource.name.clone());
        }
      }
    }
  }

  let dynamic_resources_map = get_dynamic_resources_map(
    module_group_graph,
    entry,
    &resource_pot_map,
    resource_map,
    module_graph,
  );
  let (dynamic_resources, dynamic_module_resources_map) =
    get_dynamic_resources_code(&dynamic_resources_map, context.config.mode.clone());

  result.dynamic_resources = dynamic_resources;
  result.dynamic_module_resources_map = dynamic_module_resources_map;
  result
}

pub fn handle_entry_resources(
  resources_map: &mut HashMap<String, Resource>,
  context: &Arc<CompilationContext>,
  async_modules: &HashSet<ModuleId>,
) {
  let module_graph = context.module_graph.read();
  let module_group_graph = context.module_group_graph.read();
  let is_library = context.config.output.target_env.is_library();

  // create a runtime resource
  let mut runtime_code = None;
  let mut runtime_resource = None;
  let mut should_inject_runtime = false;

  for (entry, _) in &module_graph.entries {
    let module = module_graph
      .module(entry)
      .expect("module is not found in module graph");

    // find entry resource and other resources that is required by entry resource
    if module.module_type.is_script() {
      let EntryResourceAndDepResources {
        entry_js_resource_name,
        entry_js_resource_code,
        entry_js_resource_source_map_name,
        entry_js_resource_source_map,
        mut dep_resources,
        dynamic_resources,
        dynamic_module_resources_map,
      } = get_entry_resource_and_dep_resources_name(
        entry,
        module,
        &module_group_graph,
        resources_map,
        &module_graph,
        context,
      );
      dep_resources.sort();

      if !should_inject_runtime {
        should_inject_runtime = !dep_resources.is_empty();
      }

      // 1. import 'dep' or require('dep') to entry resource if target env is node
      let dep_resources_require_code = dep_resources
        .iter()
        .map(|rn| match context.config.output.format {
          ModuleFormat::EsModule => format!("import \"./{rn}\";"),
          ModuleFormat::CommonJs => format!("require(\"./{rn}\");"),
        })
        .collect::<Vec<_>>()
        .join("");

      let farm_global_this = get_farm_global_this(
        &context.config.runtime.namespace,
        &context.config.output.target_env,
      );

      // 4. setInitialLoadedResources and setDynamicModuleResourcesMap
      let set_initial_loaded_resources_code = format!(
        r#"{farm_global_this}.{FARM_MODULE_SYSTEM}.setInitialLoadedResources([{initial_loaded_resources}]);"#,
        initial_loaded_resources = dep_resources
          .iter()
          .map(|rn| format!("'{rn}'"))
          .collect::<Vec<_>>()
          .join(",")
      );
      let set_dynamic_resources_map_code = if !dynamic_resources.is_empty() {
        format!(
          r#"{farm_global_this}.{FARM_MODULE_SYSTEM}.setDynamicModuleResourcesMap({dynamic_resources},{dynamic_module_resources_map});"#,
        )
      } else {
        "".to_string()
      };

      let top_level_await_entry =
        if context.config.script.native_top_level_await && async_modules.contains(entry) {
          "await "
        } else {
          ""
        };

      // 5. append call entry
      let call_entry_code = format!(
        r#"var farmModuleSystem = {}.{};farmModuleSystem.bootstrap();var entry = {}farmModuleSystem.require("{}");"#,
        farm_global_this,
        FARM_MODULE_SYSTEM,
        top_level_await_entry,
        entry.id(context.config.mode.clone()),
      );

      // 6. append export code
      let export_info_code = get_export_info_code(entry, &module_graph, context);

      let runtime_code = if let Some(runtime_code) = runtime_code.as_ref() {
        runtime_code
      } else {
        runtime_code = Some(create_runtime_code(resources_map, context));
        runtime_code.as_ref().unwrap()
      };

      let entry_js_resource = resources_map
        .get_mut(&entry_js_resource_name)
        .expect("entry resource is not found");

      let mut entry_bundle = MagicString::new(&entry_js_resource_code, None);

      for pre in vec![
        dep_resources_require_code,
        if should_inject_runtime && !is_library {
          let runtime_resource = if let Some(runtime_resource) = runtime_resource.as_ref() {
            runtime_resource
          } else {
            runtime_resource = Some(create_farm_runtime_resource(runtime_code, context));
            runtime_resource.as_ref().unwrap()
          };

          match context.config.output.format {
            ModuleFormat::EsModule => format!("import \"./{}\";", runtime_resource.name),
            ModuleFormat::CommonJs => format!("require(\"./{}\");", runtime_resource.name),
          }
        } else {
          runtime_code.clone()
        },
      ] {
        entry_bundle.prepend(&pre);
      }

      if !is_library {
        for post in vec![
          set_initial_loaded_resources_code,
          set_dynamic_resources_map_code,
          call_entry_code,
          export_info_code,
        ] {
          entry_bundle.append(&post);
        }
      }

      let entry_bundle_code = entry_bundle.to_string();
      // update entry resource
      entry_js_resource.bytes = entry_bundle_code.into_bytes();
      // update sourcemap
      if let Some(mut source_map) = entry_js_resource_source_map {
        let entry_bundle_resource_map = entry_bundle
          .generate_map(SourceMapOptions {
            include_content: Some(true),
            ..Default::default()
          })
          .unwrap();
        let original_source_map = SourceMap::from_slice(&source_map.bytes).unwrap();
        let collapsed_source_map = collapse_sourcemap_chain(
          vec![original_source_map, entry_bundle_resource_map],
          Default::default(),
        );
        let mut src_map = vec![];
        collapsed_source_map
          .to_writer(&mut src_map)
          .expect("failed to write sourcemap");

        source_map.bytes = src_map;
        source_map.name = entry_js_resource_source_map_name.clone();
        append_source_map_comment(entry_js_resource, &source_map, &context.config.sourcemap);
        // update sourcemap resource
        resources_map.insert(entry_js_resource_source_map_name, source_map);
      }
    }
  }

  if should_inject_runtime {
    if let Some(runtime_resource) = runtime_resource {
      resources_map.insert(runtime_resource.name.clone(), runtime_resource);
    }
  }
}

fn create_runtime_code(
  resources_map: &HashMap<String, Resource>,
  context: &Arc<CompilationContext>,
) -> String {
  let node_specific_code = if context.config.output.target_env.is_node() {
    match context.config.output.format {
      ModuleFormat::EsModule => {
        format!(
          r#"import {FARM_NODE_MODULE} from 'node:module';global.nodeRequire = {FARM_NODE_MODULE}.createRequire(import.meta.url);"#
        )
      }
      ModuleFormat::CommonJs => r#"global.nodeRequire = require;"#.to_string(), // _ => panic!("node only support cjs and esm format"),
    }
  } else {
    "".to_string()
  };

  // 2. __farm_global_this by namespace
  let farm_global_this = get_farm_global_this(
    &context.config.runtime.namespace,
    &context.config.output.target_env,
  );
  let farm_global_this_code = format!(
    r#"{farm_global_this} = {{__FARM_TARGET_ENV__: '{}'}};"#,
    match &context.config.output.target_env {
      TargetEnv::Browser => "browser",
      TargetEnv::Node => "node",
      // should never be here
      TargetEnv::Library => "library",
      TargetEnv::Custom(env) => transform_string_to_static_str(env.clone()),
    }
  );

  // 3. find runtime resource
  let runtime_resource_code = String::from_utf8(
    resources_map
      .values()
      .find(|r| matches!(r.resource_type, ResourceType::Runtime))
      .expect("runtime resource not found")
      .bytes
      .clone(),
  )
  .unwrap();

  format!("{node_specific_code}{farm_global_this_code}{runtime_resource_code}")
}

fn create_farm_runtime_resource(runtime_code: &str, context: &Arc<CompilationContext>) -> Resource {
  let bytes = runtime_code.to_string().into_bytes();
  let name = transform_output_entry_filename(
    context.config.output.entry_filename.clone(),
    "__farm_runtime",
    "__farm_runtime",
    &bytes,
    "js", // todo: support configuring extension
          // match context.config.output.format {
          //   ModuleFormat::EsModule => "mjs",
          //   ModuleFormat::CommonJs => "cjs",
          // },
  );
  Resource {
    name: name.clone(),
    bytes,
    emitted: false,
    // this resource should be Js instead of Runtime because it may cause duplicated runtime code when HMR if it's Runtime
    resource_type: ResourceType::Js,
    origin: ResourceOrigin::ResourcePot(name),
    info: None,
  }
}
