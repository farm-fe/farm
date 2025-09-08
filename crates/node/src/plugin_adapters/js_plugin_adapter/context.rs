use std::{
  ffi::{c_void, CString},
  ptr,
  sync::Arc,
  usize,
};

use napi::{
  bindgen_prelude::{FromNapiValue, JsObjectValue, Object, ObjectRef, ToNapiValue},
  sys::{
    napi_callback, napi_callback_info, napi_create_function, napi_create_object, napi_env,
    napi_get_cb_info, napi_value,
  },
  Env, Error, JsValue, Status, Unknown,
};

use farmfe_core::{
  context::{CompilationContext, EmitFileParams},
  module::ModuleId,
  plugin::{PluginHookContext, PluginResolveHookParam},
};

const RESOLVE: &str = "resolve";
const ADD_WATCH_FILE: &str = "addWatchFile";
const EMIT_FILE: &str = "emitFile";
const GET_WATCH_FILES: &str = "getWatchFiles";
const WARN: &str = "warn";
const ERROR: &str = "error";
const SOURCE_MAP_ENABLED: &str = "sourceMapEnabled";

use crate::plugin_adapters::js_plugin_adapter::context_methods::cache::{
  context_read_cache, context_read_cache_by_scope, context_write_cache, CONTEXT_READ_CACHE,
  CONTEXT_READ_CACHE_BY_SCOPE, CONTEXT_WRITE_CACHE,
};

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
pub unsafe fn create_js_context(raw_env: napi_env, ctx: Arc<CompilationContext>) -> ObjectRef {
  let mut js_context_ptr = ptr::null_mut();
  let mut js_context = {
    napi_create_object(raw_env, &mut js_context_ptr);
    Object::from_napi_value(raw_env, js_context_ptr).unwrap()
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
    (CONTEXT_WRITE_CACHE, context_write_cache),
    (CONTEXT_READ_CACHE, context_read_cache),
    (CONTEXT_READ_CACHE_BY_SCOPE, context_read_cache_by_scope),
  ];

  for (name, cb) in methods {
    js_context = attach_context_method(raw_env, js_context, name, Some(cb), ctx.clone());
  }

  js_context.create_ref().unwrap()
}

/// Create a js resolve function based on [farmfe_core::context::CompilationContext]
/// and attach it to the js context object
fn attach_context_method<'a>(
  env: napi_env,
  mut context: Object<'a>,
  name: &str,
  cb: napi_callback,
  ctx: Arc<CompilationContext>,
) -> Object<'a> {
  let len = name.len();
  let s = CString::new(name).unwrap();

  let mut func = ptr::null_mut();
  unsafe {
    napi_create_function(
      env,
      s.as_ptr(),
      len as isize,
      cb,
      Box::into_raw(Box::new(ctx)) as *mut c_void,
      &mut func,
    );

    context
      .set_named_property(name, Unknown::from_napi_value(env, func).unwrap())
      .unwrap();
  }

  context
}

#[repr(C)]
pub struct ArgvAndContext {
  pub argv: [napi_value; 2],
  pub ctx: Box<Arc<CompilationContext>>,
}

#[repr(C)]
pub struct DynamicArgvAndContext {
  pub argv: Vec<napi_value>,
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

pub unsafe extern "C" fn get_argv_and_context_from_cb_info_dynamic_arg_len(
  env: napi_env,
  info: napi_callback_info,
  mut len: usize,
) -> DynamicArgvAndContext {
  // accept 2 arguments at most
  let mut argv = Vec::with_capacity(len);
  (0..len).for_each(|_| argv.push(ptr::null_mut()));
  let mut data = ptr::null_mut();

  napi_get_cb_info(
    env,
    info,
    &mut len,
    argv.as_mut_ptr(),
    ptr::null_mut(),
    &mut data,
  );

  let ctx = data.cast::<Arc<CompilationContext>>();

  DynamicArgvAndContext {
    argv,
    ctx: Box::new((*ctx).clone()),
  }
}

unsafe extern "C" fn resolve(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let param: PluginResolveHookParam = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[0]).expect(
      "Argument should be a PluginResolveHookParam { source, importer, kind } when calling resolve",
    ))
    .expect(
      "Failed to convert argument to PluginResolveHookParam, please ensure your argument concert",
    );

  let hook_context: PluginHookContext = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[1]).unwrap())
    .unwrap();

  let binding = Env::from_raw(env);
  let (promise, result) = binding
    .create_deferred::<Unknown, Box<dyn FnOnce(Env) -> napi::Result<Unknown<'static>>>>()
    .unwrap();

  std::thread::spawn(move || {
    let resolved = ctx
      .plugin_driver
      .resolve(&param, &ctx, &hook_context)
      .map_err(|e| Error::new(Status::GenericFailure, format!("{e}")));

    match resolved {
      Ok(resolved) => promise.resolve(Box::new(move |e| e.to_js_value(&resolved))),
      Err(err) => promise.reject(Error::new(
        Status::GenericFailure,
        format!("can not resolve {param:?}: {err:?}"),
      )),
    }
  });

  result.raw()
}

unsafe extern "C" fn add_watch_file(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let from: String = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a string when calling addWatchFile");
  let to: String = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[1]).unwrap())
    .expect("Argument 1 should be a string when calling addWatchFile");

  let from = ModuleId::new(&from, "", &ctx.config.root);
  let to = ModuleId::new(&to, "", &ctx.config.root);

  ctx.add_watch_files(from, vec![to]).unwrap();
  ToNapiValue::to_napi_value(env, ()).unwrap()
}

unsafe extern "C" fn emit_file(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let params: EmitFileParams = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a EmitFileParams { name, content, resolvedPath, resourceType } when calling emitFile");

  ctx.emit_file(params);

  ToNapiValue::to_napi_value(env, ()).unwrap()
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
    .from_js_value(Unknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a string when calling warn");

  ctx.log_store.lock().add_warning(message);

  ToNapiValue::to_napi_value(env, ()).unwrap()
}

unsafe extern "C" fn error(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let message: String = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a string when calling error");

  ctx.log_store.lock().add_error(message);

  ToNapiValue::to_napi_value(env, ()).unwrap()
}

unsafe extern "C" fn source_map_enabled(env: napi_env, info: napi_callback_info) -> napi_value {
  let ArgvAndContext { argv, ctx } = get_argv_and_context_from_cb_info(env, info);

  let id: String = Env::from_raw(env)
    .from_js_value(Unknown::from_napi_value(env, argv[0]).unwrap())
    .expect("Argument 0 should be a string when calling get_modules_by_file");

  let enabled = ctx.sourcemap_enabled(&id);

  ToNapiValue::to_napi_value(env, enabled).unwrap()
}
