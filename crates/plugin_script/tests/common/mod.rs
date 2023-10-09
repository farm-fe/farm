use std::{collections::HashMap, path::PathBuf, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::{Module, ModuleId},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginAnalyzeDepsHookResultEntry,
    PluginFinalizeModuleHookParam, PluginHookContext, PluginLoadHookParam, PluginParseHookParam,
    PluginProcessModuleHookParam,
  },
};

pub fn build_module_deps(
  path: PathBuf,
  base: PathBuf,
) -> (Module, Vec<PluginAnalyzeDepsHookResultEntry>) {
  let config = Default::default();
  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
  let script_plugin = farmfe_plugin_script::FarmPluginScript::new(&context.config);

  let hook_context = PluginHookContext {
    caller: None,
    meta: HashMap::new(),
  };

  let load_result = script_plugin
    .load(
      &PluginLoadHookParam {
        resolved_path: &path.to_string_lossy(),
        query: vec![],
        meta: HashMap::new(),
        module_id: path.to_string_lossy().to_string(),
      },
      &context,
      &hook_context,
    )
    .unwrap()
    .unwrap();

  let mut parse_result = script_plugin
    .parse(
      &PluginParseHookParam {
        module_id: ModuleId::new(&path.to_string_lossy(), "", base.to_str().unwrap()),
        resolved_path: path.to_string_lossy().to_string(),
        query: vec![],
        module_type: load_result.module_type.clone(),
        content: load_result.content,
      },
      &context,
      &hook_context,
    )
    .unwrap()
    .unwrap();

  let mut module = Module::new(ModuleId::new(
    &path.to_string_lossy(),
    "",
    base.to_str().unwrap(),
  ));

  module.module_type = load_result.module_type;

  let mut process_module_param = PluginProcessModuleHookParam {
    module_id: &module.id,
    module_type: &module.module_type,
    meta: &mut parse_result,
  };
  script_plugin.process_module(&mut process_module_param, &context);

  module.meta = parse_result;

  let mut analyze_deps_param = PluginAnalyzeDepsHookParam {
    module: &module,
    deps: vec![],
  };

  script_plugin
    .analyze_deps(&mut analyze_deps_param, &context)
    .unwrap();

  let deps = analyze_deps_param.deps;

  script_plugin
    .finalize_module(
      &mut PluginFinalizeModuleHookParam {
        module: &mut module,
        deps: &deps,
      },
      &context,
    )
    .unwrap();

  (module, deps)
}

pub fn build_module(path: PathBuf, base: PathBuf) -> Module {
  build_module_deps(path, base).0
}
