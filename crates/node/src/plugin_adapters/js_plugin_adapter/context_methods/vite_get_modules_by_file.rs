use farmfe_core::module::{Module, ModuleId, ModuleType};
use farmfe_core::{HashMap, HashSet};
use napi::JsValue;
use napi::{
  bindgen_prelude::FromNapiValue,
  sys::{napi_callback_info, napi_env, napi_value},
  Env, Unknown,
};

use crate::plugin_adapters::js_plugin_adapter::context::{
  get_argv_and_context_from_cb_info, ArgvAndContext,
};

pub const VITE_GET_MODULES_BY_FILE: &str = "viteGetModulesByFile";

pub fn create_vite_module(id: String, m: &Module, root: &str) -> HashMap<String, String> {
  HashMap::from_iter([
    ("url".to_string(), id.clone()),
    ("id".to_string(), id),
    ("file".to_string(), m.id.resolved_path(root)),
    (
      "type".to_string(),
      if m.module_type == ModuleType::Css {
        "css".to_string()
      } else {
        "js".to_string()
      },
    ),
  ])
}

pub unsafe extern "C" fn vite_get_modules_by_file(
  env: napi_env,
  info: napi_callback_info,
) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let file: String = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a string when calling get_modules_by_file");

  let module_graph = ctx.module_graph.read();
  let file_id = ModuleId::from_resolved_path_with_query(&file, &ctx.config.root);

  let mut seen = HashSet::default();
  let mut module_ids = vec![];

  if module_graph.has_module(&file_id) {
    seen.insert(file_id.clone());
    module_ids.push(file_id.clone());
  }

  module_ids.extend(
    module_graph
      .module_ids_by_file(&file_id)
      .into_iter()
      .filter(|m_id| seen.insert(m_id.clone())),
  );

  let modules = module_ids
    .into_iter()
    .map(|m_id| {
      let m = module_graph.module(&m_id).unwrap();
      let id = m_id.resolved_path_with_query(&ctx.config.root);
      create_vite_module(id, m, &ctx.config.root)
    })
    .collect::<Vec<_>>();

  Env::from_raw(env).to_js_value(&modules).unwrap().raw()
}
