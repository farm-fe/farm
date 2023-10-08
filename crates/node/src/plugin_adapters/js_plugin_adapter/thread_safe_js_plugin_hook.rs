#![allow(clippy::type_complexity)]
use std::{
  ffi::{c_void, CStr, CString},
  fmt::Debug,
  ptr,
  sync::{
    mpsc::{channel, Sender},
    Arc,
  },
};

use farmfe_core::{
  config::config_regex::ConfigRegex,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::ModuleType,
  plugin::{
    EmptyPluginHookParam, EmptyPluginHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginResolveHookParam, PluginResolveHookResult,
    PluginTransformHookParam, PluginTransformHookResult, PluginUpdateModulesHookParams,
  },
  serde::{de::DeserializeOwned, Serialize},
};
use napi::{
  bindgen_prelude::FromNapiValue,
  sys::{
    napi_call_function, napi_call_threadsafe_function, napi_callback_info, napi_create_function,
    napi_create_string_utf8, napi_create_threadsafe_function, napi_env, napi_get_cb_info,
    napi_get_named_property, napi_get_undefined, napi_release_threadsafe_function,
    napi_threadsafe_function, napi_unref_threadsafe_function, napi_value,
    ThreadsafeFunctionReleaseMode,
  },
  threadsafe_function::ThreadsafeFunctionCallMode,
  Env, JsFunction, JsObject, JsUnknown, NapiRaw, NapiValue, ValueType,
};

use super::context::create_js_context;

struct ThreadSafeJsPluginHook {
  raw_tsfn: napi_threadsafe_function,
}

unsafe impl Send for ThreadSafeJsPluginHook {}
unsafe impl Sync for ThreadSafeJsPluginHook {}

impl ThreadSafeJsPluginHook {
  pub fn new<P: Serialize, T: DeserializeOwned + Debug + Send + 'static>(
    env: &Env,
    func: JsFunction,
  ) -> Self {
    let mut raw_tsfn = ptr::null_mut();

    let mut async_resource_name = ptr::null_mut();
    let s = "thread_safe_js_plugin_hook";
    let len = s.len();
    let s = CString::new(s).unwrap();
    unsafe { napi_create_string_utf8(env.raw(), s.as_ptr(), len, &mut async_resource_name) };

    unsafe {
      napi_create_threadsafe_function(
        env.raw(),
        func.raw(),
        ptr::null_mut(),
        async_resource_name,
        0,
        1,
        ptr::null_mut(),
        Some(thread_finalize_cb),
        ptr::null_mut(),
        Some(call_js_cb::<P, T>),
        &mut raw_tsfn,
      );
      // exit as soon as possible the callback returned
      napi_unref_threadsafe_function(env.raw(), raw_tsfn);
    }

    Self { raw_tsfn }
  }

  pub fn call<P: Serialize, T: DeserializeOwned + Debug>(
    &self,
    param: P,
    ctx: Arc<CompilationContext>,
    hook_context: Option<PluginHookContext>,
  ) -> Result<Option<T>> {
    let (sender, receiver) = channel::<Result<Option<T>>>();

    unsafe {
      napi_call_threadsafe_function(
        self.raw_tsfn,
        Box::into_raw(Box::new((param, ctx, hook_context, sender))) as *mut c_void,
        ThreadsafeFunctionCallMode::NonBlocking.into(),
      );
    }

    receiver
      .recv()
      .unwrap_or_else(|e| panic!("recv error: {:?}", e.to_string()))
  }
}

impl Drop for ThreadSafeJsPluginHook {
  fn drop(&mut self) {
    unsafe {
      napi_release_threadsafe_function(self.raw_tsfn, ThreadsafeFunctionReleaseMode::release);
    }
  }
}

/// empty callback does nothing, just used as a parameter
unsafe extern "C" fn thread_finalize_cb(
  _raw_env: napi_env,
  _data: *mut c_void,
  _hint: *mut c_void,
) {
}

