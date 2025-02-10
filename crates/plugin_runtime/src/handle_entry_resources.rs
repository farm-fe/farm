use std::sync::Arc;

use farmfe_core::enhanced_magic_string::collapse_sourcemap::collapse_sourcemap_chain;
use farmfe_core::enhanced_magic_string::magic_string::MagicString;
use farmfe_core::enhanced_magic_string::types::SourceMapOptions;
use farmfe_core::plugin::PluginHandleEntryResourceHookParam;
use farmfe_core::{
  config::{ModuleFormat, FARM_MODULE_SYSTEM},
  context::CompilationContext,
  module::Module,
  resource::Resource,
};

use farmfe_toolkit::html::get_farm_global_this;
use farmfe_toolkit::sourcemap::append_sourcemap_comment;
use farmfe_toolkit::sourcemap::is_sourcemap_comment_line;
use farmfe_toolkit::sourcemap::SourceMap;

const PREVIOUS_ENTRY_RESOURCE_CODE: &str = "PREVIOUS_ENTRY_RESOURCE_CODE";
const PREVIOUS_ENTRY_RESOURCE_SOURCEMAP_CODE: &str = "PREVIOUS_ENTRY_RESOURCE_CODE";

/// When targetEnv is browser and output.manifest is true, no import/export will be generated and a .farm/manifest.json will be generated instead.
/// Otherwise following code will be generated, if want to run the production in browser without native esm/cjs support, you have to configure bundle rules to make sure only one entry bundle is created
///
/// if single bundle file emitted
/// ```js
/// global['xxx'] = { FARM_TARGET_ENV: 'node' };
/// (function(){
///   // runtime code ...
/// })();
/// (function(m, o) {
///    m.r(xxx)
///  })(m, {
///    "module_id": function(module, exports, require) {
///      const fs = require('fs');
///    }
///  })
///
/// var _m = global['xxx'].m;
/// _m.si(); // setInitialLoadedResources
/// _m.sd(); // setDynamicResourcesMap
/// _m.b();
/// var __farm_entry__ =  _m.r('module_id');
/// var __farm_entry_default = __farm_entry__.default;
/// export { __farm_entry_default as default };
/// ```
/// if multiple bundle file emitted
/// ```js
/// import './farm_runtime.js';
/// import './dep_1.js';
/// import * as __external_fs from 'fs';
///
/// global['xxx'].m.e({ 'fs': __external_fs });
/// // The rest code is the same as above
/// // ...
/// var __farm_entry__ = m.r('module_id');
/// var __farm_entry_default = __farm_entry__.default;
/// export { __farm_entry_default as default };
/// ```
pub fn handle_entry_resources(
  params: &mut PluginHandleEntryResourceHookParam,
  context: &Arc<CompilationContext>,
) {
  let module_graph = params.module_graph;

  let entry_module = module_graph
    .module(params.entry_module_id)
    .expect("module is not found in module graph");

  if !entry_module.module_type.is_script() {
    return;
  }

  let dep_resources = &params
    .initial_resources
    .iter()
    .filter(|res| res.0.as_str() != params.resource.name.as_str())
    .map(|res| &res.0)
    .cloned()
    .collect::<Vec<_>>();

  // 1. runtime code
  let runtime_code = if !dep_resources.is_empty() {
    // runtime resources should emit if there are other initial resources
    params.emit_runtime = true;

    match context.config.output.format {
      ModuleFormat::EsModule => format!("import \"./{}\";", params.runtime_resource_name),
      ModuleFormat::CommonJs => format!("require(\"./{}\");", params.runtime_resource_name),
    }
  } else {
    format!("(function(){{{}}}());", params.runtime_code.to_string())
  };

  // 2. import 'dep' or require('dep'), return empty string if dep_resources is empty
  let load_dep_resources_code = create_load_dep_resources_code(dep_resources, context);

  // 3. moduleSystem.r('module_id')
  let call_entry_module_code = create_call_entry_module_code(
    entry_module,
    dep_resources,
    &params.dynamic_resources,
    &params.dynamic_module_resources_map,
    context,
  );

  // 4. entry resource code
  let entry_resource_code = create_entry_resource_code(&mut params.resource);

  // 5. export code
  let export_info_code = create_export_info_code(entry_module, &context.config.output.format);

  let mut entry_bundle = MagicString::new(&entry_resource_code, None);

  for pre in [load_dep_resources_code, runtime_code] {
    entry_bundle.prepend(&pre);
  }

  for post in [call_entry_module_code, export_info_code] {
    entry_bundle.append(&post);
  }

  // update sourcemap
  let entry_bundle_code = entry_bundle.to_string();
  // update entry resource
  params.resource.bytes = entry_bundle_code.into_bytes();
  // update sourcemap
  if let Some(source_map) = &mut params.resource_sourcemap {
    update_entry_sourcemap(entry_bundle, source_map, &mut params.resource, context);
  }
}

fn create_entry_resource_code(resource: &mut Resource) -> String {
  let mut entry_resource_code = if let Some(code) = resource.meta.get(PREVIOUS_ENTRY_RESOURCE_CODE)
  {
    code.to_string()
  } else {
    let code = String::from_utf8(std::mem::take(&mut resource.bytes)).unwrap();
    resource
      .meta
      .insert(PREVIOUS_ENTRY_RESOURCE_CODE.to_string(), code.clone());
    code
  };

  let mut lines = entry_resource_code.lines().collect::<Vec<_>>();
  // remove last line if it contains source map comment
  if is_sourcemap_comment_line(lines[lines.len() - 1]) {
    lines.pop();
    entry_resource_code = lines.join("\n");
  }

  entry_resource_code
}

