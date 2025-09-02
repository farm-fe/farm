use std::path::Path;

use farmfe_core::module::ModuleId;
use napi::{
  bindgen_prelude::FromNapiValue,
  sys::{napi_callback_info, napi_env, napi_value},
  Env, JsValue, Unknown,
};

use crate::plugin_adapters::js_plugin_adapter::context::{
  get_argv_and_context_from_cb_info, ArgvAndContext,
};

use super::vite_get_modules_by_file::create_vite_module;

pub const VITE_GET_MODULE_BY_ID: &str = "viteGetModuleById";

pub unsafe extern "C" fn vite_get_module_by_id(
  env: napi_env,
  info: napi_callback_info,
) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let file: String = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a string when calling get_modules_by_file");
  let file_path = Path::new(&file);
  let module_graph = ctx.module_graph.read();
  let file_id = if file_path.is_absolute() {
    ModuleId::from_resolved_path_with_query(&file, &ctx.config.root)
  } else {
    file.into()
  };

  let module = module_graph.module(&file_id).map(|m| {
    let id = m.id.resolved_path_with_query(&ctx.config.root);
    create_vite_module(id, m, &ctx.config.root)
  });

  Env::from_raw(env).to_js_value(&module).unwrap().raw()
}