/// Thread safe function callback, call the real hook function and return its result. Promise is also supported.
unsafe extern "C" fn call_js_cb<P: Serialize, T: DeserializeOwned + Debug + Send + 'static>(
  raw_env: napi_env,
  func: napi_value,
  _context: *mut c_void,
  data: *mut c_void,
) {
  let mut recv = ptr::null_mut();
  napi_get_undefined(raw_env, &mut recv);

  let data: Box<(
    P,
    Arc<CompilationContext>,
    Option<PluginHookContext>,
    Sender<Result<Option<T>>>,
  )> = Box::from_raw(data.cast());
  let (param, ctx, hook_context, sender) = *data;
  // let js_context = create_js_context(raw_env, ctx);
  let mut js_func = JsObject::from_napi_value(raw_env, func).unwrap();
  let mut js_context = js_func
    .get_named_property::<JsObject>("farm_js_plugin_context")
    .unwrap();

  if JsUnknown::from_raw(raw_env, js_context.raw())
    .unwrap()
    .get_type()
    .unwrap()
    == ValueType::Undefined
  {
    let new_js_context = create_js_context(raw_env, ctx);
    js_func
      .set_named_property("farm_js_plugin_context", new_js_context)
      .unwrap();
    js_context = js_func
      .get_named_property::<JsObject>("farm_js_plugin_context")
      .unwrap();
  }

  unsafe fn to_napi_value<T: Serialize>(arg: T, raw_env: napi_env) -> napi_value {
    Env::from_raw(raw_env).to_js_value(&arg).unwrap().raw()
  }

  let mut args: Vec<napi_value> = vec![to_napi_value(param, raw_env), js_context.raw()];

  if let Some(hook_context) = hook_context {
    args.push(to_napi_value(hook_context, raw_env));
  }

  let mut result = ptr::null_mut();
  napi_call_function(raw_env, recv, func, args.len(), args.as_ptr(), &mut result);

  let result_obj = match JsUnknown::from_napi_value(raw_env, result) {
    Ok(result_obj) => result_obj,
    Err(e) => {
      sender
        .send(Err(CompilationError::NAPIError(format!(
          "Invalid hook return, except object. {:?}",
          e
        ))))
        .unwrap();
      return;
    }
  };

  // treat null or undefined as Ok(None)
  let ty = result_obj.get_type().unwrap();
  if ty == ValueType::Undefined || ty == ValueType::Null {
    sender.send(Ok(None)).unwrap();
    return;
  }

  // if the result is a promise, retrieve the data from promise.then, else return the object
  if result_obj.is_promise().unwrap() {
    retrieve_result_from_promise(sender, raw_env, result_obj.raw());
  } else {
    let result = Env::from_raw(raw_env)
      .from_js_value(result_obj)
      .map(|r| Some(r))
      .map_err(|e| {
        CompilationError::NAPIError(format!(
          "Invalid hook return, can not transform it to rust struct. {:?}",
          e
        ))
      });

    sender.send(result).unwrap();
  }
}

pub struct JsPluginResolveHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginResolveHookFilters,
}

macro_rules! new_js_plugin_hook {
  ($filter:ident, $js_filter:ident, $param:ident, $ret:ident) => {
    pub fn new(env: &Env, obj: JsObject) -> Self {
      let filters: $filter = unsafe {
        $js_filter::from_napi_value(
          env.raw(),
          obj
            .get_named_property::<JsObject>("filters")
            .expect("filters should be checked in js side")
            .raw(),
        )
        .unwrap()
        .into()
      };

      let func = obj
        .get_named_property::<JsFunction>("executor")
        .expect("executor should be checked in js side");

      Self {
        tsfn: ThreadSafeJsPluginHook::new::<$param, $ret>(env, func),
        filters,
      }
    }
  };
}

impl JsPluginResolveHook {
  new_js_plugin_hook!(
    PluginResolveHookFilters,
    JsPluginResolveHookFilters,
    PluginResolveHookParam,
    PluginResolveHookResult
  );

  pub fn call(
    &self,
    param: PluginResolveHookParam,
    ctx: Arc<CompilationContext>,
    hook_context: PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    let filtered = self.filters.importers.iter().any(|i| {
      if let Some(importer) = &param.importer {
        i.is_match(&importer.resolved_path(&ctx.config.root))
      } else {
        i.is_match("None")
      }
    }) && self
      .filters
      .sources
      .iter()
      .any(|f| f.is_match(&param.source));

    if filtered {
      self
        .tsfn
        .call::<PluginResolveHookParam, PluginResolveHookResult>(param, ctx, Some(hook_context))
    } else {
      Ok(None)
    }
  }
}

pub struct JsPluginLoadHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginLoadHookFilters,
}

impl JsPluginLoadHook {
  new_js_plugin_hook!(
    PluginLoadHookFilters,
    JsPluginLoadHookFilters,
    PluginLoadHookParam,
    PluginLoadHookResult
  );

  pub fn call(
    &self,
    param: PluginLoadHookParam,
    ctx: Arc<CompilationContext>,
    hook_context: PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    if self
      .filters
      .resolved_paths
      .iter()
      .any(|f| f.is_match(param.resolved_path))
    {
      self.tsfn.call::<PluginLoadHookParam, PluginLoadHookResult>(
        param.clone(),
        ctx,
        Some(hook_context),
      )
    } else {
      Ok(None)
    }
  }
}