fn create_load_dep_resources_code(
  dep_resources: &Vec<String>,
  context: &Arc<CompilationContext>,
) -> String {
  // for backend integration, import/require is not needed, it's handled by backend
  // TODO
  // if context.config.output.manifest {
  //   return "".to_string();
  // }

  dep_resources
    .iter()
    .map(|rn| match context.config.output.format {
      ModuleFormat::EsModule => format!("import \"./{rn}\";"),
      ModuleFormat::CommonJs => format!("require(\"./{rn}\");"),
    })
    .collect::<Vec<_>>()
    .join("")
}

/// create
/// ```js
/// var __farm_entry__ =  _m.r('module_id');
/// var __farm_entry_default = __farm_entry__.default;
/// export { __farm_entry_default as default };
/// ```
fn create_export_info_code(entry_module: &Module, format: &ModuleFormat) -> String {
  let export_idents = entry_module.meta.as_script().get_export_idents();
  let mut decls = vec![];
  let mut exports = vec![];

  for (exported, _) in export_idents {
    decls.push(format!(
      "var __farm_entry_{exported}__=__farm_entry__.{exported};"
    ));
    exports.push((format!("__farm_entry_{exported}__"), exported));
  }

  if !exports.is_empty() {
    match format {
      ModuleFormat::EsModule => {
        let exported_fields = exports
          .into_iter()
          .map(|(value, exported)| format!("{value} as {exported}"))
          .collect::<Vec<_>>();

        format!(
          "{}export {{{}}};",
          decls.join(""),
          exported_fields.join(",")
        )
      }
      ModuleFormat::CommonJs => {
        let exported_fields = exports
          .into_iter()
          .map(|(value, exported)| format!("{exported}:{value}"))
          .collect::<Vec<_>>();

        format!(
          "{}module.exports = {{{}}};",
          decls.join(""),
          exported_fields.join(",")
        )
      }
    }
  } else {
    "".to_string()
  }
}

/// create
/// ```js
/// var _m = global['xxx'].m;
/// _m.si(); // setInitialLoadedResources
/// _m.sd(); // setDynamicResourcesMap
/// _m.b();
/// ```
fn create_call_entry_module_code(
  entry_module: &Module,
  dep_resources: &Vec<String>,
  dynamic_resources: &str,
  dynamic_module_resources_map: &str,
  context: &Arc<CompilationContext>,
) -> String {
  let farm_global_this = get_farm_global_this(
    &context.config.runtime.namespace,
    &context.config.output.target_env,
  );
  // do not set initial loaded resources if there is no dynamic resources(which is a empty array)
  let is_dynamic_empty = dynamic_resources == "[]";

  // setInitialLoadedResources and setDynamicModuleResourcesMap
  let module_system = format!("var __farm_ms__ = {farm_global_this}.{FARM_MODULE_SYSTEM};");
  let set_initial_loaded_resources_code = if !is_dynamic_empty {
    format!(
      r#"__farm_ms__.si([{initial_loaded_resources}]);"#,
      initial_loaded_resources = dep_resources
        .iter()
        .map(|rn| format!("'{rn}'"))
        .collect::<Vec<_>>()
        .join(",")
    )
  } else {
    "".to_string()
  };

  let set_dynamic_resources_map_code = if !is_dynamic_empty {
    format!(r#"__farm_ms__.sd({dynamic_resources},{dynamic_module_resources_map});"#,)
  } else {
    "".to_string()
  };

  let top_level_await_entry =
    if context.config.script.native_top_level_await && entry_module.meta.as_script().is_async {
      "await "
    } else {
      ""
    };

  format!(
    r#"{module_system}{set_initial_loaded_resources_code}{set_dynamic_resources_map_code}__farm_ms__.b();var __farm_entry__={}__farm_ms__.r("{}");"#,
    top_level_await_entry,
    entry_module.id.id(context.config.mode.clone()),
  )
}

fn update_entry_sourcemap(
  entry_bundle: MagicString,
  source_map: &mut Resource,
  resource: &mut Resource,
  context: &Arc<CompilationContext>,
) {
  let entry_bundle_resource_map = entry_bundle
    .generate_map(SourceMapOptions {
      include_content: Some(true),
      ..Default::default()
    })
    .unwrap();
  // read original sourcemap
  let original_source_map =
    if let Some(sourcemap_code) = source_map.meta.get(PREVIOUS_ENTRY_RESOURCE_SOURCEMAP_CODE) {
      SourceMap::from_slice(sourcemap_code.as_bytes()).unwrap()
    } else {
      let code = String::from_utf8(source_map.bytes.clone()).unwrap();
      source_map
        .meta
        .insert(PREVIOUS_ENTRY_RESOURCE_SOURCEMAP_CODE.to_string(), code);
      SourceMap::from_slice(&source_map.bytes).unwrap()
    };

  let collapsed_source_map = collapse_sourcemap_chain(
    vec![original_source_map, entry_bundle_resource_map],
    Default::default(),
  );

  let mut src_map = vec![];

  collapsed_source_map
    .to_writer(&mut src_map)
    .expect("failed to write sourcemap");

  source_map.bytes = src_map;
  append_sourcemap_comment(resource, &source_map, &context.config.sourcemap);
}
