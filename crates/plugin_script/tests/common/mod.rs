use std::{collections::HashMap, path::PathBuf, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::{Module, ModuleId},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginHookContext, PluginLoadHookParam,
    PluginParseHookParam, ResolveKind,
  },
};
use farmfe_toolkit::script::module_system_from_deps;

pub fn build_module(path: PathBuf, base: PathBuf) -> Module {
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
        resolved_path: &path.to_string_lossy().to_string(),
        query: vec![],
        meta: HashMap::new(),
      },
      &context,
      &hook_context,
    )
    .unwrap()
    .unwrap();

  let parse_result = script_plugin
    .parse(
      &PluginParseHookParam {
        module_id: ModuleId::new(
          &path.to_string_lossy().to_string(),
          "",
          base.to_str().unwrap(),
        ),
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
    &path.to_string_lossy().to_string(),
    "",
    base.to_str().unwrap(),
  ));

  module.module_type = load_result.module_type;
  module.meta = parse_result;

  let mut analyze_deps_param = PluginAnalyzeDepsHookParam {
    module: &module,
    deps: vec![],
  };

  script_plugin
    .analyze_deps(&mut analyze_deps_param, &context)
    .unwrap();

  let deps = analyze_deps_param
    .deps
    .into_iter()
    .map(|dep| dep.kind)
    .collect::<Vec<ResolveKind>>();
  let module_system = module_system_from_deps(deps);

  module.meta.as_script_mut().module_system = module_system;

  module
}
