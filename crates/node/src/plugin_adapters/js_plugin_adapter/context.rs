use std::{
  ffi::{c_void, CString},
  ptr,
  sync::Arc,
};

use napi::{
  bindgen_prelude::FromNapiValue,
  sys::{
    napi_callback_info, napi_create_function, napi_create_object, napi_env, napi_get_cb_info,
    napi_value,
  },
  Env, Error, JsFunction, JsObject, JsUnknown, NapiRaw, Status,
};

use farmfe_core::{context::CompilationContext, plugin::PluginResolveHookParam};

const RESOLVE: &str = "resolve";

/// create a js object context that wraps [Arc<CompilationContext>]
/// # Safety
/// calling [napi_create_object]
pub unsafe fn create_js_context(raw_env: napi_env, ctx: Arc<CompilationContext>) -> JsObject {
  let mut js_context_ptr = ptr::null_mut();
  let mut js_context = {
    napi_create_object(raw_env, &mut js_context_ptr);
    JsObject::from_napi_value(raw_env, js_context_ptr).unwrap()
  };

  js_context = attach_resolve_property(raw_env, js_context, ctx);

  js_context
}

/// Create a js resolve function based on [farmfe_core::context::CompilationContext]
/// and attach it to the js context object
fn attach_resolve_property(
  env: napi_env,
  mut context: JsObject,
  ctx: Arc<CompilationContext>,
) -> JsObject {
  let len = RESOLVE.len();
  let s = CString::new(RESOLVE).unwrap();

  let mut func = ptr::null_mut();
  unsafe {
    napi_create_function(
      env,
      s.as_ptr(),
      len,
      Some(resolve),
      Box::into_raw(Box::new(ctx)) as *mut c_void,
      &mut func,
    );

    context
      .set_named_property(RESOLVE, JsFunction::from_napi_value(env, func).unwrap())
      .unwrap();
  }

  context
}

unsafe extern "C" fn resolve(env: napi_env, info: napi_callback_info) -> napi_value {
  let mut argv: [napi_value; 1] = [ptr::null_mut()];
  let mut data = ptr::null_mut();
  napi_get_cb_info(
    env,
    info,
    &mut 1,
    argv.as_mut_ptr(),
    ptr::null_mut(),
    &mut data,
  );

  let ctx: Box<Arc<CompilationContext>> = Box::from_raw(data.cast());
  let param: PluginResolveHookParam = Env::from_raw(env)
    .from_js_value(JsUnknown::from_napi_value(env, argv[0]).unwrap())
    .unwrap();

  Env::from_raw(env)
    .execute_tokio_future(
      async move {
        let resolved = ctx
          .plugin_driver
          .resolve(&param, &ctx)
          .map_err(|e| Error::new(Status::GenericFailure, format!("{}", e)))?;

        resolved.ok_or_else(|| {
          Error::new(
            Status::GenericFailure,
            format!("can not resolve {:?}", param),
          )
        })
      },
      |&mut env, data| env.to_js_value(&data),
    )
    .unwrap()
    .raw()
}
