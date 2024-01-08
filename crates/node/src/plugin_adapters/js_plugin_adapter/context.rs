use std::{
  ffi::{c_void, CString},
  ptr,
  sync::Arc,
};

use napi::{
  bindgen_prelude::FromNapiValue,
  sys::{
    napi_callback, napi_callback_info, napi_create_function, napi_create_object, napi_env,
    napi_get_cb_info, napi_value,
  },
  Env, Error, JsFunction, JsObject, JsUnknown, NapiRaw, Status,
};

use farmfe_core::{
  context::{CompilationContext, EmitFileParams},
  module::ModuleId,
  // swc_ecma_ast::EsVersion,
  plugin::{PluginHookContext, PluginResolveHookParam},
};

const RESOLVE: &str = "resolve";
const ADD_WATCH_FILE: &str = "addWatchFile";
const EMIT_FILE: &str = "emitFile";
const GET_WATCH_FILES: &str = "getWatchFiles";
const WARN: &str = "warn";
const ERROR: &str = "error";
const SOURCE_MAP_ENABLED: &str = "sourceMapEnabled";

/// These functions are used to make farm js plugin compatible with Vite plugin
use super::context_methods::vite_get_importers::{vite_get_importers, VITE_GET_IMPORTERS};
use super::context_methods::vite_get_module_by_id::{vite_get_module_by_id, VITE_GET_MODULE_BY_ID};
use super::context_methods::vite_get_modules_by_file::{
  vite_get_modules_by_file, VITE_GET_MODULES_BY_FILE,
};

pub struct GetModulesByFileResultItem {
  pub url: String,
  pub id: String,
  pub file: String,
  pub ty: String,
}

/// create a js object context that wraps [Arc<CompilationContext>]
/// # Safety
/// calling [napi_create_object]
pub unsafe fn create_js_context(raw_env: napi_env, ctx: Arc<CompilationContext>) -> JsObject {
  let mut js_context_ptr = ptr::null_mut();
  let mut js_context = {
    napi_create_object(raw_env, &mut js_context_ptr);
    JsObject::from_napi_value(raw_env, js_context_ptr).unwrap()
  };

  let methods = vec![
    (
      RESOLVE,
      resolve as unsafe extern "C" fn(napi_env, napi_callback_info) -> napi_value,
    ),
    // (PARSE, parse),
    (ADD_WATCH_FILE, add_watch_file),
    (EMIT_FILE, emit_file),
    (GET_WATCH_FILES, get_watch_files),
    (WARN, warn),
    (ERROR, error),
    (SOURCE_MAP_ENABLED, source_map_enabled),
    (VITE_GET_IMPORTERS, vite_get_importers),
    (VITE_GET_MODULES_BY_FILE, vite_get_modules_by_file),
    (VITE_GET_MODULE_BY_ID, vite_get_module_by_id),
  ];

  for (name, cb) in methods {
    js_context = attach_context_method(raw_env, js_context, name, Some(cb), ctx.clone());
  }

  js_context
}

/// Create a js resolve function based on [farmfe_core::context::CompilationContext]
/// and attach it to the js context object
fn attach_context_method(
  env: napi_env,
  mut context: JsObject,
  name: &str,
  cb: napi_callback,
  ctx: Arc<CompilationContext>,
) -> JsObject {
  let len = name.len();
  let s = CString::new(name).unwrap();

  let mut func = ptr::null_mut();
  unsafe {
    napi_create_function(
      env,
      s.as_ptr(),
      len,
      cb,
      Box::into_raw(Box::new(ctx)) as *mut c_void,
      &mut func,
    );

    context
      .set_named_property(name, JsFunction::from_napi_value(env, func).unwrap())
      .unwrap();
  }

  context
}

#[repr(C)]
pub struct ArgvAndContext {
  pub argv: [napi_value; 2],
  pub ctx: Box<Arc<CompilationContext>>,
}

