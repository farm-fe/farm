use farmfe_core::cache::module_cache::MetadataOption;
use napi::{
  bindgen_prelude::FromNapiValue,
  sys::{napi_callback_info, napi_env, napi_value},
  Env, JsValue, Unknown,
};

use crate::plugin_adapters::js_plugin_adapter::context::{
  get_argv_and_context_from_cb_info, ArgvAndContext,
};

pub const CONTEXT_WRITE_METADATA: &str = "writeMetadata";
pub const CONTEXT_READ_METADATA: &str = "readMetadata";
pub const CONTEXT_READ_METADATA_BY_SCOPE: &str = "readMetadataByScope";

pub unsafe extern "C" fn context_write_metadata(
  env: napi_env,
  info: napi_callback_info,
) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);
  let name: String = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0:name should be a string when calling writeCache");
  let data: String = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[1]).unwrap())
    .expect("Arguments 1:data should be any when calling writeCache");
  let options: Option<MetadataOption> = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[2]).unwrap())
    .expect("Arguments 2: options should be object when calling writeCache");

  ctx.write_metadata(&name, data, options);

  Env::from_raw(env).to_js_value(&()).unwrap().raw()
}
pub unsafe extern "C" fn context_read_metadata(
  env: napi_env,
  info: napi_callback_info,
) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);
  let name: String = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0:name should be a string when calling writeCache");
  let options: Option<MetadataOption> = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[1]).unwrap())
    .expect("Arguments 2: options should be {} when calling writeCache");

  let data = ctx.read_metadata::<String>(&name, options);

  Env::from_raw(env).to_js_value(&data).unwrap().raw()
}
pub unsafe extern "C" fn context_read_metadata_by_scope(
  env: napi_env,
  info: napi_callback_info,
) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);
  let name: String = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0:name should be a string when calling writeCache");

  let cache = ctx.read_metadata_by_scope::<String>(&name);
  Env::from_raw(env).to_js_value(&cache).unwrap().raw()
}
