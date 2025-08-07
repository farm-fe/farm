use farmfe_core::{
  module::{ModuleId, ModuleType},
  relative_path::RelativePath,
  HashMap,
};
use napi::{
  bindgen_prelude::FromNapiValue,
  sys::{napi_callback_info, napi_env, napi_value},
  Env, JsValue, Unknown,
};

use crate::plugin_adapters::js_plugin_adapter::context::{
  get_argv_and_context_from_cb_info, ArgvAndContext,
};

pub const VITE_GET_IMPORTERS: &str = "viteGetImporters";

pub unsafe extern "C" fn vite_get_importers(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let id: String = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a string when calling viteGetImporters");

  let module_graph = ctx.module_graph.read();
  let module_id: ModuleId = id.into();
  let dependents = module_graph.dependents_ids(&module_id);

  let importers = dependents
    .into_iter()
    .map(|id| {
      let m = module_graph.module(&id).unwrap();
      let id = RelativePath::new(&m.id.to_string())
        .to_logical_path(&ctx.config.root)
        .to_string_lossy()
        .to_string();
      HashMap::from_iter([
        ("url".to_string(), id.clone()),
        ("id".to_string(), id),
        ("file".to_string(), m.id.resolved_path(&ctx.config.root)),
        (
          "type".to_string(),
          if m.module_type == ModuleType::Css {
            "css".to_string()
          } else {
            "js".to_string()
          },
        ),
      ])
    })
    .collect::<Vec<_>>();

  Env::from_raw(env).to_js_value(&importers).unwrap().raw()
}