/// # Safety
pub unsafe extern "C" fn get_argv_and_context_from_cb_info(
  env: napi_env,
  info: napi_callback_info,
) -> ArgvAndContext {
  // accept 2 arguments at most
  let mut argv: [napi_value; 2] = [ptr::null_mut(); 2];
  let mut data = ptr::null_mut();
  napi_get_cb_info(
    env,
    info,
    &mut 2,
    argv.as_mut_ptr(),
    ptr::null_mut(),
    &mut data,
  );

  let ctx = data.cast::<Arc<CompilationContext>>();

  ArgvAndContext {
    argv,
    ctx: Box::new((*ctx).clone()),
  }
}

unsafe extern "C" fn resolve(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);
  let param: PluginResolveHookParam = Env::from_raw(env)
    .from_js_value(JsUnknown::from_napi_value(env, argv[0]).unwrap())
    .unwrap();
  let hook_context: PluginHookContext = Env::from_raw(env)
    .from_js_value(JsUnknown::from_napi_value(env, argv[1]).unwrap())
    .unwrap();

  let (promise, result) = Env::from_raw(env)
    .create_deferred::<JsUnknown, Box<dyn FnOnce(Env) -> napi::Result<JsUnknown>>>()
    .unwrap();

  std::thread::spawn(move || {
    let resolved = ctx
      .plugin_driver
      .resolve(&param, &ctx, &hook_context)
      .map_err(|e| Error::new(Status::GenericFailure, format!("{}", e)));

    match resolved {
      Ok(resolved) => promise.resolve(Box::new(move |e| {
        resolved.map(|r| e.to_js_value(&r).unwrap()).ok_or_else(|| {
          Error::new(
            Status::GenericFailure,
            format!("can not resolve {:?}", param),
          )
        })
      })),
      Err(err) => promise.reject(Error::new(
        Status::GenericFailure,
        format!("can not resolve {:?}: {:?}", param, err),
      )),
    }
  });

  result.raw()
}

unsafe extern "C" fn add_watch_file(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let from: String = Env::from_raw(env)
    .from_js_value(JsUnknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a string when calling addWatchFile");
  let to: String = Env::from_raw(env)
    .from_js_value(JsUnknown::from_napi_value(env, argv[1]).unwrap())
    .expect("Argument 1 should be a string when calling addWatchFile");

  let from = ModuleId::new(&from, "", &ctx.config.root);
  let to = ModuleId::new(&to, "", &ctx.config.root);

  ctx.add_watch_files(from, vec![to]).unwrap();
  Env::from_raw(env).get_undefined().unwrap().raw()
}

unsafe extern "C" fn emit_file(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let params: EmitFileParams = Env::from_raw(env)
    .from_js_value(JsUnknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a EmitFileParams { name, content, resolvedPath, resourceType } when calling emitFile");

  ctx.emit_file(params);

  Env::from_raw(env).get_undefined().unwrap().raw()
}

unsafe extern "C" fn get_watch_files(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv: _, ctx } = get_argv_and_context_from_cb_info(env, info);

  let watch_graph = ctx.watch_graph.read();
  let mut watched_files = watch_graph
    .modules()
    .into_iter()
    .map(|p| p.to_string())
    .collect::<Vec<_>>();
  let module_graph = ctx.module_graph.read();
  let mut modules = module_graph
    .modules()
    .into_iter()
    .map(|s| s.id.resolved_path(&ctx.config.root))
    .collect::<Vec<_>>();

  modules.append(&mut watched_files);

  Env::from_raw(env).to_js_value(&modules).unwrap().raw()
}

unsafe extern "C" fn warn(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let message: String = Env::from_raw(env)
    .from_js_value(JsUnknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a string when calling warn");

  ctx.log_store.lock().add_warning(message);

  Env::from_raw(env).get_undefined().unwrap().raw()
}

unsafe extern "C" fn error(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let message: String = Env::from_raw(env)
    .from_js_value(JsUnknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a string when calling error");

  ctx.log_store.lock().add_error(message);

  Env::from_raw(env).get_undefined().unwrap().raw()
}

unsafe extern "C" fn source_map_enabled(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let id: String = Env::from_raw(env)
    .from_js_value(JsUnknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a string when calling get_modules_by_file");

  let enabled = ctx.sourcemap_enabled(&id);

  Env::from_raw(env).to_js_value(&enabled).unwrap().raw()
}