pub struct JsPluginTransformHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginTransformHookFilters,
}

impl JsPluginTransformHook {
  new_js_plugin_hook!(
    PluginTransformHookFilters,
    JsPluginTransformHookFilters,
    PluginTransformHookParam,
    PluginTransformHookResult
  );

  pub fn call(
    &self,
    param: PluginTransformHookParam,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<PluginTransformHookResult>> {
    if self
      .filters
      .resolved_paths
      .iter()
      .any(|f| f.is_match(param.resolved_path))
      || self
        .filters
        .module_types
        .iter()
        .any(|ty| &param.module_type == ty)
    {
      self
        .tsfn
        .call::<PluginTransformHookParam, PluginTransformHookResult>(param, ctx, None)
    } else {
      Ok(None)
    }
  }
}

macro_rules! define_empty_params_js_plugin_hook {
  ($name:ident) => {
    pub struct $name {
      tsfn: ThreadSafeJsPluginHook,
    }

    impl $name {
      pub fn new(env: &Env, obj: JsObject) -> Self {
        let func = obj
          .get_named_property::<JsFunction>("executor")
          .expect("executor should be checked in js side");

        Self {
          tsfn: ThreadSafeJsPluginHook::new::<EmptyPluginHookParam, EmptyPluginHookResult>(
            env, func,
          ),
        }
      }

      pub fn call(
        &self,
        param: EmptyPluginHookParam,
        ctx: Arc<CompilationContext>,
      ) -> Result<Option<EmptyPluginHookResult>> {
        self
          .tsfn
          .call::<EmptyPluginHookParam, EmptyPluginHookResult>(param, ctx, None)
      }
    }
  };
}

define_empty_params_js_plugin_hook!(JsPluginBuildStartHook);
define_empty_params_js_plugin_hook!(JsPluginBuildEndHook);
define_empty_params_js_plugin_hook!(JsPluginFinishHook);

pub struct JsPluginUpdateModulesHook {
  tsfn: ThreadSafeJsPluginHook,
}

impl JsPluginUpdateModulesHook {
  pub fn new(env: &Env, obj: JsObject) -> Self {
    let func = obj
      .get_named_property::<JsFunction>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new::<PluginUpdateModulesHookParams, Vec<String>>(env, func),
    }
  }

  pub fn call(
    &self,
    param: PluginUpdateModulesHookParams,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<Vec<String>>> {
    self
      .tsfn
      .call::<PluginUpdateModulesHookParams, Vec<String>>(param, ctx, None)
  }
}

/// Resolve hook filters, works as `||`. If any importers or sources matches any regex item in the Vec, we treat it as filtered.
#[napi(object)]
struct JsPluginResolveHookFilters {
  pub importers: Vec<String>,
  pub sources: Vec<String>,
}

#[derive(Debug)]
struct PluginResolveHookFilters {
  pub importers: Vec<ConfigRegex>,
  pub sources: Vec<ConfigRegex>,
}

impl From<JsPluginResolveHookFilters> for PluginResolveHookFilters {
  fn from(f: JsPluginResolveHookFilters) -> Self {
    Self {
      importers: f
        .importers
        .into_iter()
        .map(|f| ConfigRegex::new(&f))
        .collect(),
      sources: f
        .sources
        .into_iter()
        .map(|f| ConfigRegex::new(&f))
        .collect(),
    }
  }
}

#[napi(object)]
pub struct JsPluginLoadHookFilters {
  pub resolved_paths: Vec<String>,
}

#[derive(Debug)]
pub struct PluginLoadHookFilters {
  pub resolved_paths: Vec<ConfigRegex>,
}

impl From<JsPluginLoadHookFilters> for PluginLoadHookFilters {
  fn from(f: JsPluginLoadHookFilters) -> Self {
    Self {
      resolved_paths: f
        .resolved_paths
        .into_iter()
        .map(|f| ConfigRegex::new(&f))
        .collect(),
    }
  }
}

#[napi(object)]
pub struct JsPluginTransformHookFilters {
  pub resolved_paths: Vec<String>,
  pub module_types: Vec<String>,
}

#[derive(Debug)]
pub struct PluginTransformHookFilters {
  pub resolved_paths: Vec<ConfigRegex>,
  pub module_types: Vec<ModuleType>,
}

impl From<JsPluginTransformHookFilters> for PluginTransformHookFilters {
  fn from(f: JsPluginTransformHookFilters) -> Self {
    Self {
      resolved_paths: f
        .resolved_paths
        .into_iter()
        .map(|f| ConfigRegex::new(&f))
        .collect(),
      module_types: f.module_types.into_iter().map(|ty| ty.into()).collect(),
    }
  }
}

/// retrieve result and catch error from the promise
pub fn retrieve_result_from_promise<T: DeserializeOwned>(
  sender: Sender<Result<Option<T>>>,
  env: napi_env,
  value: napi_value,
) {
  let mut then_ret_promise = ptr::null_mut();
  let then_c_string = unsafe { CStr::from_bytes_with_nul_unchecked(b"then\0") };
  let mut then_field = ptr::null_mut();
  unsafe { napi_get_named_property(env, value, then_c_string.as_ptr(), &mut then_field) };
  let mut then_callback = ptr::null_mut();

  // then
  unsafe {
    napi_create_function(
      env,
      then_c_string.as_ptr(),
      4,
      Some(then_cb::<T>),
      Box::into_raw(Box::new(sender.clone())) as _,
      &mut then_callback,
    );

    napi_call_function(
      env,
      value,
      then_field,
      1,
      [then_callback].as_ptr(),
      &mut then_ret_promise,
    );
  };

  // catch
  let catch_c_string = unsafe { CStr::from_bytes_with_nul_unchecked(b"catch\0") };
  let mut catch_field = ptr::null_mut();
  unsafe {
    napi_get_named_property(
      env,
      then_ret_promise,
      catch_c_string.as_ptr(),
      &mut catch_field,
    )
  };
  let mut catch_callback = ptr::null_mut();

  unsafe {
    napi_create_function(
      env,
      catch_c_string.as_ptr(),
      5,
      Some(catch_cb::<T>),
      Box::into_raw(Box::new(sender)) as _,
      &mut catch_callback,
    );

    napi_call_function(
      env,
      then_ret_promise,
      catch_field,
      1,
      [catch_callback].as_ptr(),
      ptr::null_mut(), // ignore result of catch
    );
  }
}

unsafe extern "C" fn then_cb<T: DeserializeOwned>(
  env: napi_env,
  info: napi_callback_info,
) -> napi_value {
  let mut this = ptr::null_mut();
  let mut data = ptr::null_mut();
  let mut resolved_value: [napi_value; 1] = [ptr::null_mut()];

  napi_get_cb_info(
    env,
    info,
    &mut 1,
    resolved_value.as_mut_ptr(),
    &mut this,
    &mut data,
  );

  let sender = Box::from_raw(data as *mut Sender<Result<Option<T>>>);

  let result = JsUnknown::from_napi_value(env, resolved_value[0]).unwrap();
  let ty = result.get_type().unwrap();

  if ty == ValueType::Undefined || ty == ValueType::Null {
    sender.send(Ok(None)).unwrap();
    return this;
  }

  let result = Env::from_raw(env)
    .from_js_value(result)
    .map(|r| Some(r))
    .map_err(|e| {
      CompilationError::NAPIError(format!(
        "Invalid hook return, can not transform it to rust struct. {:?}",
        e
      ))
    });

  sender.send(result).unwrap();

  this
}

unsafe extern "C" fn catch_cb<T: DeserializeOwned>(
  env: napi_env,
  info: napi_callback_info,
) -> napi_value {
  let mut this = ptr::null_mut();
  let mut data = ptr::null_mut();
  let mut rejected_value: [napi_value; 1] = [ptr::null_mut()];

  napi_get_cb_info(
    env,
    info,
    &mut 1,
    rejected_value.as_mut_ptr(),
    &mut this,
    &mut data,
  );

  let rejected_value = JsUnknown::from_raw_unchecked(env, rejected_value[0]);
  // detect if the rejected value is a js error object
  let is_error = rejected_value
    .get_type()
    .map(|ty| ty == ValueType::Object)
    .unwrap_or(false);
  let is_string = rejected_value
    .get_type()
    .map(|ty| ty == ValueType::String)
    .unwrap_or(false);

  let msg = if is_error {
    let rejected_value = rejected_value.coerce_to_object().unwrap();
    // get message and stack from the error object
    let message = rejected_value
      .get_named_property::<String>("message")
      .unwrap();
    let stack = rejected_value
      .get_named_property::<String>("stack")
      .unwrap();

    format!("{}\n{}", message, stack)
  } else if is_string {
    // get the string value
    rejected_value
      .coerce_to_string()
      .unwrap()
      .into_utf8()
      .unwrap()
      .as_str()
      .unwrap()
      .to_string()
  } else {
    String::from("unsupported error type for js plugins")
  };

  let sender = Box::from_raw(data as *mut Sender<Result<Option<T>>>);
  sender.send(Err(CompilationError::NAPIError(msg))).unwrap();

  this
}
